use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    text::Line,
};

use crate::{
    app::{App, InputContext},
    ui::preview,
};

pub const MIN_WIDTH: u16 = 60;
pub const MIN_HEIGHT: u16 = 22;
pub const SCREEN_MARGIN: Margin = Margin::new(1, 0);
// Preserve pane choices at the same terminal sizes while drawing into the larger area.
const PLACEMENT_INSET: Margin = Margin::new(1, 1);

const FILTERS_CONTROL_HEIGHT: u16 = 10;
pub const HEADER_LABEL_WIDTH: u16 = 9;

pub fn header_info(area: Rect, app: &App) -> Rect {
    let pin_width = if app.mode.is_compare() {
        0
    } else {
        app.pinned_theme()
            .map_or(0, |pin| Line::raw(pin.name).width())
    };
    let width = (usize::from(HEADER_LABEL_WIDTH) + pin_width)
        .min(usize::from(area.width / 3))
        .max(usize::from(HEADER_LABEL_WIDTH) + "Palette Medium".len())
        .min(usize::from(area.width)) as u16;
    Rect::new(area.right() - width, area.y, width, 1)
}

pub struct ScreenLayout {
    pub header: Rect,
    pub query: Rect,
    pub divider: Rect,
    pub main: Rect,
    pub footer: Rect,
}

pub fn comparison(area: Rect, direction: Direction) -> [Rect; 2] {
    let gap = if direction == Direction::Horizontal {
        2
    } else {
        1
    };
    let length = match direction {
        Direction::Horizontal => area.width,
        Direction::Vertical => area.height,
    }
    .saturating_sub(gap)
        / 2;
    let [pin, _, current_area, _] = Layout::default()
        .direction(direction)
        .constraints([
            Constraint::Length(length),
            Constraint::Length(gap),
            Constraint::Length(length),
            Constraint::Min(0),
        ])
        .areas(area);
    [pin, current_area]
}

pub struct SavedLayout {
    pub list: Rect,
    pub details: Rect,
}

pub struct ColorDetailsLayout {
    pub themes: Option<Rect>,
    pub preview: Option<Rect>,
    pub area: Rect,
    pub rows: usize,
    pub detail_rows: u16,
}

pub fn comparison_columns(area: Rect) -> [Rect; 4] {
    let area = area.inner(Margin::new(1, 0));
    let values = area.width - 14 - 7 - 6;
    let a_width = values / 2;
    let b_width = values - a_width;
    let a_x = area.x + 16;
    let b_x = a_x + a_width + 2;
    [
        Rect::new(area.x, area.y, 14, 1),
        Rect::new(a_x, area.y, a_width, 1),
        Rect::new(b_x, area.y, b_width, 1),
        Rect::new(b_x + b_width + 2, area.y, 7, 1),
    ]
}

pub fn detail_columns(area: Rect, compare: bool) -> [Rect; 4] {
    if compare {
        return comparison_columns(area);
    }
    [
        Rect::new(area.x + 1, area.y, 14, 1),
        Rect::new(area.x + 17, area.y, area.width.saturating_sub(19), 1),
        Rect::default(),
        Rect::default(),
    ]
}

impl ColorDetailsLayout {
    pub const HEADER_ROWS: u16 = 3;

    fn new(main: Rect, app: &App) -> Self {
        let placement = main.inner(PLACEMENT_INSET);
        let [_, pin, current_area, _] = detail_columns(placement, app.mode.is_compare());
        // A ratio up to 21.00:1 followed by " on #ffffff" needs 18 columns.
        let value_width = if app.mode.is_compare() {
            pin.width.min(current_area.width)
        } else {
            pin.width
        };
        let detail_rows = if value_width >= 18 { 2 } else { 3 };
        let fixed_rows = Self::HEADER_ROWS + 1 + detail_rows;
        if !app.details_maximized {
            for rows in (4..=7).rev() {
                let height = fixed_rows + rows;
                let top = Rect::new(
                    main.x,
                    main.y,
                    main.width,
                    main.height.saturating_sub(height + 1),
                );
                let layout = theme_preview(top, placement.width, app);
                if preview_fits(app, layout.preview) {
                    return Self {
                        themes: layout.themes,
                        preview: Some(layout.preview),
                        area: Rect::new(main.x, main.bottom() - height, main.width, height),
                        rows: rows as usize,
                        detail_rows,
                    };
                }
            }
        }
        Self {
            themes: None,
            preview: None,
            area: main,
            rows: main.height.saturating_sub(fixed_rows).min(20) as usize,
            detail_rows,
        }
    }
}

