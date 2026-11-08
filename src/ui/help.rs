use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};

use crate::{
    app::{Action, App, InputContext, saved::DialogKind},
    config::Config,
    keybindings::{KeyBindings, KeyChord},
    ui::{
        theme::UiTheme,
        widgets::{draw_scrollbar, panel},
    },
};

struct HelpGroup {
    actions: &'static [Action],
    description: &'static str,
    detail: Option<(&'static [Action], &'static str)>,
    priority: u8,
    section: &'static str,
}

impl HelpGroup {
    fn with_detail(mut self, actions: &'static [Action], description: &'static str) -> Self {
        self.detail = Some((actions, description));
        self
    }
}

fn group(
    section: &'static str,
    actions: &'static [Action],
    description: &'static str,
    priority: u8,
) -> HelpGroup {
    HelpGroup {
        actions,
        description,
        detail: None,
        priority,
        section,
    }
}

fn groups(app: &App, context: InputContext) -> Vec<HelpGroup> {
    use crate::app::Action::*;
    let mut groups = Vec::new();
    match context {
        InputContext::Browse | InputContext::Colors | InputContext::ThemeInfo => {
            let browsing = context == InputContext::Browse;
            let info_focused = context == InputContext::ThemeInfo;
            let info_open = app.theme_info.is_some();
            let colors_open = app.color_details.is_open() && !info_open;
            let panel_open = info_open || colors_open;
            if !info_focused || !app.info_is_credits() || app.selected_credit().is_some() {
                groups.extend([
                    group(
                        "Navigation",
                        &[Next, Previous],
                        match context {
                            InputContext::Colors => "Roles",
                            InputContext::ThemeInfo if app.info_is_credits() => "Select",
                            InputContext::ThemeInfo => "Scroll",
                            _ => "Move",
                        },
                        2,
                    ),
                    group("Navigation", &[PageNext, PagePrevious], "Page", 5),
                    group("Navigation", &[First, Last], "Ends", 5),
                ]);
            }
            if panel_open {
                let (section, description, detail) = if info_open {
                    (
                        "Theme info",
                        if browsing { "Info" } else { "Current" },
                        "Switch between the current theme and theme info",
                    )
                } else {
                    (
                        "Preview and comparison",
                        if browsing { "Colors" } else { "Current" },
                        "Switch between the current theme and color roles",
                    )
                };
                groups.push(
                    group(section, &[FieldNext], description, 2)
                        .with_detail(&[FieldNext, FieldPrevious], detail),
                );
            }
            if info_focused {
                groups.push(group(
                    "Theme info",
                    &[ValuePrevious, ValueNext],
                    "Section",
                    2,
                ));
                if app.can_switch_info_target() {
                    groups.push(group(
                        "Theme info",
                        &[SwitchThemeTarget],
                        "Target",
                        if app.info_is_credits() { 2 } else { 3 },
                    ));
                }
            } else if context == InputContext::Colors && app.can_switch_color_target() {
                groups.push(group(
                    "Preview and comparison",
                    &[SwitchThemeTarget],
                    "Target",
                    2,
                ));
            }
            let condition_priority = if !browsing {
                4
            } else if app.mode.is_compare() {
                3
            } else {
                2
            };
            groups.extend([
                group("Conditions", &[EditQuery], "Search", condition_priority),
                group("Conditions", &[EditFilters], "Filters", condition_priority),
            ]);
            if !app.filter.is_default() {
                groups.push(group("Conditions", &[ClearFilters], "Reset", 4));
            }
            groups.extend([
                group("Organize and save", &[OpenSaved], "Saved", 4),
                group("Organize and save", &[SaveFilter], "Save", 4),
            ]);
            if browsing {
                groups.push(group(
                    "Preview and comparison",
                    &[TogglePin],
                    "Pin",
                    if app.mode.is_compare() { 2 } else { 3 },
                ));
            }
            if !app.mode.is_compare() {
                groups.push(group(
                    "Preview and comparison",
                    &[ToggleCompare],
                    "Compare",
                    if browsing { 3 } else { 4 },
                ));
            }
            groups.extend([
                group(
                    "Preview and comparison",
                    &[PresetPrevious, PresetNext],
                    "Preview",
                    if browsing { 2 } else { 3 },
                ),
                group("Preview and comparison", &[OpenPreset], "Select preview", 5),
            ]);
            if !colors_open {
                groups.push(group(
                    "Preview and comparison",
                    &[ToggleColors],
                    "Colors",
                    3,
                ));
            }
            if !info_open {
                groups.push(group(
                    "Preview and comparison",
                    &[OpenThemeInfo],
                    "Theme info",
                    3,
                ));
            }
            groups.push(group("Organize and save", &[OpenCopy], "Copy", 3));
            if browsing {
                groups.extend([
                    group("Organize and save", &[ToggleFavorite], "Favorite", 4),
                    group("Organize and save", &[ToggleIgnore], "Ignore", 4),
                ]);
            }
            if app.mode.is_compare() {
                groups.push(group(
                    "Preview and comparison",
                    &[ToggleLayout],
                    "Layout",
                    5,
                ));
            }
            if let Some(maximized) = app.panel_maximized() {
                groups.push(group(
                    "Preview and comparison",
                    &[TogglePanelMaximize],
                    if maximized { "Auto" } else { "Maximize" },
                    5,
                ));
            }
            groups.push(group(
                "Preview and comparison",
                &[ToggleThemeList],
                "List",
                5,
            ));
            if info_focused && app.selected_credit().is_some() {
                groups.push(group("Theme info", &[Confirm], "Open", 1));
            }
            if app.mode.is_compare() {
                groups.push(if panel_open {
                    group("Preview and comparison", &[ToggleCompare], "Explore", 4)
                } else {
                    group("Preview and comparison", &[Cancel], "Explore", 1)
                        .with_detail(&[Cancel, ToggleCompare], "Back to Explore")
                });
            }
            if info_open {
                groups.push(
                    group("Theme info", &[Cancel], "Close", 1)
                        .with_detail(&[Cancel, OpenThemeInfo], "Close theme info"),
                );
            } else if colors_open {
                groups.push(
                    group("Preview and comparison", &[Cancel], "Close", 1)
                        .with_detail(&[Cancel, ToggleColors], "Close color details"),
                );
            }
        }
        InputContext::Preset => {
            groups.extend([
                group("Selection", &[FieldNext, FieldPrevious], "Field", 2),
                group("Selection", &[ValuePrevious, ValueNext], "Value", 2),
                group("Selection", &[Confirm], "Use", 1),
                group("Selection", &[Cancel], "Cancel", 1),
            ]);
        }
        InputContext::Copy => {
            groups.push(group("Selection", &[Next, Previous], "Format", 2));
            if app.can_switch_theme_target() {
                groups.push(group("Selection", &[SwitchThemeTarget], "Target", 2));
            }
            groups.push(group("Selection", &[Confirm], "Copy", 1));
            groups.push(group("Selection", &[Cancel], "Cancel", 1));
        }
        InputContext::Help => {
            groups.extend([
                group("Navigation", &[Next, Previous], "Scroll", 2),
                group("Navigation", &[PageNext, PagePrevious], "Page", 5),
                group("Navigation", &[First, Last], "Ends", 5),
                group("General", &[Cancel, ToggleHelp], "Close", 0),
            ]);
        }
        InputContext::Saved => {
            groups.extend([
                group("Navigation", &[Next, Previous], "Move", 2),
                group("Navigation", &[PageNext, PagePrevious], "Page", 5),
                group("Navigation", &[First, Last], "Ends", 5),
                group("Saved conditions", &[SaveFilter], "Save current", 3),
            ]);
            if app.selected_saved_filter().is_some() {
                groups.push(group("Saved conditions", &[RenameFilter], "Rename", 4));
                groups.push(group("Saved conditions", &[DeleteFilter], "Delete", 4));
            }
            groups.push(group("Saved conditions", &[Confirm], "Apply", 1));
            groups.push(group("Saved conditions", &[Cancel], "Back", 1));
        }
        InputContext::DialogName | InputContext::DialogConfirm => {
            if context.has_text() {
                groups.push(group("Editing", &[ClearInput], "Clear", 4));
            }
            let dialog = app.dialog.as_ref().expect("dialog context");
            let description = match dialog.kind {
                DialogKind::New { .. } => "Save",
                DialogKind::Rename { .. } => "Rename",
                DialogKind::Delete { .. } => "Delete",
            };
            groups.push(group("Editing", &[Confirm], description, 1));
            groups.push(group("Editing", &[Cancel], "Cancel", 1));
        }
        InputContext::Query | InputContext::FilterName | InputContext::FilterChoice => {
            if context != InputContext::Query {
                groups.push(group("Editing", &[FieldNext, FieldPrevious], "Field", 2));
            }
            if context == InputContext::FilterChoice {
                groups.push(group("Editing", &[ValuePrevious, ValueNext], "Value", 2));
            }
            if context.has_text() {
                groups.push(group("Editing", &[ClearInput], "Clear", 4));
            }
            if context == InputContext::Query {
                groups.push(group("Navigation", &[Next, Previous], "Move", 3));
            }
            groups.push(group(
                "Navigation",
                &[PageNext, PagePrevious],
                if context == InputContext::Query {
                    "Page"
                } else {
                    "Results"
                },
                5,
            ));
            groups.push(group("Editing", &[Confirm], "Apply", 1));
            groups.push(group("Editing", &[Cancel], "Cancel", 1));
        }
    }
    if context.has_text() {
        groups.splice(
            0..0,
            [
                &[CursorPrevious][..],
                &[CursorNext],
                &[CursorWordPrevious],
                &[CursorWordNext],
                &[CursorFirst],
                &[CursorLast],
                &[DeleteBefore],
                &[DeleteAfter],
                &[DeleteWordBefore],
                &[DeleteWordAfter],
            ]
            .into_iter()
            .map(|actions| group("Editing", actions, "", 100)),
        );
    }
    groups.push(group("General", &[Quit], "Quit", 0));
    if context != InputContext::Help {
        groups.push(group("General", &[ToggleHelp], "Help", 0));
    }
    groups.retain(|group| {
        group
            .actions
            .iter()
            .any(|action| app.action_enabled(*action))
    });
    groups
}

