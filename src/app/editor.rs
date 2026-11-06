use ratatui::widgets::ListState;
use tui_input::Input;

use crate::{
    app::{Action, App},
    filter_criteria::{BackgroundColor, Brightness, TextContrast},
    search::{Appearance, Favorite, Filter, Ignored},
    text_input,
};

#[derive(Clone)]
struct BrowseSnapshot {
    filter: Filter,
    list: ListState,
}

pub struct QueryEdit {
    pub input: Input,
    pub error: Option<String>,
    before: BrowseSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterField {
    Name,
    Appearance,
    Favorite,
    Ignored,
    Brightness,
    BackgroundColor,
    TextContrast,
}

impl FilterField {
    fn next(self) -> Self {
        match self {
            Self::Name => Self::Appearance,
            Self::Appearance => Self::Favorite,
            Self::Favorite => Self::Ignored,
            Self::Ignored => Self::Brightness,
            Self::Brightness => Self::BackgroundColor,
            Self::BackgroundColor => Self::TextContrast,
            Self::TextContrast => Self::Name,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Name => Self::TextContrast,
            Self::Appearance => Self::Name,
            Self::Favorite => Self::Appearance,
            Self::Ignored => Self::Favorite,
            Self::Brightness => Self::Ignored,
            Self::BackgroundColor => Self::Brightness,
            Self::TextContrast => Self::BackgroundColor,
        }
    }
}

pub struct FiltersEdit {
    pub input: Input,
    pub appearance: Appearance,
    pub favorite: Favorite,
    pub ignored: Ignored,
    pub brightness: Option<Brightness>,
    pub background_color: Option<BackgroundColor>,
    pub text_contrast: Option<TextContrast>,
    pub field: FilterField,
    pub error: Option<String>,
    before: BrowseSnapshot,
    pub offset: usize,
}

pub enum Editor {
    Query(QueryEdit),
    Filters(FiltersEdit),
}

impl Editor {
    fn before(&self) -> &BrowseSnapshot {
        match self {
            Self::Query(edit) => &edit.before,
            Self::Filters(edit) => &edit.before,
        }
    }

    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Query(edit) => edit.error.as_deref(),
            Self::Filters(edit) => edit.error.as_deref(),
        }
    }
}

impl App {
    pub fn filters(&self) -> Option<&FiltersEdit> {
        match &self.editor {
            Some(Editor::Filters(edit)) => Some(edit),
            _ => None,
        }
    }

    pub fn query_editor(&self) -> Option<&QueryEdit> {
        match &self.editor {
            Some(Editor::Query(edit)) => Some(edit),
            _ => None,
        }
    }

    pub fn open_editor(&mut self, action: Action) {
        let before = BrowseSnapshot {
            filter: self.filter.clone(),
            list: self.theme_list,
        };
        let editor = match action {
            Action::EditQuery => {
                let name = self.filter.name_query();
                let mut query = self.filter.query();
                if name.is_empty() {
                    query.insert(0, ' ');
                }
                Editor::Query(QueryEdit {
                    input: Input::new(query).with_cursor(name.chars().count()),
                    error: None,
                    before,
                })
            }
            Action::EditFilters => Editor::Filters(FiltersEdit {
                input: Input::new(self.filter.name_query()),
                appearance: self.filter.appearance,
                favorite: self.filter.favorite,
                ignored: self.filter.ignored,
                brightness: self.filter.brightness,
                background_color: self.filter.background_color,
                text_contrast: self.filter.text_contrast,
                field: FilterField::Appearance,
                error: None,
                before,
                offset: self.theme_list.offset(),
            }),
            _ => return,
        };
        self.editor = Some(editor);
    }

    fn refresh_filters(&mut self, edit: &mut FiltersEdit) {
        match Filter::from_fields(
            edit.input.value(),
            edit.appearance,
            edit.favorite,
            edit.ignored,
            edit.brightness,
            edit.background_color,
            edit.text_contrast,
        ) {
            Ok(filter) => {
                self.apply_filter(filter);
                edit.error = None;
                edit.offset = 0;
            }
            Err(error) => {
                edit.error = Some(error);
            }
        }
    }

    pub fn update_editor(&mut self, action: Action, rows: usize) {
        let Some(mut editor) = self.editor.take() else {
            return;
        };
        if action == Action::Cancel {
            self.apply_filter(editor.before().filter.clone());
            self.theme_list = editor.before().list;
            return;
        }
        if action == Action::Confirm && editor.error().is_none() {
            self.record_session(true);
            return;
        }
        match &mut editor {
            Editor::Query(edit) => match action {
                Action::Next | Action::Previous | Action::PageNext | Action::PagePrevious => {
                    self.navigate(action, rows)
                }
                _ => {
                    if text_input::update(&mut edit.input, action) {
                        match Filter::parse(edit.input.value()) {
                            Ok(filter) => {
                                self.apply_filter(filter);
                                edit.error = None;
                            }
                            Err(error) => edit.error = Some(error),
                        }
                    }
                }
            },
            Editor::Filters(edit) => match action {
                Action::FieldNext => edit.field = edit.field.next(),
                Action::FieldPrevious => edit.field = edit.field.previous(),
                Action::ValuePrevious | Action::ValueNext => {
                    let next = action == Action::ValueNext;
                    match edit.field {
                        FilterField::Appearance => {
                            edit.appearance = if next {
                                edit.appearance.next()
                            } else {
                                edit.appearance.previous()
                            }
                        }
                        FilterField::Favorite => {
                            edit.favorite = if next {
                                edit.favorite.next()
                            } else {
                                edit.favorite.previous()
                            }
                        }
                        FilterField::Ignored => {
                            edit.ignored = if next {
                                edit.ignored.next()
                            } else {
                                edit.ignored.previous()
                            }
                        }
                        FilterField::Brightness => {
                            edit.brightness = cycle_choice(edit.brightness, &Brightness::ALL, next);
                        }
                        FilterField::BackgroundColor => {
                            edit.background_color =
                                cycle_choice(edit.background_color, &BackgroundColor::ALL, next);
                        }
                        FilterField::TextContrast => {
                            edit.text_contrast =
                                cycle_choice(edit.text_contrast, &TextContrast::ALL, next);
                        }
                        FilterField::Name => {}
                    }
                    if edit.field != FilterField::Name {
                        self.refresh_filters(edit);
                    }
                }
                Action::PagePrevious => edit.offset = edit.offset.saturating_sub(rows),
                Action::PageNext => {
                    edit.offset = edit
                        .offset
                        .saturating_add(rows)
                        .min(self.results.len().saturating_sub(rows))
                }
                _ if edit.field == FilterField::Name
                    && text_input::update(&mut edit.input, action) =>
                {
                    self.refresh_filters(edit)
                }
                _ => {}
            },
        }
        self.editor = Some(editor);
    }
}

fn cycle_choice<T: Copy + PartialEq>(value: Option<T>, choices: &[T], forward: bool) -> Option<T> {
    let count = choices.len() + 1;
    let index = choices
        .iter()
        .position(|choice| Some(*choice) == value)
        .map_or(0, |index| index + 1);
    let index = (index + if forward { 1 } else { count - 1 }) % count;
    index.checked_sub(1).map(|index| choices[index])
}