pub struct ThemePreviewLayout {
    pub themes: Option<Rect>,
    pub preview: Rect,
}

pub struct ThemeInfoLayout {
    pub themes: Option<Rect>,
    pub preview: Option<Rect>,
    pub area: Rect,
    pub rows: usize,
}

impl ThemeInfoLayout {
    pub const HEADER_ROWS: u16 = 3;

    fn new(main: Rect, app: &App) -> Self {
        let placement = main.inner(PLACEMENT_INSET);
        let height = 13;
        let top = Rect::new(
            main.x,
            main.y,
            main.width,
            main.height.saturating_sub(height + 1),
        );
        let layout = theme_preview(top, placement.width, app);
        if !app.details_maximized && preview_fits(app, layout.preview) {
            Self {
                themes: layout.themes,
                preview: Some(layout.preview),
                area: Rect::new(main.x, main.bottom() - height, main.width, height),
                rows: usize::from(height - Self::HEADER_ROWS),
            }
        } else {
            Self {
                themes: None,
                preview: None,
                area: main,
                rows: usize::from(main.height.saturating_sub(Self::HEADER_ROWS)),
            }
        }
    }
}

fn theme_preview(area: Rect, placement_width: u16, app: &App) -> ThemePreviewLayout {
    if !app.theme_list_visible() {
        return ThemePreviewLayout {
            themes: None,
            preview: area,
        };
    }
    let width = if app.mode.is_compare() && placement_width < 104 {
        20
    } else {
        28
    };
    let [themes, _, preview] = Layout::horizontal([
        Constraint::Length(width),
        Constraint::Length(2),
        Constraint::Min(0),
    ])
    .areas(area);
    ThemePreviewLayout {
        themes: Some(themes),
        preview,
    }
}

pub struct ModalLayout {
    pub outer: Rect,
    pub title: Rect,
    pub content: Rect,
    pub actions: Rect,
}

pub fn modal(main: Rect, width: u16, body_rows: u16) -> ModalLayout {
    let width = width.min(main.width);
    let height = (body_rows + 6).min(main.height);
    let x = main.x + (main.width - width) / 2;
    let y = main.y + (main.height - height) / 2;
    let outer = Rect::new(x, y, width, height);
    let inner = outer.inner(Margin::new(2, 1));
    ModalLayout {
        outer,
        title: Rect::new(inner.x, inner.y, inner.width, 1),
        content: Rect::new(inner.x, inner.y + 2, inner.width, height.saturating_sub(6)),
        actions: Rect::new(inner.x, outer.bottom().saturating_sub(2), inner.width, 1),
    }
}

pub struct EditorLayout {
    pub controls: Rect,
    pub results: Rect,
    pub preview: Option<Rect>,
}