struct RenderedGroup {
    keys: String,
    description: &'static str,
    priority: u8,
}

fn key_labels(keys: impl IntoIterator<Item = KeyChord>) -> Vec<String> {
    let mut labels = Vec::new();
    for key in keys {
        let label = key.label();
        if !labels.contains(&label) {
            labels.push(label);
        }
    }
    labels
}

fn detail_description(app: &App, context: InputContext, action: Action, short: &str) -> String {
    use crate::app::Action::*;
    let target = match context {
        InputContext::Colors => "color role",
        InputContext::Preset => "preview",
        InputContext::Copy => "copy format",
        InputContext::ThemeInfo => "credit",
        InputContext::Saved => "saved filter",
        _ => "theme",
    };
    match action {
        Next | Previous
            if context == InputContext::Help
                || (context == InputContext::ThemeInfo && !app.info_is_credits()) =>
        {
            format!(
                "Scroll {} one line",
                if action == Next { "down" } else { "up" }
            )
        }
        Next | Previous => format!(
            "Select the {} {target}",
            if action == Next { "next" } else { "previous" }
        ),
        PageNext | PagePrevious => format!(
            "{} {} one page",
            if matches!(
                context,
                InputContext::Help
                    | InputContext::ThemeInfo
                    | InputContext::FilterChoice
                    | InputContext::FilterName
            ) {
                "Scroll"
            } else {
                "Move"
            },
            if action == PageNext {
                "forward"
            } else {
                "backward"
            }
        ),
        First | Last if matches!(context, InputContext::Help | InputContext::ThemeInfo) => format!(
            "Go to the {} of {}",
            if action == First { "beginning" } else { "end" },
            if context == InputContext::ThemeInfo && app.info_is_credits() {
                "credits"
            } else if context == InputContext::ThemeInfo {
                "overview"
            } else {
                "help"
            }
        ),
        First | Last => format!(
            "Select the {} {target}",
            if action == First { "first" } else { "last" }
        ),
        PresetPrevious => "Use the previous preview".into(),
        PresetNext => "Use the next preview".into(),
        ToggleThemeList => "Show or hide the theme list".into(),
        ToggleCompare => {
            if app.mode.is_compare() {
                "Back to Explore".into()
            } else {
                "Compare the pinned theme with the current theme".into()
            }
        }
        TogglePin => "Pin the current theme; unpin if already pinned".into(),
        ToggleColors => if app.mode.is_compare() {
            "Open color details for both themes"
        } else {
            "Open color details for one theme"
        }
        .into(),
        TogglePanelMaximize => if app.panel_maximized() == Some(true) {
            "Use automatic panel layout; show previews when space allows"
        } else {
            "Maximize the panel; hide the theme list and previews"
        }
        .into(),
        ToggleLayout => "Switch between side-by-side and stacked views".into(),
        ToggleFavorite => "Add or remove the current theme's favorite mark".into(),
        ToggleIgnore => "Add or remove the current theme's ignored mark".into(),
        EditQuery => "Edit the theme search query".into(),
        EditFilters => "Edit name, appearance, marks and color filters".into(),
        ClearFilters => "Reset all conditions to Initial".into(),
        OpenPreset => "Open the preview selector".into(),
        OpenCopy => "Choose a target and format to copy".into(),
        OpenThemeInfo => "Open or close theme information".into(),
        SwitchThemeTarget => "Switch between Current and Pin".into(),
        SaveFilter => "Save the current exploration condition with a name".into(),
        OpenSaved => "Open saved filters to apply or manage them".into(),
        RenameFilter => "Rename the selected saved filter".into(),
        DeleteFilter => "Confirm deletion of the selected saved filter".into(),
        FieldNext | FieldPrevious if context == InputContext::Preset => {
            "Switch between Sample and Content".into()
        }
        FieldNext | FieldPrevious if context == InputContext::ThemeInfo => {
            "Switch between the current theme and theme info".into()
        }
        FieldNext | FieldPrevious => format!(
            "Edit the {} filter field",
            if action == FieldNext {
                "next"
            } else {
                "previous"
            }
        ),
        ValueNext | ValuePrevious if context == InputContext::ThemeInfo => {
            "Switch between Overview and Credits".into()
        }
        ValueNext | ValuePrevious => format!(
            "Use the {} value for this field",
            if action == ValueNext {
                "next"
            } else {
                "previous"
            }
        ),
        CursorPrevious => "Move the cursor left one character".into(),
        CursorNext => "Move the cursor right one character".into(),
        CursorWordPrevious => "Move the cursor to the previous word".into(),
        CursorWordNext => "Move the cursor to the next word".into(),
        CursorFirst => "Move the cursor to the start of the input".into(),
        CursorLast => "Move the cursor to the end of the input".into(),
        DeleteBefore => "Delete the character before the cursor".into(),
        DeleteAfter => "Delete the character after the cursor".into(),
        DeleteWordBefore => "Delete the word before the cursor".into(),
        DeleteWordAfter => "Delete the word after the cursor".into(),
        ClearInput => "Clear the entire input".into(),
        Confirm => match context {
            InputContext::Saved => "Apply the selected filter and return",
            InputContext::Preset => "Use the selected preview and return",
            InputContext::Copy => "Copy the selected theme in the chosen format",
            InputContext::ThemeInfo => "Open the selected URL in the browser",
            InputContext::DialogConfirm => "Delete the selected saved filter",
            InputContext::DialogName => "Save the entered name",
            _ => "Apply the edited conditions and return",
        }
        .into(),
        Cancel => match context {
            InputContext::Saved => "Return without applying a filter",
            InputContext::Help => "Close help and return",
            InputContext::ThemeInfo => "Close theme info and return",
            _ => "Cancel and return without applying changes",
        }
        .into(),
        ToggleHelp if context == InputContext::Help => "Close help and return".into(),
        ToggleHelp => "Open help for this screen".into(),
        Quit => "Quit the application".into(),
        _ => short.into(),
    }
}

fn wrap_words(value: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in value.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_owned()
        } else {
            format!("{current} {word}")
        };
        if Line::raw(&candidate).width() <= width {
            current = candidate;
        } else {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            let mut parts = wrap_lines(vec![Line::raw(word.to_owned())], width)
                .into_iter()
                .map(|line| line.to_string())
                .collect::<Vec<_>>();
            current = parts.pop().unwrap_or_default();
            lines.extend(parts);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

fn help_entry(
    keys: &str,
    description: &str,
    width: usize,
    key_width: usize,
    theme: UiTheme,
) -> Vec<Line<'static>> {
    let description_x = key_width + 2;
    if width <= description_x {
        let mut lines = wrap_lines(
            vec![Line::styled(keys.to_owned(), theme.help_key_style())],
            width,
        );
        lines.extend(wrap_words(description, width).into_iter().map(Line::raw));
        return lines;
    }
    let keys = if Line::raw(keys).width() <= key_width {
        vec![keys.to_owned()]
    } else {
        wrap_words(keys, key_width)
    };
    let descriptions = wrap_words(description, width - description_x);
    (0..keys.len().max(descriptions.len()))
        .map(|index| {
            let key = keys.get(index).cloned().unwrap_or_default();
            let padding = description_x - Line::raw(&key).width();
            Line::from(vec![
                Span::styled(key, theme.help_key_style()),
                Span::raw(format!(
                    "{}{}",
                    " ".repeat(padding),
                    descriptions.get(index).map_or("", String::as_str)
                )),
            ])
        })
        .collect()
}