impl EditorLayout {
    fn search(area: Rect, app: &App) -> Self {
        let [controls, _, body] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(area);
        let [results, _, preview] = Layout::horizontal([
            Constraint::Length(28),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .areas(body);
        if preview_fits(app, preview) {
            Self {
                controls,
                results,
                preview: Some(preview),
            }
        } else {
            Self {
                controls,
                results: body,
                preview: None,
            }
        }
    }

    fn filters(area: Rect, placement_width: u16, preview: Option<Rect>) -> Self {
        let (controls, results) = if placement_width >= 56 {
            let result_width = if placement_width >= 64 { 28 } else { 20 };
            let [results, _, controls] = Layout::horizontal([
                Constraint::Length(result_width),
                Constraint::Length(2),
                Constraint::Min(0),
            ])
            .areas(area);
            (controls, results)
        } else {
            let [controls, _, results] = Layout::vertical([
                Constraint::Length(FILTERS_CONTROL_HEIGHT),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .areas(area);
            (controls, results)
        };
        Self {
            controls,
            results,
            preview,
        }
    }

    pub fn visible_rows(&self) -> usize {
        self.results.height.saturating_sub(2) as usize
    }
}

impl ScreenLayout {
    pub fn theme_preview(&self, app: &App) -> ThemePreviewLayout {
        theme_preview(self.main, self.main.inner(PLACEMENT_INSET).width, app)
    }

    pub fn preview(&self, app: &App) -> Option<Rect> {
        if app.saved.is_some() {
            None
        } else if let Some(editor) = self.editor(app) {
            editor.preview
        } else if let Some(info) = self.info(app) {
            info.preview
        } else if let Some(colors) = self.colors(app) {
            colors.preview
        } else {
            Some(self.theme_preview(app).preview)
        }
    }

    pub fn colors(&self, app: &App) -> Option<ColorDetailsLayout> {
        (app.theme_info.is_none()
            && app.color_details.is_open()
            && app.editor.is_none()
            && app.saved.is_none())
        .then(|| ColorDetailsLayout::new(self.main, app))
    }

    pub fn info(&self, app: &App) -> Option<ThemeInfoLayout> {
        (app.theme_info.is_some() && app.editor.is_none() && app.saved.is_none())
            .then(|| ThemeInfoLayout::new(self.main, app))
    }

    pub fn visible_rows(&self) -> usize {
        self.main.height.saturating_sub(2) as usize
    }

    pub fn saved(&self) -> SavedLayout {
        let [themes, _, preview] = Layout::horizontal([
            Constraint::Length(if self.main.inner(PLACEMENT_INSET).width < 76 {
                20
            } else {
                28
            }),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .areas(self.main);
        SavedLayout {
            list: themes,
            details: preview,
        }
    }

    pub fn editor(&self, app: &App) -> Option<EditorLayout> {
        app.editor.as_ref()?;
        if app.filters().is_none() {
            return Some(EditorLayout::search(self.main, app));
        }
        let placement = self.main.inner(PLACEMENT_INSET);
        let horizontal_comparison =
            app.mode.is_compare() && app.compare_direction == Direction::Horizontal;
        let choices = filter_areas(placement, horizontal_comparison);
        let areas = filter_areas(self.main, horizontal_comparison);
        for ((editor, preview), (draw_editor, draw_preview)) in choices.into_iter().zip(areas) {
            if editor.width > placement.width || editor.height > placement.height {
                continue;
            }
            let layout = EditorLayout::filters(editor, editor.width, Some(preview));
            let controls_fit = layout.controls.height >= FILTERS_CONTROL_HEIGHT;
            if layout.visible_rows() >= 4 && controls_fit && preview_fits(app, draw_preview) {
                return Some(EditorLayout::filters(
                    draw_editor,
                    editor.width,
                    Some(draw_preview),
                ));
            }
        }
        Some(EditorLayout::filters(self.main, placement.width, None))
    }

    pub fn input_rows(&self, app: &App) -> usize {
        if app.context() == InputContext::ThemeInfo {
            return self.info(app).map_or(0, |layout| layout.rows);
        }
        if app.context() == InputContext::Colors {
            return self.colors(app).map_or(0, |layout| layout.rows);
        }
        self.result_rows(app)
    }

    pub fn result_rows(&self, app: &App) -> usize {
        if app.saved.is_some() {
            return self.visible_rows();
        }
        if let Some(editor) = self.editor(app) {
            return editor.visible_rows();
        }
        self.info(app)
            .map(|info| info.themes)
            .unwrap_or_else(|| self.colors(app).and_then(|colors| colors.themes))
            .map_or_else(
                || self.visible_rows(),
                |themes| themes.height.saturating_sub(2) as usize,
            )
    }
}

fn filter_areas(main: Rect, horizontal_comparison: bool) -> [(Rect, Rect); 2] {
    let width = 36;
    let height = FILTERS_CONTROL_HEIGHT;
    let left = (
        Rect::new(main.x, main.y, width, main.height),
        Rect::new(
            main.x + width + 2,
            main.y,
            main.width.saturating_sub(width + 2),
            main.height,
        ),
    );
    let bottom = (
        Rect::new(
            main.x,
            main.bottom().saturating_sub(height),
            main.width,
            height,
        ),
        Rect::new(
            main.x,
            main.y,
            main.width,
            main.height.saturating_sub(height + 1),
        ),
    );
    if horizontal_comparison {
        [bottom, left]
    } else {
        [left, bottom]
    }
}

fn preview_fits(app: &App, area: Rect) -> bool {
    let fits = |pane: Rect| {
        let content = preview::content_area(pane);
        content.width >= 13 && content.height >= 3
    };
    if app.mode.is_compare() {
        comparison(area, app.compare_direction)
            .into_iter()
            .all(fits)
    } else {
        fits(area)
    }
}

pub fn screen(area: Rect) -> Option<ScreenLayout> {
    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        return None;
    }
    let inner = area.inner(SCREEN_MARGIN);
    let [header, query, divider, _, main, _, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    Some(ScreenLayout {
        header,
        query,
        divider,
        main,
        footer,
    })
}