pub fn footer(app: &App, width: u16, keybindings: &KeyBindings, theme: UiTheme) -> Line<'static> {
    let context = app.context();
    let groups = groups(app, context);
    let (right, left): (Vec<_>, Vec<_>) = groups
        .iter()
        .filter(|group| group.priority < 100)
        .filter(|group| !has_modal_actions(context) || group.section == "General")
        .partition(|group| group.actions.contains(&Action::ToggleHelp));
    let tuples = |groups: Vec<&HelpGroup>| {
        groups
            .into_iter()
            .map(|group| (group.actions, group.description, group.priority))
            .collect::<Vec<_>>()
    };
    let right = action_footer(context, &tuples(right), width, keybindings, theme);
    let remaining = width.saturating_sub(right.width() as u16);
    let left_width = if right.width() > 0 {
        remaining.saturating_sub(2)
    } else {
        remaining
    };
    let mut left = action_footer(context, &tuples(left), left_width, keybindings, theme);
    if right.width() > 0 {
        left.spans
            .push(Span::raw(" ".repeat(usize::from(remaining) - left.width())));
        left.spans.extend(right.spans);
    }
    left
}

pub fn has_modal_actions(context: InputContext) -> bool {
    matches!(
        context,
        InputContext::Preset
            | InputContext::Copy
            | InputContext::DialogName
            | InputContext::DialogConfirm
    )
}

pub fn modal_footer(
    app: &App,
    width: u16,
    keybindings: &KeyBindings,
    theme: UiTheme,
) -> Line<'static> {
    let context = app.context();
    let tuples: Vec<_> = groups(app, context)
        .into_iter()
        .filter(|group| {
            group.section != "General"
                && group.actions.iter().any(|action| {
                    matches!(
                        action,
                        Action::Next
                            | Action::SwitchThemeTarget
                            | Action::Previous
                            | Action::Confirm
                            | Action::Cancel
                            | Action::FieldNext
                            | Action::FieldPrevious
                            | Action::ValueNext
                            | Action::ValuePrevious
                    )
                })
        })
        .map(|group| {
            let actions = if group.actions.contains(&Action::FieldNext) {
                &[Action::FieldNext][..]
            } else {
                group.actions
            };
            (actions, group.description, group.priority)
        })
        .collect();
    action_footer(context, &tuples, width, keybindings, theme)
}

pub fn action_footer(
    context: InputContext,
    groups: &[(&[Action], &'static str, u8)],
    width: u16,
    keybindings: &KeyBindings,
    theme: UiTheme,
) -> Line<'static> {
    let groups = groups
        .iter()
        .filter_map(|(actions, description, priority)| {
            let keys = key_labels(
                actions
                    .iter()
                    .filter_map(|action| keybindings.primary_key_in(*action, context)),
            );
            (!keys.is_empty()).then(|| RenderedGroup {
                keys: keys.join("/"),
                description,
                priority: *priority,
            })
        })
        .collect();
    fit(groups, width, theme)
}

fn fit(mut groups: Vec<RenderedGroup>, width: u16, theme: UiTheme) -> Line<'static> {
    loop {
        let mut spans = Vec::new();
        for (index, group) in groups.iter().enumerate() {
            if index > 0 {
                spans.push(Span::raw("  "));
            }
            spans.push(Span::styled(
                group.keys.clone(),
                theme.role_style(theme.key),
            ));
            spans.push(Span::styled(
                format!(" {}", group.description),
                theme.role_style(theme.muted),
            ));
        }
        let line = Line::from(spans);
        if line.width() <= usize::from(width) {
            return line;
        }
        let Some(index) = groups
            .iter()
            .enumerate()
            .max_by_key(|(index, group)| (group.priority, *index))
            .map(|(index, _)| index)
        else {
            return Line::default();
        };
        groups.remove(index);
    }
}

pub fn title(context: InputContext, app: &App) -> &'static str {
    match context {
        InputContext::Colors => "Color details",
        InputContext::Browse => app.mode.label(),
        InputContext::Preset => "Select preview",
        InputContext::Copy => "Copy theme",
        InputContext::ThemeInfo => "Theme info",
        InputContext::Query => "Search",
        InputContext::FilterName | InputContext::FilterChoice => "Filters",
        InputContext::Saved => "Saved filters",
        InputContext::DialogName => match app.dialog.as_ref().map(|dialog| &dialog.kind) {
            Some(DialogKind::Rename { .. }) => "Rename filter",
            _ => "Save filter",
        },
        InputContext::DialogConfirm => "Delete confirmation",
        _ => "Editing",
    }
}

pub fn detail_lines(
    app: &App,
    config: &Config,
    context: InputContext,
    width: usize,
) -> Vec<Line<'static>> {
    let groups = groups(app, context);
    let mut lines = Vec::new();
    let entries: Vec<_> = groups
        .iter()
        .flat_map(|group| {
            let actions = group.detail.map_or(group.actions, |detail| detail.0);
            let rows: Vec<_> = if let Some((_, description)) = group.detail {
                vec![(actions, description.to_owned())]
            } else {
                actions
                    .iter()
                    .map(|action| {
                        (
                            std::slice::from_ref(action),
                            detail_description(app, context, *action, group.description),
                        )
                    })
                    .collect()
            };
            rows.into_iter()
                .filter_map(|(actions, description)| {
                    let keys = key_labels(
                        actions
                            .iter()
                            .flat_map(|action| config.keybindings.keys_in(*action, context)),
                    );
                    (!keys.is_empty()).then(|| (group.section, keys.join("  "), description))
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let key_width = entries
        .iter()
        .map(|(_, keys, _)| Line::raw(keys).width())
        .max()
        .unwrap_or(0)
        .max(16)
        .min(width / 2);
    for section in [
        "Navigation",
        "Preview and comparison",
        "Conditions",
        "Organize and save",
        "Selection",
        "Theme info",
        "Saved conditions",
        "Editing",
        "General",
    ] {
        let entries: Vec<_> = entries
            .iter()
            .filter(|(entry_section, _, _)| *entry_section == section)
            .flat_map(|(_, keys, description)| {
                help_entry(keys, description, width, key_width, config.ui_theme)
            })
            .collect();
        if entries.is_empty() {
            continue;
        }
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        lines.push(Line::styled(section, config.ui_theme.heading_style()));
        lines.extend(entries);
    }
    lines
}

struct HelpContent {
    block: Block<'static>,
    area: Rect,
    lines: Vec<Line<'static>>,
}

fn content(app: &App, config: &Config, area: Rect) -> Option<HelpContent> {
    let help = app.help.as_ref()?;
    let block = panel(
        format!("Asinus · Help · {}", title(help.context, app)),
        config.ui_theme,
    );
    let inner = block.inner(area);
    let lines = wrap_lines(
        detail_lines(app, config, help.context, inner.width as usize),
        inner.width as usize,
    );
    Some(HelpContent {
        block,
        area: inner,
        lines,
    })
}

pub fn prepare(app: &mut App, config: &Config, area: Rect) {
    let Some(content) = content(app, config, area) else {
        return;
    };
    let help = app.help.as_mut().unwrap();
    help.offset = help.offset.min(
        content
            .lines
            .len()
            .saturating_sub(content.area.height as usize),
    );
}

pub fn draw(frame: &mut Frame, app: &App, config: &Config, area: Rect) {
    let Some(content) = content(app, config, area) else {
        return;
    };
    // Include the right gutter so a scrollbar from the covered view cannot show through.
    let covered = Rect::new(area.x, area.y, area.width.saturating_add(1), area.height)
        .intersection(frame.area());
    frame.render_widget(Clear, covered);
    frame.render_widget(content.block, area);
    draw_scrollbar(
        frame,
        content.area,
        content.lines.len(),
        app.help.as_ref().unwrap().offset,
        config.ui_theme,
    );
    let lines: Vec<_> = content
        .lines
        .into_iter()
        .skip(app.help.as_ref().unwrap().offset)
        .collect();
    frame.render_widget(
        Paragraph::new(lines).style(config.ui_theme.base_style()),
        content.area,
    );
}

fn wrap_lines(lines: Vec<Line<'static>>, width: usize) -> Vec<Line<'static>> {
    if width == 0 {
        return Vec::new();
    }
    let mut wrapped = Vec::new();
    for line in lines {
        let mut spans = Vec::new();
        let mut used = 0;
        for grapheme in line.styled_graphemes(Style::default()) {
            let span = Span::styled(grapheme.symbol.to_owned(), grapheme.style);
            let columns = span.width();
            if used + columns > width && !spans.is_empty() {
                wrapped.push(Line::from(std::mem::take(&mut spans)));
                used = 0;
            }
            spans.push(span);
            used += columns;
        }
        wrapped.push(Line::from(spans));
    }
    wrapped
}

#[cfg(test)]
mod tests {
    use ratatui::crossterm::event::{KeyCode, KeyModifiers};
    use rstest::rstest;

    use super::*;
    use crate::keybindings::KeyChord;

    #[test]
    fn theme_info_keeps_main_shortcuts_visible_and_uses_the_remapped_key_in_help() {
        let app = App::default();
        let mut config = Config::default();
        for (key, action) in &mut config.keybindings.bindings {
            if *action == Action::OpenThemeInfo {
                *key = KeyChord::new(KeyCode::F(6), KeyModifiers::NONE);
            }
        }
        let short = footer(&app, 76, &config.keybindings, config.ui_theme).to_string();
        assert!(!short.contains("Theme info"));
        for label in ["Search", "Filters", "Preview", "Help", "Quit"] {
            assert!(short.contains(label), "{short}");
        }
        let wide = footer(&app, 400, &config.keybindings, config.ui_theme).to_string();
        assert!(wide.contains("F6 Theme info"));
        assert!(!wide.contains("Maximize"));
        let details = detail_lines(&app, &config, InputContext::Browse, 76);
        assert!(details.iter().any(|line| {
            let line = line.to_string();
            line.starts_with("F6 ") && line.contains("Open or close theme information")
        }));
    }

    #[rstest]
    #[case(false, "j/k Move  / Search  f Filters  q Quit  F1 Help")]
    #[case(true, "j/k Move  p Pin  [/] Preview  Esc Explore  q Quit  F1 Help")]
    fn browse_help_groups_related_actions_and_keeps_the_main_operations_at_minimum_width(
        #[case] compare: bool,
        #[case] minimum: &str,
    ) {
        let app = if compare {
            crate::test_support::comparison_in_context(InputContext::Browse)
        } else {
            App::default()
        };
        let config = Config::default();
        let wide = footer(&app, 400, &config.keybindings, config.ui_theme).to_string();
        for block in [
            "j/k Move  C-f/C-b Page  g/G Ends  / Search  f Filters  s Saved  S Save  p Pin",
            "[/] Preview  P Select preview  d Colors  i Theme info",
            "y Copy  m Favorite  x Ignore",
        ] {
            assert!(wide.contains(block), "{wide}");
        }
        if compare {
            assert!(
                wide.trim_end_matches("F1 Help")
                    .trim_end()
                    .ends_with("v Layout  z List  Esc Explore  q Quit"),
                "{wide}"
            );
            assert!(!wide.contains("  c Explore"));
        } else {
            assert!(wide.contains("p Pin  c Compare  [/] Preview"), "{wide}");
            assert!(
                wide.trim_end_matches("F1 Help")
                    .trim_end()
                    .ends_with("z List  q Quit"),
                "{wide}"
            );
        }
        assert_eq!(
            footer(&app, 58, &config.keybindings, config.ui_theme)
                .to_string()
                .split_whitespace()
                .collect::<Vec<_>>(),
            minimum.split_whitespace().collect::<Vec<_>>(),
        );
    }

    #[rstest]
    #[case(InputContext::Colors)]
    #[case(InputContext::ThemeInfo)]
    fn panel_layout_help_shows_the_next_action_for_both_focuses_and_remapped_keys(
        #[case] context: InputContext,
        #[values(false, true)] current_focus: bool,
        #[values(false, true)] maximized: bool,
    ) {
        let mut app = crate::test_support::comparison_in_context(context);
        if context == InputContext::ThemeInfo {
            app.update(Action::ValueNext, 14);
        }
        if current_focus {
            app.update(Action::FieldNext, 14);
        }
        if maximized {
            app.update(Action::TogglePanelMaximize, 14);
        }
        let mut config = Config::default();
        let label = if maximized { "Auto" } else { "Maximize" };
        let wide = footer(&app, 400, &config.keybindings, config.ui_theme).to_string();
        assert!(wide.contains(&format!("Z {label}  z List")), "{wide}");
        let short = footer(&app, 58, &config.keybindings, config.ui_theme).to_string();
        for expected in ["j/k ", "Esc Close", "Quit", "F1 Help"] {
            assert!(short.contains(expected), "{short}");
        }
        assert!(!short.contains("Z "), "{short}");
        assert!(!short.contains("z List"), "{short}");
        for (key, action) in &mut config.keybindings.bindings {
            if *action == Action::TogglePanelMaximize {
                *key = KeyChord::new(KeyCode::F(6), KeyModifiers::NONE);
            }
        }
        let wide = footer(&app, 400, &config.keybindings, config.ui_theme).to_string();
        assert!(wide.contains(&format!("F6 {label}  z List")), "{wide}");
        let details = detail_lines(&app, &config, app.context(), 400)
            .iter()
            .map(Line::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(details.contains("F6"));
        assert!(details.contains(if maximized {
            "Use automatic panel layout; show previews when space allows"
        } else {
            "Maximize the panel; hide the theme list and previews"
        }));
    }

    #[rstest]
    fn info_help_lists_common_actions_for_both_focuses(#[values(false, true)] current_focus: bool) {
        let mut app = crate::test_support::comparison_in_context(InputContext::ThemeInfo);
        if current_focus {
            app.update(Action::FieldNext, 14);
        }
        let mut config = Config::default();
        for (key, action) in &mut config.keybindings.bindings {
            if *action == Action::OpenPreset {
                *key = KeyChord::new(KeyCode::F(7), KeyModifiers::NONE);
            }
        }
        let details = detail_lines(&app, &config, app.context(), 116)
            .iter()
            .map(Line::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        for description in [
            "Edit the theme search query",
            "Edit name, appearance, marks and color filters",
            "Choose a target and format to copy",
            "Save the current exploration condition with a name",
            "Open saved filters to apply or manage them",
            "F7",
            "Open the preview selector",
            "Open color details for both themes",
            "Back to Explore",
            "Switch between side-by-side and stacked views",
        ] {
            assert!(details.contains(description), "{details}");
        }
    }
    #[rstest]
    fn full_help_keeps_descriptions_aligned_when_they_wrap(
        #[values(56, 76, 116)] width: usize,
        #[values(
            InputContext::Browse,
            InputContext::Colors,
            InputContext::ThemeInfo,
            InputContext::Saved
        )]
        context: InputContext,
    ) {
        let app = crate::test_support::comparison_in_context(context);
        let lines = detail_lines(&app, &Config::default(), context, width);
        let mut entries = 0;
        for line in lines.iter().filter(|line| line.spans.len() == 2) {
            assert!(line.width() <= width);
            let key = &line.spans[0];
            let description = &line.spans[1].content;
            let padding = description.len() - description.trim_start().len();
            assert_eq!(key.width() + padding, 18);
            assert!(!key.content.contains(" / "));
            entries += 1;
        }
        assert!(entries > 10);
    }

    #[rstest]
    #[case(InputContext::Query)]
    #[case(InputContext::FilterName)]
    #[case(InputContext::DialogName)]
    fn text_help_explains_editing_and_uses_remapped_keys(#[case] context: InputContext) {
        let app = crate::test_support::comparison_in_context(context);
        let mut config = Config::default();
        for (key, action) in &mut config.keybindings.editing {
            if *action == Action::CursorWordPrevious {
                *key = KeyChord::new(KeyCode::F(8), KeyModifiers::NONE);
            }
            if *action == Action::ToggleHelp {
                *key = KeyChord::new(KeyCode::F(12), KeyModifiers::NONE);
            }
        }
        let details = detail_lines(&app, &config, context, 116)
            .iter()
            .map(Line::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        for expected in [
            "Move the cursor left one character",
            "Move the cursor right one character",
            "F8",
            "Move the cursor to the previous word",
            "Move the cursor to the next word",
            "Move the cursor to the start of the input",
            "Move the cursor to the end of the input",
            "Delete the character before the cursor",
            "Delete the character after the cursor",
            "Delete the word before the cursor",
            "Delete the word after the cursor",
            "Clear the entire input",
        ] {
            assert!(details.contains(expected), "missing {expected}: {details}");
        }
        let short = footer(&app, 58, &config.keybindings, config.ui_theme);
        assert_eq!(short.width(), 58);
        assert!(short.to_string().ends_with("F12 Help"));
        assert!(!short.to_string().contains("F8"));
    }

    #[rstest]
    #[case(InputContext::Browse)]
    #[case(InputContext::Colors)]
    #[case(InputContext::FilterChoice)]
    #[case(InputContext::FilterName)]
    #[case(InputContext::Query)]
    #[case(InputContext::Saved)]
    #[case(InputContext::DialogName)]
    #[case(InputContext::DialogConfirm)]
    #[case(InputContext::Preset)]
    #[case(InputContext::Copy)]
    #[case(InputContext::ThemeInfo)]
    #[case(InputContext::Help)]
    fn footer_fits_terminal_columns_and_keeps_the_local_exit(#[case] context: InputContext) {
        let mut app = if context == InputContext::DialogConfirm {
            crate::test_support::comparison_in_context(context)
        } else {
            App::default()
        };
        match context {
            InputContext::Colors => app.update(Action::ToggleColors, 16),
            InputContext::FilterChoice => app.update(Action::EditFilters, 16),
            InputContext::FilterName => {
                app.update(Action::EditFilters, 16);
                app.update(Action::FieldPrevious, 16);
            }
            InputContext::DialogName => app.update(Action::SaveFilter, 16),
            InputContext::Query => app.update(Action::EditQuery, 16),
            InputContext::Saved => {
                app.update(Action::OpenSaved, 16);
            }
            InputContext::Preset => app.update(Action::OpenPreset, 16),
            InputContext::Copy => app.update(Action::OpenCopy, 16),
            InputContext::ThemeInfo => app.update(Action::OpenThemeInfo, 16),
            InputContext::Help => app.update(Action::ToggleHelp, 16),
            _ => {}
        }
        assert_eq!(app.context(), context);
        let mut config = Config::default();
        let next = config
            .keybindings
            .bindings
            .iter_mut()
            .find(|(_, action)| *action == Action::Next)
            .unwrap();
        next.0 = KeyChord::new(KeyCode::Char('界'), KeyModifiers::NONE);
        let quit = if matches!(context, InputContext::Browse | InputContext::Colors) {
            "q Quit"
        } else {
            "C-c Quit"
        };
        let right = if context == InputContext::Help {
            "Esc/F1 Close"
        } else {
            "F1 Help"
        };
        for width in 0..=120 {
            let line = footer(&app, width, &config.keybindings, config.ui_theme);
            assert!(line.width() <= width as usize);
            if width >= right.len() as u16 {
                assert_eq!(line.width(), width as usize);
                assert!(line.to_string().ends_with(right), "{line}");
            }
            if width as usize >= right.len() + 2 + quit.len() {
                assert!(line.to_string().contains(quit), "{line}");
            }
        }
        let minimum = footer(&app, 58, &config.keybindings, config.ui_theme).to_string();
        let local_exit = match context {
            InputContext::Colors | InputContext::ThemeInfo => Some("Esc Close"),
            InputContext::Query | InputContext::FilterName | InputContext::FilterChoice => {
                Some("Enter Apply  Esc Cancel")
            }
            InputContext::Saved => Some("Enter Apply  Esc Back"),
            _ => None,
        };
        if let Some(local_exit) = local_exit {
            assert!(
                minimum
                    .trim_end_matches(right)
                    .trim_end()
                    .ends_with(&format!("{local_exit}  {quit}")),
                "{minimum}"
            );
        }
        let narrow = footer(&app, 78, &config.keybindings, config.ui_theme).to_string();
        assert_eq!(narrow.contains("Help"), context != InputContext::Help);
        if context == InputContext::Browse {
            assert!(narrow.contains("界/k Move"));
            assert!(!narrow.contains("Cycle"));
            assert!(!narrow.contains("Ignore"));
            config
                .keybindings
                .bindings
                .retain(|(_, action)| *action != Action::ToggleCompare);
            let text = detail_lines(&app, &config, context, 76)
                .iter()
                .map(|line| line.to_string())
                .collect::<String>();
            assert!(!text.contains("Compare"));
        }
    }
}
