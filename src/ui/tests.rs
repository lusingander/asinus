use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    layout::{Direction, Rect, Size},
    style::{Color, Modifier},
    text::Line,
};
use rstest::rstest;

use super::*;
use crate::{
    app::{Action, InputContext},
    marks::{Mark, MarkChange},
    preview::Preset,
    test_support::{custom_navigation_config, insert_text, render},
    ui::widgets::panel,
    workspace::WorkspaceChange,
};

fn row(b: &Buffer, y: u16) -> String {
    (0..b.area.width).map(|x| b[(x, y)].symbol()).collect()
}

fn text(b: &Buffer) -> String {
    (0..b.area.height)
        .map(|y| row(b, y))
        .collect::<Vec<_>>()
        .join("\n")
}

fn modal_rect(buffer: &Buffer) -> Rect {
    let top = (0..buffer.area.height)
        .find(|&y| row(buffer, y).contains('┌'))
        .unwrap();
    let left = (0..buffer.area.width)
        .find(|&x| buffer[(x, top)].symbol() == "┌")
        .unwrap();
    let right = (left..buffer.area.width)
        .find(|&x| buffer[(x, top)].symbol() == "┐")
        .unwrap();
    let bottom = (top..buffer.area.height)
        .find(|&y| buffer[(left, y)].symbol() == "└")
        .unwrap();
    Rect::new(left, top, right - left + 1, bottom - top + 1)
}

fn query(app: &mut App, value: &str) {
    app.update(Action::EditQuery, 14);
    app.update(Action::ClearInput, 14);
    insert_text(app, value);
    app.update(Action::Confirm, 14);
}

fn selected_color_row(buffer: &Buffer, role: &str, context: &str) -> u16 {
    (0..buffer.area.height - 1)
        .find(|&y| {
            row(buffer, y).trim_start().split("  ").next() == Some(role)
                && row(buffer, y + 1).trim_start().split("  ").next() == Some(context)
        })
        .unwrap()
}

#[rstest]
#[case(8, 0, "┃┃  ")]
#[case(8, 2, " ┃┃ ")]
#[case(8, 4, "  ┃┃")]
#[case(8, 7, "   ┃")]
#[case(400, 396, "   ┃")]
#[case(4, 0, "    ")]
#[case(3, 0, "    ")]
#[case(0, 0, "    ")]
fn scrollbar_shows_the_visible_fraction_in_the_gutter_with_reset_style(
    #[case] length: usize,
    #[case] offset: usize,
    #[case] expected: &str,
) {
    let mut terminal = Terminal::new(TestBackend::new(12, 7)).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(
                Block::default().style(
                    Style::default()
                        .fg(Color::Red)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD | Modifier::REVERSED),
                ),
                frame.area(),
            );
            draw_scrollbar(
                frame,
                Rect::new(1, 1, 8, 4),
                length,
                offset,
                UiTheme::default(),
            );
        })
        .unwrap();
    let buffer = terminal.backend().buffer();
    let gutter: String = (1..5).map(|y| buffer[(9, y)].symbol()).collect();
    assert_eq!(gutter, expected);
    for y in 0..7 {
        for x in 0..12 {
            let cell = &buffer[(x, y)];
            if cell.symbol() == "┃" {
                assert_eq!(cell.fg, Color::Reset);
                assert_eq!(cell.bg, Color::Reset);
                assert!(cell.modifier.is_empty());
            } else {
                assert_eq!(cell.fg, Color::Red);
                assert_eq!(cell.bg, Color::Blue);
                assert_eq!(cell.modifier, Modifier::BOLD | Modifier::REVERSED);
            }
        }
    }
}

#[rstest]
#[case(80, 24, InputContext::Browse, false)]
#[case(80, 24, InputContext::Browse, true)]
#[case(60, 22, InputContext::Query, false)]
#[case(120, 36, InputContext::Query, false)]
#[case(60, 22, InputContext::FilterChoice, false)]
#[case(120, 36, InputContext::FilterChoice, true)]
fn result_scrollbars_follow_the_viewport_without_changing_the_selection(
    #[case] width: u16,
    #[case] height: u16,
    #[case] context: InputContext,
    #[case] compare: bool,
) {
    let mut app = App::default();
    if compare {
        for action in [
            Action::TogglePin,
            Action::ToggleCompare,
            Action::ToggleThemeList,
        ] {
            app.update(action, 14);
        }
    }
    match context {
        InputContext::Query => app.update(Action::EditQuery, 14),
        InputContext::FilterChoice => app.update(Action::EditFilters, 14),
        _ => {}
    }
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &config);
    let screen = layout::screen(terminal.backend().buffer().area).unwrap();
    let results = screen.editor(&app).map_or_else(
        || screen.theme_preview(&app).themes.unwrap(),
        |editor| editor.results,
    );
    let content = panel("", config.ui_theme).inner(results);
    let bar = |buffer: &Buffer| -> String {
        (0..height)
            .map(|y| buffer[(content.right(), y)].symbol())
            .collect()
    };
    assert_eq!(
        terminal.backend().buffer()[(content.right(), content.y)].symbol(),
        "┃"
    );
    let initial = bar(terminal.backend().buffer());
    let selection = app.selected();
    if context != InputContext::FilterChoice {
        app.update(Action::Next, content.height as usize);
        render(&mut terminal, &mut app, &config);
        assert_eq!(bar(terminal.backend().buffer()), initial);
    }
    for _ in 0..100 {
        app.update(Action::PageNext, content.height as usize);
    }
    render(&mut terminal, &mut app, &config);
    assert_ne!(bar(terminal.backend().buffer()), initial);
    assert_eq!(
        terminal.backend().buffer()[(content.right(), content.bottom() - 1)].symbol(),
        "┃"
    );
    if context == InputContext::FilterChoice {
        assert_eq!(app.selected(), selection);
    }
}

#[rstest]
#[case(60, 22)]
#[case(80, 24)]
#[case(120, 36)]
fn help_covers_the_color_scrollbar_and_resize_hides_a_fully_visible_document(
    #[case] width: u16,
    #[case] height: u16,
) {
    let mut app = crate::test_support::comparison_in_context(InputContext::Colors);
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &config);
    let screen = layout::screen(terminal.backend().buffer().area).unwrap();
    let colors = screen.colors(&app).unwrap();
    app.update(Action::Last, colors.rows);
    render(&mut terminal, &mut app, &config);
    let x = screen.main.right();
    let last_color_row = colors.area.y + 2 + colors.rows as u16;
    assert_eq!(
        terminal.backend().buffer()[(x, last_color_row)].symbol(),
        "┃"
    );

    app.update(Action::ToggleHelp, colors.rows);
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    assert_eq!(buffer[(x, last_color_row)].symbol(), " ");
    assert_eq!(buffer[(x, screen.main.y + 2)].symbol(), "┃");
    app.update(Action::Last, screen.visible_rows());
    render(&mut terminal, &mut app, &config);
    assert_eq!(
        terminal.backend().buffer()[(x, screen.main.bottom() - 1)].symbol(),
        "┃"
    );
    terminal.backend_mut().resize(60, 120);
    terminal.resize(Size::new(60, 120).into()).unwrap();
    render(&mut terminal, &mut app, &config);
    assert!(!text(terminal.backend().buffer()).contains("┃"));
    app.update(Action::Cancel, 100);
    app.update(Action::TogglePanelMaximize, 20);
    app.update(Action::First, 20);
    render(&mut terminal, &mut app, &config);
    assert!(!text(terminal.backend().buffer()).contains("┃"));
}

#[rstest]
#[case("Blue", 6)]
#[case("Bright Blue", 14)]
fn selected_normal_and_bright_roles_survive_width_changes(
    #[case] role: &str,
    #[case] steps: usize,
) {
    let mut app = crate::test_support::comparison_in_context(InputContext::Colors);
    for _ in 0..steps {
        app.update(Action::Next, 6);
    }
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    for (width, height) in [(80, 24), (68, 24), (69, 24), (60, 22), (80, 24)] {
        terminal.backend_mut().resize(width, height);
        terminal.resize(Size::new(width, height).into()).unwrap();
        render(&mut terminal, &mut app, &config);
        let buffer = terminal.backend().buffer();
        assert_eq!(app.color_details.role().label(), role);
        selected_color_row(buffer, role, "vs own BG");
        let colors = layout::screen(buffer.area).unwrap().colors(&app).unwrap();
        let selected_y = colors.area.y + 3 + steps as u16 - app.color_details.list.offset() as u16;
        let label = row(buffer, selected_y);
        assert_eq!(label.trim_start().split("  ").next(), Some(role));
        assert!(
            buffer[(colors.area.x, selected_y)]
                .modifier
                .contains(Modifier::REVERSED)
        );
        assert_eq!(text(buffer).contains("Aa 0123 sample.txt"), width >= 69);
    }
}

#[rstest]
#[case(60, 22, Direction::Horizontal, true)]
#[case(60, 24, Direction::Horizontal, true)]
#[case(60, 25, Direction::Horizontal, false)]
#[case(80, 23, Direction::Horizontal, true)]
#[case(80, 24, Direction::Horizontal, false)]
#[case(120, 36, Direction::Horizontal, false)]
#[case(60, 32, Direction::Vertical, true)]
#[case(60, 33, Direction::Vertical, false)]
#[case(80, 31, Direction::Vertical, true)]
#[case(80, 32, Direction::Vertical, false)]
fn colors_keep_the_selected_role_and_only_show_previews_with_usable_content(
    #[case] width: u16,
    #[case] height: u16,
    #[case] direction: Direction,
    #[case] hidden: bool,
    #[values(false, true)] list_visible: bool,
) {
    let mut app = crate::test_support::comparison_in_context(InputContext::Colors);
    app.compare_direction = direction;
    if list_visible {
        app.update(Action::ToggleThemeList, 4);
    }
    app.update(Action::Last, 4);
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let content = text(buffer);
    let colors = layout::screen(buffer.area).unwrap().colors(&app).unwrap();
    assert_eq!(colors.preview.is_none(), hidden);
    selected_color_row(buffer, "Selection", "Stored value");
    if width >= 68 {
        assert!(content.contains("Pin: 3024 Day"));
        assert!(content.contains("Current: 3024 Night"));
    }
    let summary = row(buffer, colors.area.y);
    assert!(summary.contains("D4") || summary.contains("Diff 4"));
    assert!(
        buffer
            .content
            .iter()
            .any(|cell| cell.symbol() == "S" && cell.modifier.contains(Modifier::REVERSED))
    );
    let selected = app.color_details.list.selected();
    app.update(Action::FieldNext, 4);
    render(&mut terminal, &mut app, &config);
    assert_eq!(app.color_details.list.selected(), selected);
    assert!(!app.color_details.is_focused());
}

#[rstest]
#[case(60, 22, "Aa 0123")]
#[case(68, 24, "Aa 0123")]
#[case(69, 24, "Aa 0123 sample.txt")]
#[case(80, 24, "Aa 0123 sample.txt")]
#[case(120, 36, "Aa 0123 sample.txt")]
fn color_swatches_ignore_selection_and_samples_align_with_their_own_ratios(
    #[case] width: u16,
    #[case] height: u16,
    #[case] sample: &str,
) {
    let mut app = crate::test_support::comparison_in_context(InputContext::Colors);
    for _ in 0..10 {
        app.update(Action::Next, 6);
    }
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let layout = layout::screen(buffer.area).unwrap().colors(&app).unwrap();
    let selected_row = layout.area.y + 3 + 10 - app.color_details.list.offset() as u16;
    let chips: Vec<_> = (layout.area.x..layout.area.right())
        .map(|x| &buffer[(x, selected_row)])
        .filter(|cell| cell.bg == Color::Rgb(92, 88, 85))
        .collect();
    assert_eq!(chips.len(), 4);
    assert!(chips.iter().all(|cell| {
        !cell
            .modifier
            .intersects(Modifier::REVERSED | Modifier::BOLD)
    }));
    let hex_x = row(buffer, selected_row).find("#5c5855").unwrap() as u16;
    let hex = &buffer[(hex_x, selected_row)];
    assert_eq!(hex.fg, Color::Reset);
    assert!(hex.modifier.contains(Modifier::REVERSED));
    let y = selected_color_row(buffer, "Bright Black", "vs own BG");
    let headers = row(buffer, layout.area.y + 2);
    let a_x = headers.find("Pin").unwrap() as u16;
    let b_x = headers.find("Current").unwrap() as u16;
    let rgb_x = headers.find("RGB").unwrap() as u16;
    for (x, end, background, ratio, bg_hex) in [
        (a_x, b_x - 2, Color::Rgb(247, 247, 247), "6.57:1", "#f7f7f7"),
        (b_x, rgb_x - 2, Color::Rgb(9, 3, 0), "2.91:1", "#090300"),
    ] {
        let cell = &buffer[(x, y)];
        assert_eq!(cell.symbol(), "A");
        assert_eq!(cell.fg, Color::Rgb(92, 88, 85));
        assert_eq!(cell.bg, background);
        assert!(
            !cell
                .modifier
                .intersects(Modifier::BOLD | Modifier::DIM | Modifier::REVERSED)
        );
        assert_eq!(buffer[(end - 1, y)].bg, background);
        let cell_text = |y| {
            (x..end)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        };
        assert_eq!(cell_text(y).trim_end(), sample);
        assert_eq!(buffer[(x, y + 1)].fg, Color::Reset);
        assert!(
            !buffer[(x, y + 1)]
                .modifier
                .intersects(Modifier::BOLD | Modifier::DIM | Modifier::REVERSED)
        );
        if width >= 69 {
            assert_eq!(cell_text(y + 1).trim_end(), format!("{ratio} on {bg_hex}"));
        } else {
            assert_eq!(cell_text(y + 1).trim_end(), ratio);
            assert_eq!(cell_text(y + 2).trim_end(), bg_hex);
        }
    }
}

#[test]
fn color_summary_keeps_all_counts_and_styles_only_the_diff_label() {
    let mut app = crate::test_support::comparison_in_context(InputContext::Colors);
    app.update(Action::Last, 3);
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(60, 22)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let area = layout::screen(buffer.area)
        .unwrap()
        .colors(&app)
        .unwrap()
        .area;
    let header: String = row(buffer, area.y)
        .chars()
        .skip(area.x as usize)
        .take(area.width as usize)
        .collect();
    assert!(header.starts_with("Color details"), "{header}");
    assert!(header.ends_with("RGB Same 16 Diff 4 Missing 0"), "{header}");
    let difference = (area.x..area.right())
        .find(|&x| buffer[(x, area.y)].symbol() == "D")
        .unwrap();
    assert_eq!(buffer[(difference, area.y)].fg, Color::Yellow);
    assert_eq!(buffer[(difference + 4, area.y)].fg, Color::Reset);
}

#[rstest]
#[case(60, "Pin", "Current")]
#[case(67, "Pin", "Current")]
#[case(68, "Pin: LiquidCarbo..", "Current: LiquidCa..")]
#[case(69, "Pin: LiquidCarbon..", "Current: LiquidCa..")]
fn narrow_color_details_fit_headings_and_show_ratios_and_background_values(
    #[case] width: u16,
    #[case] pin_title: &str,
    #[case] current_title: &str,
) {
    let mut app = App::default();
    query(&mut app, "LiquidCarbonTransparentInverse");
    for action in [
        Action::TogglePin,
        Action::ToggleCompare,
        Action::ToggleColors,
    ] {
        app.update(action, 6);
    }
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(width, 22)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let layout = layout::screen(buffer.area).unwrap().colors(&app).unwrap();
    let [_, pin, current, _] = layout::comparison_columns(layout.area);
    let headings = row(buffer, layout.area.y + 2);
    for (cell, title) in [(pin, pin_title), (current, current_title)] {
        assert_eq!(
            headings[usize::from(cell.x)..usize::from(cell.right())].trim_end(),
            title,
        );
    }
    assert!(
        row(buffer, layout.area.y + 3)
            .trim_start()
            .starts_with("Foreground")
    );
    let y = selected_color_row(buffer, "Foreground", "vs own BG");
    assert_eq!(row(buffer, y + 1).matches("11.32:1").count(), 2);
    if width < 69 {
        assert_eq!(row(buffer, y + 2).matches("#000000").count(), 2);
        assert_eq!(layout.rows, 9);
        assert_eq!(y + 2, layout.area.bottom() - 1);
    } else {
        assert_eq!(row(buffer, y + 1).matches("11.32:1 on #000000").count(), 2);
        assert_eq!(layout.rows, 10);
        assert_eq!(y + 1, layout.area.bottom() - 1);
    }
}

#[test]
fn color_comparison_labels_yield_to_selection_and_restore_their_color_on_focus_change() {
    let mut app = crate::test_support::comparison_in_context(InputContext::Colors);
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(60, 22)).unwrap();
    render(&mut terminal, &mut app, &config);
    let y = layout::screen(terminal.backend().buffer().area)
        .unwrap()
        .colors(&app)
        .unwrap()
        .area
        .y
        + 3;
    let x = (0..60)
        .find(|&x| terminal.backend().buffer()[(x, y)].symbol() == "D")
        .unwrap();
    let selected = &terminal.backend().buffer()[(x, y)];
    assert_eq!(selected.fg, Color::Reset);
    assert!(selected.modifier.contains(Modifier::REVERSED));
    let background_row = &terminal.backend().buffer()[(x, y + 1)];
    assert_eq!(background_row.fg, Color::Yellow);
    assert!(!background_row.modifier.contains(Modifier::REVERSED));
    app.update(Action::FieldNext, 6);
    render(&mut terminal, &mut app, &config);
    let unfocused = &terminal.backend().buffer()[(x, y)];
    assert_eq!(unfocused.fg, Color::Yellow);
    assert!(!unfocused.modifier.contains(Modifier::REVERSED));
}

#[rstest]
#[case("3024 Day", Some("3024 Night"), 1, "", "RGB Same 16 Diff 4 Missing 0")]
#[case(
    "DraculaPlus",
    Some("DraculaPlus"),
    18,
    "Not specified",
    "RGB Same 18 Diff 0 Missing 2"
)]
#[case(
    "DraculaPlus",
    Some("DraculaPlus"),
    19,
    "Not specified",
    "RGB Same 18 Diff 0 Missing 2"
)]
#[case(
    "DraculaPlus",
    Some("Dracula"),
    19,
    "Not specified",
    "RGB Same 4 Diff 14 Missing 2"
)]
#[case("3024 Day", None, 19, "", "No theme selected")]
fn raw_and_missing_colors_keep_registered_values_distinct_from_an_unselected_theme(
    #[case] pin: &str,
    #[case] current_theme: Option<&str>,
    #[case] role: usize,
    #[case] sample: &str,
    #[case] summary: &str,
) {
    let mut app = App::default();
    query(&mut app, pin);
    app.update(Action::TogglePin, 6);
    query(&mut app, current_theme.unwrap_or("missing-theme-e1"));
    app.update(Action::First, 6);
    app.update(Action::ToggleCompare, 6);
    app.update(Action::ToggleColors, 6);
    for _ in 0..role {
        app.update(Action::Next, 6);
    }
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(60, 22)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let layout = layout::screen(buffer.area).unwrap().colors(&app).unwrap();
    assert!(row(buffer, layout.area.y).contains(summary));
    let y = selected_color_row(buffer, &app.color_details.role().label(), "Stored value");
    let samples = row(buffer, y);
    assert!(samples.contains(sample));
    assert!(!samples.contains("Aa 0123"));
    let values = row(buffer, y + 1);
    assert!(!values.contains(" on "));
    assert!(!values.contains(":1"));
    if current_theme.is_none() {
        let current = layout::comparison_columns(layout.area)[2];
        let current = usize::from(current.x)..usize::from(current.right());
        assert!(samples[current.clone()].trim().is_empty());
        assert_eq!(values[current].trim(), "-");
    } else if sample.is_empty() {
        assert!(values.contains("#f7f7f7"));
        assert!(values.contains("#090300"));
    } else if sample == "Not specified" {
        assert!(text(buffer).contains("Missing"));
        let headers = row(buffer, layout.area.y + 2);
        let a_x = headers.find("Pin").unwrap();
        let b_x = headers.find("Current").unwrap();
        assert!(values[a_x..b_x].trim().is_empty());
        if current_theme == Some(pin) {
            assert!(values[b_x..].trim().is_empty());
        }
    }
}

#[test]
fn aliased_themes_show_equal_colors_and_their_contrast() {
    let mut app = App::default();
    query(&mut app, "Catppuccin Mocha");
    app.update(Action::TogglePin, 6);
    query(&mut app, "catppuccin-mocha");
    for action in [Action::ToggleCompare, Action::ToggleColors] {
        app.update(action, 6);
    }
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &Config::default());
    let buffer = terminal.backend().buffer();
    assert!(text(buffer).contains("RGB Same 20 Diff 0 Missing 0"));
    let y = selected_color_row(buffer, "Foreground", "vs own BG");
    assert_eq!(row(buffer, y + 1).matches("11.34:1").count(), 2);
}

#[test]
fn saved_screen_preserves_the_hidden_theme_list_until_returning() {
    let mut app = App::default();
    let config = Config::default();
    app.update(Action::Last, 14);
    let offset = app.theme_list.offset();
    let first = app.themes().nth(offset).unwrap().name.to_owned();
    app.update(Action::OpenSaved, 14);

    let mut terminal = Terminal::new(TestBackend::new(120, 36)).unwrap();
    prepare(Rect::new(0, 0, 120, 36), &mut app, &config);
    assert_eq!(app.theme_list.offset(), offset);
    terminal.draw(|frame| draw(frame, &app, &config)).unwrap();
    assert!(!text(terminal.backend().buffer()).contains(&first));

    app.update(Action::Cancel, 1);
    prepare(Rect::new(0, 0, 120, 36), &mut app, &config);
    assert!(app.theme_list.offset() < offset);
    let first = app
        .themes()
        .nth(app.theme_list.offset())
        .unwrap()
        .name
        .to_owned();
    terminal.draw(|frame| draw(frame, &app, &config)).unwrap();
    assert!(row(terminal.backend().buffer(), 6).contains(&first));
}

#[rstest]
fn saved_details_keep_selected_and_current_conditions_separate_when_values_are_long(
    #[values(60, 80, 120)] width: u16,
) {
    let mut app = App::default();
    query(
        &mut app,
        &format!(
            "{} type:light brightness:very-light bg:gray contrast:medium",
            "catpp ".repeat(30)
        ),
    );
    app.workspace = app
        .workspace
        .changed(&WorkspaceChange::New {
            name: "夜".repeat(40),
            query: format!("{} type:dark bg:blue contrast:medium", "catpp ".repeat(30)),
        })
        .unwrap();
    app.update(Action::OpenSaved, 12);
    app.update(Action::Next, 12);
    let mut terminal = Terminal::new(TestBackend::new(width, 22)).unwrap();
    render(&mut terminal, &mut app, &Config::default());
    let buffer = terminal.backend().buffer();
    let contents = text(buffer);
    let selected = contents.find("Selected filter").unwrap();
    let selected_count = contents.find("6 matching themes").unwrap();
    let current = contents.find("Current exploration").unwrap();
    let current_count = contents.find("2 matching themes").unwrap();
    assert!(selected < selected_count && selected_count < current && current < current_count);
    assert_eq!(
        contents
            .lines()
            .filter(|line| line.trim_end().ends_with("Condition"))
            .count(),
        2
    );
    assert!(contents.contains("catpp") && contents.contains(".."));
    let footer = layout::screen(buffer.area).unwrap().footer;
    assert!(row(buffer, footer.y).contains("Enter Apply"));
    assert!(row(buffer, footer.y - 1).trim().is_empty());
}

#[test]
fn saved_selection_is_visible_in_the_prepared_viewport() {
    let mut app = App::default();
    let config = Config::default();
    for index in 0..17 {
        app.workspace = app
            .workspace
            .changed(&WorkspaceChange::New {
                name: format!("Filter {index:02}"),
                query: "type:dark".into(),
            })
            .unwrap();
    }
    app.update(Action::OpenSaved, 26);
    app.update(Action::Last, 26);
    prepare(Rect::new(0, 0, 80, 24), &mut app, &config);
    let saved = app.saved.as_ref().unwrap();
    assert_eq!(saved.list.offset(), 2);

    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    terminal.draw(|frame| draw(frame, &app, &config)).unwrap();
    let buffer = terminal.backend().buffer();
    assert!(row(buffer, 6).contains("Filter 01"));
    assert!(row(buffer, 21).contains("Filter 16"));
    assert!(buffer[(1, 21)].modifier.contains(Modifier::REVERSED));
    assert_eq!(buffer[(29, 21)].symbol(), "┃");
    assert_eq!(buffer[(29, 4)].symbol(), " ");

    terminal.backend_mut().resize(120, 36);
    terminal.resize(Size::new(120, 36).into()).unwrap();
    render(&mut terminal, &mut app, &config);
    assert!(!text(terminal.backend().buffer()).contains("┃"));
}

#[test]
fn focus_and_shortcut_keys_use_independent_colors() {
    let mut app = App::default();
    let mut config = Config::default();
    config.ui_theme.focus = Color::Magenta;
    config.ui_theme.key = Color::Green;
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &config);

    let buffer = terminal.backend().buffer();
    let screen = layout::screen(buffer.area).unwrap();
    let themes = screen.theme_preview(&app).themes.unwrap();
    let title = &buffer[(themes.x, themes.y)];
    assert_eq!((title.symbol(), title.fg), ("T", Color::Magenta));
    assert!(title.modifier.contains(Modifier::BOLD));
    let key = &buffer[(screen.footer.x, screen.footer.y)];
    assert_eq!((key.symbol(), key.fg), ("j", Color::Green));
    assert!(key.modifier.is_empty());

    app.update(Action::ToggleHelp, screen.visible_rows());
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let y = (screen.main.y..screen.main.bottom())
        .find(|&y| row(buffer, y).contains("Select the next theme"))
        .unwrap();
    let key = &buffer[(screen.main.x, y)];
    assert_eq!((key.symbol(), key.fg), ("j", Color::Green));
    assert!(key.modifier.contains(Modifier::BOLD));
}

#[test]
fn custom_bindings_appear_in_footer_notice_and_comparison() {
    let mut app = App::default();
    let config = custom_navigation_config();
    for action in [Action::Next, Action::Next, Action::PresetNext] {
        app.update(action, 14);
    }
    let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    assert!(row(buffer, 4).contains("Aardvark Blue"));
    assert!(text(buffer).contains("Preview  Shell"));
    assert_eq!(
        row(buffer, 23).trim(),
        "n/i Move  / Search  f Filters  a Pin  s Compare  [/o Preview  q Quit   F1 Help",
    );

    app.update(Action::ToggleCompare, 14);
    render(&mut terminal, &mut app, &config);
    assert!(text(terminal.backend().buffer()).contains("Pin a theme first (a)"));

    for action in [
        Action::TogglePin,
        Action::ToggleCompare,
        Action::ToggleLayout,
    ] {
        app.update(action, 14);
    }
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    assert_eq!(text(buffer).matches("Aardvark Blue").count(), 2);
    assert!(text(buffer).contains("Compare"));
    assert!(row(buffer, 13).contains("Current: Aardvark Blue"));
    assert!(row(buffer, 23).contains("Esc Explore"));
    let details = help::detail_lines(&app, &config, InputContext::Browse, 76);
    assert!(details.iter().any(|line| line.to_string().starts_with("t ")
        && line.to_string().contains("Switch between side-by-side")));
}

#[rstest]
#[case(0, 0, false)]
#[case(59, 22, false)]
#[case(60, 21, false)]
#[case(60, 22, true)]
#[case(80, 24, true)]
fn operational_minimum(#[case] w: u16, #[case] h: u16, #[case] usable: bool) {
    let mut app = App::default();
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    render(&mut t, &mut app, &Config::default());
    let s = text(t.backend().buffer());
    assert_eq!(s.contains("Explore"), usable);
    if w >= 18 && h > 0 && !usable {
        assert!(row(t.backend().buffer(), 0).starts_with(" Terminal too small"));
    }
    if usable {
        assert_eq!(s.lines().filter(|r| r.contains('─')).count(), 1);
        assert!(!s.contains('┌') && !s.contains('│') && !s.contains("Cycle"));
        assert!(row(t.backend().buffer(), 0).starts_with(" Explore"));
        assert!(row(t.backend().buffer(), 0).contains("Preview  Palette"));
        assert!(row(t.backend().buffer(), h - 1).contains("q Quit"));
    }
}

#[rstest]
#[case(None, 26, None)]
#[case(None, 27, Some(7))]
#[case(Some(Direction::Horizontal), 26, None)]
#[case(Some(Direction::Horizontal), 27, Some(7))]
#[case(Some(Direction::Vertical), 34, None)]
#[case(Some(Direction::Vertical), 35, Some(15))]
fn theme_info_keeps_previews_with_three_content_rows(
    #[case] direction: Option<Direction>,
    #[case] height: u16,
    #[case] preview_height: Option<u16>,
    #[values(false, true)] list_visible: bool,
) {
    let mut app = App::default();
    if let Some(direction) = direction {
        app.update(Action::TogglePin, 14);
        app.update(Action::ToggleCompare, 14);
        app.compare_direction = direction;
    }
    if app.theme_list_visible() != list_visible {
        app.update(Action::ToggleThemeList, 14);
    }
    app.update(Action::OpenThemeInfo, 14);
    let screen = layout::screen(Rect::new(0, 0, 60, height)).unwrap();
    let info = screen.info(&app).unwrap();
    assert_eq!(info.preview.map(|area| area.height), preview_height);
    assert_eq!(
        info.area.height,
        if preview_height.is_some() {
            13
        } else {
            height - 6
        }
    );
    if let Some(area) = info.preview {
        let pane = direction.map_or(area, |direction| layout::comparison(area, direction)[0]);
        assert_eq!(preview::content_area(pane).height, 3);
        if direction == Some(Direction::Horizontal) && list_visible {
            assert_eq!(preview::content_area(pane).width, 13);
        }
    }
}

#[rstest]
#[case(78, 20, 20)]
#[case(80, 28, 20)]
#[case(106, 28, 20)]
#[case(108, 28, 28)]
fn saved_and_comparison_list_widths_keep_their_terminal_breakpoints(
    #[case] width: u16,
    #[case] saved_width: u16,
    #[case] compare_width: u16,
) {
    let mut app = crate::test_support::comparison_in_context(InputContext::Browse);
    app.update(Action::ToggleThemeList, 14);
    let screen = layout::screen(Rect::new(0, 0, width, 36)).unwrap();
    assert_eq!(screen.saved().list.width, saved_width);
    assert_eq!(
        screen.theme_preview(&app).themes.unwrap().width,
        compare_width
    );
    assert_eq!(
        screen.theme_preview(&app).preview.width,
        width - compare_width - 4
    );
}

#[test]
fn selection_and_matching_have_distinct_styles_and_aligned_marks() {
    let mut app = App::default();
    query(&mut app, "catpp");
    app.update(Action::TogglePin, 14);
    app.mark_saved(MarkChange {
        name: app.theme().unwrap().name.to_owned(),
        mark: Some(Mark::Favorite),
    });
    let mut t = Terminal::new(TestBackend::new(80, 24)).unwrap();
    render(&mut t, &mut app, &Config::default());
    let b = t.backend().buffer();
    assert!(row(b, 6).contains(" *+ Catppuccin Frappe"));
    for x in 1..29 {
        assert_eq!((b[(x, 6)].fg, b[(x, 6)].bg), (Color::Reset, Color::Reset));
        assert!(b[(x, 6)].modifier.contains(Modifier::REVERSED));
    }
    for x in 5..10 {
        assert_eq!(b[(x, 7)].fg, Color::Yellow);
        assert!(b[(x, 7)].modifier.contains(Modifier::BOLD));
    }
    assert_eq!(b[(10, 7)].fg, Color::Reset);
    app.update(Action::Next, 14);
    render(&mut t, &mut app, &Config::default());
    let b = t.backend().buffer();
    assert!(!b[(6, 6)].modifier.contains(Modifier::REVERSED));
    assert_eq!(b[(1, 6)].fg, Color::Reset);
    assert_eq!(b[(5, 6)].fg, Color::Yellow);
    query(&mut app, "LiquidCarbonTransparentInverse");
    render(&mut t, &mut app, &Config::default());
    let b = t.backend().buffer();
    for x in [1, 28] {
        assert_eq!(b[(x, 6)].symbol(), " ");
        assert!(b[(x, 6)].modifier.contains(Modifier::REVERSED));
    }
    query(&mut app, "no-such-theme");
    render(&mut t, &mut app, &Config::default());
    let b = t.backend().buffer();
    let empty: String = (1..29).map(|x| b[(x, 6)].symbol()).collect();
    assert_eq!(empty.trim(), "No matching themes");
    for x in [1, 28] {
        assert_eq!(b[(x, 6)].symbol(), " ");
        assert_eq!(b[(x, 6)].modifier, Modifier::empty());
    }
}

#[rstest]
#[case(Direction::Horizontal, "Pin: LiquidCarb..", "Current: Liquid..")]
#[case(
    Direction::Vertical,
    "Pin: LiquidCarbonTransparentInverse",
    "Current: LiquidCarbonTransparentIn.."
)]
fn long_theme_names_use_trailing_ellipsis_in_previews_and_remain_readable_in_info(
    #[case] direction: Direction,
    #[case] pin_title: &str,
    #[case] current_title: &str,
) {
    let mut app = App::default();
    query(&mut app, "LiquidCarbonTransparentInverse");
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(60, 22)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let preview = layout::screen(buffer.area)
        .unwrap()
        .theme_preview(&app)
        .preview;
    assert_eq!(
        row(buffer, preview.y)[usize::from(preview.x)..usize::from(preview.right())].trim_end(),
        "LiquidCarbonTransparentInv.."
    );

    app.update(Action::TogglePin, 14);
    app.update(Action::ToggleCompare, 14);
    app.update(Action::ToggleThemeList, 14);
    app.compare_direction = direction;
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let preview = layout::screen(buffer.area)
        .unwrap()
        .theme_preview(&app)
        .preview;
    for (pane, title) in layout::comparison(preview, direction)
        .into_iter()
        .zip([pin_title, current_title])
    {
        assert_eq!(
            row(buffer, pane.y)[usize::from(pane.x)..usize::from(pane.right())].trim_end(),
            title
        );
    }

    app.update(Action::OpenThemeInfo, 14);
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let area = layout::screen(buffer.area)
        .unwrap()
        .info(&app)
        .unwrap()
        .area;
    let headings = row(buffer, area.y + 2);
    assert_eq!(headings.trim(), "Pin             Current");
    for (x, width) in [
        (headings.find("Pin").unwrap(), 14),
        (headings.find("Current").unwrap(), 15),
    ] {
        let name: String = (3..6)
            .map(|offset| {
                row(buffer, area.y + offset)[x..x + width]
                    .trim_end()
                    .to_owned()
            })
            .collect();
        assert_eq!(name, "LiquidCarbonTransparentInverse");
        assert_eq!(&row(buffer, area.y + 6)[x..x + "Dark".len()], "Dark",);
    }
}

#[rstest]
#[case(60, 22)]
#[case(120, 48)]
fn explore_detail_panels_align_labels_values_and_theme_headings_with_comparison(
    #[case] width: u16,
    #[case] height: u16,
) {
    let mut app = App::default();
    for action in [Action::TogglePin, Action::Next, Action::OpenThemeInfo] {
        app.update(action, 14);
    }
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let area = layout::screen(buffer.area)
        .unwrap()
        .info(&app)
        .unwrap()
        .area;
    let label_x = row(buffer, area.y + 3).find("Theme").unwrap();
    let value_x = row(buffer, area.y + 2).find("Current:").unwrap();
    assert_eq!(row(buffer, area.y + 3).find("3024 Night"), Some(value_x));
    assert_eq!(row(buffer, area.y + 5).find("Foreground"), Some(label_x));
    assert_eq!(row(buffer, area.y + 5).find("#a5a2a2"), Some(value_x + 3));

    app.update(Action::ToggleCompare, 14);
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let area = layout::screen(buffer.area)
        .unwrap()
        .info(&app)
        .unwrap()
        .area;
    assert_eq!(row(buffer, area.y + 3).find("Theme"), Some(label_x));
    assert_eq!(row(buffer, area.y + 2).find("Pin"), Some(value_x));

    for action in [
        Action::OpenThemeInfo,
        Action::ToggleCompare,
        Action::ToggleColors,
    ] {
        app.update(action, 14);
    }
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let area = layout::screen(buffer.area)
        .unwrap()
        .colors(&app)
        .unwrap()
        .area;
    assert_eq!(row(buffer, area.y + 2).find("Role"), Some(label_x));
    assert_eq!(row(buffer, area.y + 2).find("Current:"), Some(value_x));
    assert_eq!(row(buffer, area.y + 3).find("Foreground"), Some(label_x));
    assert_eq!(row(buffer, area.y + 3).find("#a5a2a2"), Some(value_x + 3));
    app.update(Action::SwitchThemeTarget, 14);
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    assert_eq!(row(buffer, area.y + 2).find("Pin:"), Some(value_x));
    assert_eq!(row(buffer, area.y + 3).find("#4a4543"), Some(value_x + 3));
}

#[rstest]
#[case(60, 22)]
#[case(67, 24)]
#[case(68, 24)]
#[case(69, 24)]
#[case(80, 36)]
#[case(81, 36)]
#[case(120, 48)]
#[case(121, 48)]
fn info_and_color_details_align_comparison_headings_and_values(
    #[case] width: u16,
    #[case] height: u16,
    #[values(Direction::Horizontal, Direction::Vertical)] direction: Direction,
    #[values(false, true)] list_visible: bool,
) {
    let mut app = crate::test_support::comparison_in_context(InputContext::Colors);
    app.compare_direction = direction;
    if list_visible {
        app.update(Action::ToggleThemeList, 14);
    }
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let color_y = (0..height)
        .find(|&y| row(buffer, y).contains("Color details"))
        .unwrap();
    let color_headings = row(buffer, color_y + 2);
    let pin_x = color_headings.find("Pin").unwrap();
    let current_x = color_headings.find("Current").unwrap();

    app.update(Action::OpenThemeInfo, 14);
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let info_y = (0..height)
        .find(|&y| row(buffer, y).contains("Theme info · Overview"))
        .unwrap();
    let info_headings = row(buffer, info_y + 2);
    assert_eq!(info_headings.find("Pin"), Some(pin_x));
    assert_eq!(info_headings.find("Current"), Some(current_x));
    assert_eq!(
        info_headings.contains("Pin:"),
        color_headings.contains("Pin:"),
    );
    for (x, name, appearance, foreground, background) in [
        (pin_x, "3024 Day", "Light", "#4a4543", "#f7f7f7"),
        (current_x, "3024 Night", "Dark", "#a5a2a2", "#090300"),
    ] {
        for (offset, padding, value) in [
            (3, 0, name),
            (4, 0, appearance),
            (5, 3, foreground),
            (6, 3, background),
        ] {
            let start = x + padding;
            assert_eq!(
                &row(buffer, info_y + offset)[start..start + value.len()],
                value,
            );
        }
    }
}

#[rstest]
#[case(
    80,
    24,
    Direction::Horizontal,
    Rect::new(1, 4, 38, 18),
    Rect::new(41, 4, 38, 18)
)]
#[case(
    81,
    24,
    Direction::Horizontal,
    Rect::new(1, 4, 38, 18),
    Rect::new(41, 4, 38, 18)
)]
#[case(
    80,
    24,
    Direction::Vertical,
    Rect::new(1, 4, 78, 8),
    Rect::new(1, 13, 78, 8)
)]
#[case(
    80,
    25,
    Direction::Vertical,
    Rect::new(1, 4, 78, 9),
    Rect::new(1, 14, 78, 9)
)]
fn comparisons_repaint_equal_content_and_backgrounds(
    #[case] w: u16,
    #[case] h: u16,
    #[case] direction: Direction,
    #[case] a: Rect,
    #[case] b: Rect,
) {
    let mut app = App::default();
    for action in [
        Action::PresetNext,
        Action::PresetNext,
        Action::TogglePin,
        Action::Next,
        Action::ToggleCompare,
    ] {
        app.update(action, 14);
    }
    app.compare_direction = direction;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    render(&mut t, &mut app, &Config::default());
    let buf = t.backend().buffer();
    assert!(row(buf, 0).contains("Compare") && !row(buf, 0).contains("Pin"));
    for y in 2..a.height {
        for x in 0..a.width {
            let first = &buf[(a.x + x, a.y + y)];
            let second = &buf[(b.x + x, b.y + y)];
            assert_eq!(first.symbol(), second.symbol());
            assert_eq!(first.bg, Color::Rgb(0xf7, 0xf7, 0xf7));
            assert_eq!(second.bg, Color::Rgb(0x09, 0x03, 0x00));
            assert!(!first.modifier.contains(Modifier::REVERSED));
        }
    }
    assert_eq!(buf[(a.x + 2, a.y + 3)].symbol(), "~");
    for pane in [a, b] {
        for y in pane.y + 2..pane.bottom() {
            for x in pane.x..pane.right() {
                if y == pane.y + 2
                    || y == pane.bottom() - 1
                    || x < pane.x + 2
                    || x >= pane.right() - 2
                {
                    assert_eq!(buf[(x, y)].symbol(), " ");
                }
            }
        }
    }
    app.update(Action::TogglePin, 14);
    render(&mut t, &mut app, &Config::default());
    let buf = t.backend().buffer();
    for y in 2..a.height {
        for x in 0..a.width {
            assert_eq!(buf[(a.x + x, a.y + y)], buf[(b.x + x, b.y + y)]);
        }
    }
}

#[rstest]
#[case((60, 22), false, None)]
#[case((120, 36), false, Some(Direction::Horizontal))]
#[case((60, 22), true, None)]
#[case((120, 36), true, Some(Direction::Horizontal))]
#[case((120, 36), true, Some(Direction::Vertical))]
fn editors_keep_controls_results_and_draft_errors(
    #[case] size: (u16, u16),
    #[case] filters: bool,
    #[case] direction: Option<Direction>,
) {
    let (w, h) = size;
    let mut app = App::default();
    app.update(Action::TogglePin, 14);
    app.update(Action::Next, 14);
    if let Some(d) = direction {
        app.update(Action::ToggleCompare, 14);
        app.compare_direction = d;
    }
    app.update(
        if filters {
            Action::EditFilters
        } else {
            Action::EditQuery
        },
        14,
    );
    let mode = app.mode;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    let mut previous_layout = None;
    for invalid in [false, true] {
        if invalid {
            if filters {
                app.update(Action::FieldPrevious, 14);
            }
            app.update(Action::ClearInput, 14);
            app.update(Action::Insert('"'), 14);
        }
        render(&mut t, &mut app, &Config::default());
        let cursor = t.get_cursor_position().unwrap();
        let b = t.backend().buffer();
        let s = text(b);
        assert!(s.contains("3024 Night"), "{s}");
        assert!(s.contains("Esc Cancel") && s.contains("C-c Quit"));
        assert_eq!(s.contains("Last valid results"), invalid);
        if filters {
            for field in [
                "Name",
                "Appearance",
                "Favorite",
                "Ignored",
                "BG brightness",
                "BG color",
                "Text contrast",
            ] {
                assert!(s.contains(field));
            }
        }
        let layout = layout::screen(b.area).unwrap().editor(&app).unwrap();
        if let Some(previous) = previous_layout {
            assert_eq!((layout.results, layout.preview), previous);
        }
        previous_layout = Some((layout.results, layout.preview));
        if filters && !invalid {
            let value = &b[(layout.controls.x + 16, layout.controls.y + 3)];
            assert_eq!(value.fg, Color::Reset);
            assert!(value.modifier.contains(Modifier::BOLD));
        }
        if app.context().has_text() {
            let input_x = layout.controls.x + if filters { 16 } else { 2 };
            let input = &b[(input_x, layout.controls.y + 2)];
            assert_eq!(input.fg, Color::Reset);
            assert!(input.modifier.is_empty());
            assert_eq!(b[(cursor.x, cursor.y)].fg, Color::Reset);
            assert!(b[(cursor.x, cursor.y)].modifier.is_empty());
        }
        if invalid {
            let error_x = layout.controls.x + if filters { 10 } else { 9 };
            let error_y = layout.controls.y;
            assert_eq!(b[(error_x, error_y)].fg, Color::Red);
            assert!(b[(error_x, error_y)].modifier.contains(Modifier::BOLD));
            assert!(row(b, error_y).contains("Close the double quote"));
        }
        if !filters {
            assert_eq!(layout.controls.width, w - 2);
            assert!(layout.controls.bottom() < layout.results.y);
        }
        assert!(layout.visible_rows() >= 4);
        if layout.preview.is_none() {
            assert!(row(b, 0).trim_end().ends_with("Preview  Palette"));
        }
        assert_eq!(app.mode, mode);
        assert_eq!(app.theme().unwrap().name, "3024 Night");
        assert_eq!(app.pinned_theme().unwrap().name, "3024 Day");
        if app.context().has_text() {
            assert!(layout.controls.contains(cursor));
        }
    }
}

#[rstest]
#[case((60, 22), None, false)]
#[case((60, 23), None, false)]
#[case((60, 24), None, true)]
#[case((60, 25), None, true)]
#[case((80, 24), None, true)]
#[case((60, 23), Some(Direction::Horizontal), false)]
#[case((60, 24), Some(Direction::Horizontal), true)]
#[case((60, 24), Some(Direction::Vertical), false)]
#[case((60, 25), Some(Direction::Vertical), true)]
#[case((80, 24), Some(Direction::Vertical), false)]
#[case((80, 25), Some(Direction::Vertical), true)]
fn color_filter_controls_fit_at_preview_boundaries(
    #[case] size: (u16, u16),
    #[case] direction: Option<Direction>,
    #[case] preview_visible: bool,
) {
    let mut app = App::default();
    let (width, height) = size;
    app.update(Action::TogglePin, 14);
    if let Some(direction) = direction {
        app.update(Action::ToggleCompare, 14);
        app.compare_direction = direction;
    }
    query(
        &mut app,
        "brightness:very-light bg:orange-brown contrast:high",
    );
    app.update(Action::EditFilters, 14);
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    for _ in 0..4 {
        app.update(Action::FieldNext, 14);
    }
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    let editor = layout::screen(buffer.area).unwrap().editor(&app).unwrap();
    assert_eq!(editor.preview.is_some(), preview_visible);
    assert!(editor.controls.height >= 10 && editor.visible_rows() >= 4);
    assert!(
        (editor.controls.left()..editor.controls.right())
            .all(|x| buffer[(x, editor.controls.y + 6)].symbol() == " ")
    );
    for (offset, label, value) in [
        (5, "Ignored", "Hide"),
        (7, "BG brightness", "Very light"),
        (8, "> BG color", "Orange/Brown"),
        (9, "Text contrast", "High"),
    ] {
        let line = row(buffer, editor.controls.y + offset);
        assert!(line.contains(label) && line.contains(value), "{line}");
    }
    for x in editor.controls.x + 16..editor.controls.x + 28 {
        let cell = &buffer[(x, editor.controls.y + 8)];
        assert_eq!(cell.fg, Color::Reset);
        assert!(cell.modifier.contains(Modifier::BOLD));
    }
    app.update(Action::FieldNext, 14);
    app.update(Action::FieldNext, 14);
    insert_text(&mut app, "bg:blue");
    render(&mut terminal, &mut app, &config);
    let buffer = terminal.backend().buffer();
    assert!(row(buffer, editor.controls.y).contains("Use fields or Search"));
    if editor.preview.is_none() {
        assert!(
            (editor.controls.left()..editor.controls.right())
                .all(|x| buffer[(x, editor.controls.y + 10)].symbol() == " ")
        );
        assert!(row(buffer, editor.controls.y + 11).contains("No theme selected"));
    }
}

#[rstest]
#[case(67, 22, Direction::Horizontal, None)]
#[case(68, 22, Direction::Horizontal, Some((13, 8)))]
#[case(60, 24, Direction::Vertical, None)]
#[case(60, 25, Direction::Vertical, Some((24, 3)))]
fn search_previews_keep_visible_content_at_width_and_height_boundaries(
    #[case] width: u16,
    #[case] height: u16,
    #[case] direction: Direction,
    #[case] content_size: Option<(u16, u16)>,
) {
    let mut app = crate::test_support::comparison_in_context(InputContext::Query);
    app.compare_direction = direction;
    app.preset = Preset::Code;
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &Config::default());
    let buffer = terminal.backend().buffer();
    let editor = layout::screen(buffer.area).unwrap().editor(&app).unwrap();
    assert_eq!(editor.preview.is_some(), content_size.is_some());
    assert_eq!(
        text(buffer).matches("fn main() {").count(),
        if content_size.is_some() { 2 } else { 0 }
    );
    if let Some(area) = editor.preview {
        for pane in layout::comparison(area, direction) {
            let content = preview::content_area(pane);
            assert_eq!(Some((content.width, content.height)), content_size);
        }
    }
}

#[test]
fn narrow_search_errors_keep_the_cause_and_fix_visible() {
    let mut app = App::default();
    app.update(Action::EditQuery, 14);
    app.update(Action::ClearInput, 14);
    for ch in "unknown".repeat(40).chars() {
        app.update(Action::Insert(ch), 14);
    }
    let mut t = Terminal::new(TestBackend::new(60, 22)).unwrap();
    app.update(Action::Insert(':'), 14);
    render(&mut t, &mut app, &Config::default());
    let b = t.backend().buffer();
    let after = layout::screen(b.area).unwrap().editor(&app).unwrap();
    assert!(row(b, 1).contains("Last valid results"));
    let error = row(b, after.controls.y);
    assert!(error.contains("Unknown filter: see Help for keys."));
    assert!(error.trim_end().ends_with(".."));
    assert!(after.controls.contains(t.get_cursor_position().unwrap()));
}

#[rstest]
#[case(60, "LiquidCarbon..")]
#[case(80, "LiquidCarbonTra..")]
#[case(120, "LiquidCarbonTransparentInverse")]
fn header_aligns_preview_and_pin_and_keeps_the_last_valid_count(
    #[case] width: u16,
    #[case] pin_name: &str,
) {
    let mut app = App::default();
    query(&mut app, "LiquidCarbonTransparentInverse");
    app.update(Action::TogglePin, 14);
    query(&mut app, "ignore:all");
    let mut terminal = Terminal::new(TestBackend::new(width, 22)).unwrap();
    let mut metadata_x = None;
    for action in [None, Some(Action::PresetNext)] {
        if let Some(action) = action {
            app.update(action, 14);
        }
        render(&mut terminal, &mut app, &Config::default());
        let buffer = terminal.backend().buffer();
        let preview_x = row(buffer, 0).find("Preview").unwrap();
        assert_eq!(Some(preview_x), row(buffer, 1).find("Pin"));
        if let Some(previous) = metadata_x {
            assert_eq!(preview_x, previous);
        }
        metadata_x = Some(preview_x);
        assert_eq!(row(buffer, 1)[preview_x + 9..].trim_end(), pin_name);
        for y in [0, 1] {
            let label = &buffer[(preview_x as u16, y)];
            let value = &buffer[(preview_x as u16 + 9, y)];
            assert!(label.modifier.is_empty());
            assert_eq!(value.fg, Color::Reset);
            assert!(value.modifier.contains(Modifier::BOLD));
        }
        assert!(row(buffer, 1).starts_with(" 348 / 348 themes"));
        assert!(buffer[(1, 1)].modifier.contains(Modifier::BOLD));
        assert!(buffer[(5, 1)].modifier.is_empty());
        assert!(buffer[(11, 1)].modifier.is_empty());
    }
    app.update(Action::EditQuery, 14);
    app.update(Action::CursorLast, 14);
    app.update(Action::Insert('x'), 14);
    render(&mut terminal, &mut app, &Config::default());
    let summary = row(terminal.backend().buffer(), 1);
    assert!(summary.starts_with(" 348 / 348"));
    assert!(summary.contains("Last valid results"));
    assert_eq!(
        summary
            .find("Pin")
            .map(|byte| Line::raw(&summary[..byte]).width()),
        metadata_x
    );
}

#[rstest]
fn modals_keep_reset_borders_and_clear_padding_and_restore_the_original_view(
    #[values((60,22),(80,24),(120,36))] size: (u16, u16),
    #[values(
        Action::SaveFilter,
        Action::RenameFilter,
        Action::DeleteFilter,
        Action::OpenCopy,
        Action::OpenPreset
    )]
    action: Action,
) {
    let (w, h) = size;
    let mut app = App::default();
    for a in [
        Action::TogglePin,
        Action::ToggleCompare,
        Action::Next,
        Action::PresetNext,
    ] {
        app.update(a, 14);
    }
    if matches!(action, Action::RenameFilter | Action::DeleteFilter) {
        app.workspace = app
            .workspace
            .changed(&WorkspaceChange::New {
                name: "Night".into(),
                query: "type:dark".into(),
            })
            .unwrap();
        for a in [Action::OpenSaved, Action::Next] {
            app.update(a, 14);
        }
    }
    let mut config = Config::default();
    config.ui_theme.condition_key = Color::Magenta;
    let mut t = Terminal::new(TestBackend::new(w, h)).unwrap();
    render(&mut t, &mut app, &config);
    let before = t.backend().buffer().clone();
    app.update(action, 14);
    if action == Action::SaveFilter {
        insert_text(&mut app, "Night");
    }
    render(&mut t, &mut app, &config);
    let b = t.backend().buffer();
    let s = text(b);
    assert!(s.contains("Esc") && s.contains("C-c Quit"), "{s}");
    assert!(s.contains("Enter"), "{s}");
    let title = match action {
        Action::SaveFilter => "Save filter",
        Action::RenameFilter => "Rename filter",
        Action::DeleteFilter => "Delete filter?",
        Action::OpenCopy => "Copy theme",
        Action::OpenPreset => "Select preview",
        _ => "Saved filters",
    };
    let y = (0..h).find(|&y| row(b, y).contains(title)).unwrap();
    let x = (0..w).find(|&x| b[(x, y)].symbol() == &title[..1]).unwrap();
    if matches!(action, Action::SaveFilter | Action::RenameFilter) {
        assert!(row(b, y + 2).contains("> Name  Night"));
        assert_eq!(b[(x, y + 2)].fg, Color::Cyan);
        assert_eq!(b[(x + 2, y + 2)].fg, Color::Cyan);
        assert_eq!(b[(x + 8, y + 2)].fg, Color::Reset);
        assert!(b[(x + 8, y + 2)].modifier.is_empty());
        assert!(row(b, y + 4).contains("Condition"));
    }
    if action == Action::OpenCopy {
        let selected_y = (y + 2..h)
            .find(|&py| b[(x, py)].modifier.contains(Modifier::REVERSED))
            .unwrap();
        let right = (x..w)
            .take_while(|&px| b[(px, selected_y)].modifier.contains(Modifier::REVERSED))
            .last()
            .unwrap();
        assert_eq!(b[(x, selected_y)].symbol(), " ");
        assert_eq!(b[(right, selected_y)].symbol(), " ");
    }
    if action == Action::OpenPreset {
        assert_eq!(b[(x, y + 2)].symbol(), ">");
        assert_eq!(b[(x, y + 2)].fg, Color::Cyan);
        let value = &b[(x + 14, y + 2)];
        assert_eq!(value.fg, Color::Reset);
        assert!(value.modifier.contains(Modifier::BOLD));
        assert!(!value.modifier.contains(Modifier::REVERSED));
    }
    let left = x - 2;
    let top = y - 1;
    let right = (x..w).find(|&px| b[(px, top)].symbol() == "┐").unwrap();
    let bottom = (y..h).find(|&py| b[(left, py)].symbol() == "└").unwrap();
    if matches!(
        action,
        Action::SaveFilter | Action::RenameFilter | Action::DeleteFilter
    ) {
        let mut keys = 0;
        for py in y..bottom {
            let text = row(b, py);
            for key in ["type:", "ignore:"] {
                for (start, _) in text.match_indices(key) {
                    let px = text[..start].chars().count() as u16;
                    if px < left || px >= right {
                        continue;
                    }
                    keys += 1;
                    for column in px..px + key.len() as u16 {
                        assert_eq!(b[(column, py)].fg, Color::Magenta);
                        assert!(b[(column, py)].modifier.is_empty());
                    }
                    assert_eq!(b[(px + key.len() as u16, py)].fg, Color::Reset);
                }
            }
        }
        assert!(keys > 0);
    }
    if matches!(action, Action::SaveFilter | Action::RenameFilter) {
        assert!((x..right).all(|px| b[(px, y + 3)].symbol() == " "));
    }
    assert_eq!(b[(left, top)].symbol(), "┌");
    assert_eq!(b[(right, bottom)].symbol(), "┘");
    for py in top..=bottom {
        for px in left..=right {
            if py != top && py != bottom && px != left && px != right {
                continue;
            }
            let c = &b[(px, py)];
            assert_eq!((c.fg, c.bg), (Color::Reset, Color::Reset));
            assert_eq!(c.modifier, Modifier::empty());
        }
    }
    for px in left + 1..right {
        let c = &b[(px, y + 1)];
        assert_eq!(c.symbol(), " ");
        assert_eq!((c.fg, c.bg), (Color::Reset, Color::Reset));
        assert_eq!(c.modifier, Modifier::empty());
    }
    for py in top + 1..bottom {
        for px in [left + 1, right - 1] {
            let c = &b[(px, py)];
            assert_eq!(c.symbol(), " ");
            assert_eq!((c.fg, c.bg), (Color::Reset, Color::Reset));
            assert_eq!(c.modifier, Modifier::empty());
        }
    }
    if matches!(action, Action::SaveFilter | Action::RenameFilter) {
        let bounds = modal_rect(b);
        app.update(Action::ClearInput, 14);
        app.update(Action::Confirm, 14);
        render(&mut t, &mut app, &config);
        assert_eq!(modal_rect(t.backend().buffer()), bounds);
        let footer = layout::screen(t.backend().buffer().area).unwrap().footer;
        assert!(row(t.backend().buffer(), footer.y).contains("Error: Enter a name"));
        assert_eq!(t.backend().buffer()[(footer.x, footer.y)].fg, Color::Red);
        app.update(Action::Insert('N'), 14);
        render(&mut t, &mut app, &config);
        assert_eq!(modal_rect(t.backend().buffer()), bounds);
        assert!(!row(t.backend().buffer(), footer.y).contains("Error:"));
    }
    t.backend_mut().resize(59, 21);
    t.resize(Size::new(59, 21).into()).unwrap();
    render(&mut t, &mut app, &config);
    assert!(text(t.backend().buffer()).contains("Terminal too small"));
    t.backend_mut().resize(w, h);
    t.resize(Size::new(w, h).into()).unwrap();
    app.update(Action::Cancel, 14);
    render(&mut t, &mut app, &config);
    assert_eq!(t.backend().buffer(), &before);
}

fn active_text_input(app: &App) -> &tui_input::Input {
    match app.context() {
        InputContext::Query => &app.query_editor().unwrap().input,
        InputContext::FilterName => &app.filters().unwrap().input,
        InputContext::DialogName => &app.dialog.as_ref().unwrap().input,
        _ => panic!("expected a text field"),
    }
}

#[rstest]
#[case(InputContext::Query)]
#[case(InputContext::FilterName)]
#[case(InputContext::DialogName)]
fn unicode_editing_and_scrolling_work_in_each_text_field(#[case] context: InputContext) {
    let mut app = App::default();
    let config = Config::default();
    app.update(
        match context {
            InputContext::Query => Action::EditQuery,
            InputContext::FilterName => Action::EditFilters,
            _ => Action::SaveFilter,
        },
        14,
    );
    if context == InputContext::FilterName {
        app.update(Action::FieldPrevious, 14);
    }
    app.update(Action::ClearInput, 14);
    insert_text(&mut app, "e\u{301}👩‍💻🇯🇵界R");
    for action in [
        Action::CursorFirst,
        Action::CursorNext,
        Action::CursorNext,
        Action::CursorNext,
        Action::DeleteAfter,
        Action::DeleteBefore,
        Action::DeleteBefore,
        Action::DeleteBefore,
    ] {
        app.update(action, 14);
    }
    assert_eq!(active_text_input(&app).value(), "R");
    assert_eq!(active_text_input(&app).cursor(), 0);
    app.update(Action::CursorLast, 14);
    insert_text(&mut app, &"e\u{301}👩‍💻🇯🇵界".repeat(20));
    let (width, height) = if context == InputContext::DialogName {
        (60, 22)
    } else {
        (80, 24)
    };
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &config);
    let cursor = terminal.get_cursor_position().unwrap();
    assert!((4..width - 2).contains(&cursor.x));
    assert_eq!(
        terminal.backend().buffer()[(cursor.x - 2, cursor.y)].symbol(),
        "界"
    );
    app.update(Action::CursorFirst, 14);
    render(&mut terminal, &mut app, &config);
    let start = terminal.get_cursor_position().unwrap();
    assert_eq!(start.y, cursor.y);
    assert!(start.x < cursor.x);
    assert_eq!(terminal.backend().buffer()[start].symbol(), "R");
    app.update(Action::CursorWordNext, 14);
    render(&mut terminal, &mut app, &config);
    let next = terminal.get_cursor_position().unwrap();
    assert_eq!(terminal.backend().buffer()[next].symbol(), "👩‍💻");
    app.update(Action::CursorWordPrevious, 14);
    render(&mut terminal, &mut app, &config);
    assert_eq!(terminal.get_cursor_position().unwrap(), start);
}

#[rstest]
#[case(80, 24, false, false, false, "Small", 1)]
#[case(87, 28, false, false, false, "Small", 1)]
#[case(88, 27, false, false, false, "Small", 1)]
#[case(88, 28, false, false, false, "Medium", 1)]
#[case(106, 33, false, false, false, "Medium", 1)]
#[case(105, 34, false, false, false, "Medium", 1)]
#[case(106, 34, false, false, false, "Large", 1)]
#[case(78, 36, true, false, false, "Large", 1)]
#[case(118, 30, false, true, false, "Medium", 2)]
#[case(118, 30, false, true, true, "Small", 2)]
#[case(78, 65, true, true, false, "Large", 2)]
fn auto_preview_uses_content_dimensions_and_a_shared_comparison_size(
    #[case] width: u16,
    #[case] height: u16,
    #[case] full_width: bool,
    #[case] compare: bool,
    #[case] list: bool,
    #[case] expected: &str,
    #[case] copies: usize,
) {
    let mut app = App::default();
    app.preset = Preset::Git;
    if compare {
        app.update(Action::TogglePin, 14);
        app.update(Action::Next, 14);
        app.update(Action::ToggleCompare, 14);
        if full_width {
            app.update(Action::ToggleLayout, 14);
        }
        if list {
            app.update(Action::ToggleThemeList, 14);
        }
    } else if full_width {
        app.update(Action::ToggleThemeList, 14);
    }
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    render(&mut terminal, &mut app, &Config::default());
    let screen = text(terminal.backend().buffer());
    assert!(
        screen
            .lines()
            .next()
            .unwrap()
            .contains(&format!("Git {expected}"))
    );
    assert_eq!(screen.matches("git status --short").count(), copies);
    assert_eq!(
        screen.matches("git log --oneline").count(),
        if expected == "Small" { 0 } else { copies }
    );
    assert_eq!(
        screen.matches("feature/preview-sizes").count(),
        if expected == "Large" { copies } else { 0 }
    );
}

#[test]
fn palette_keeps_bright_colors_separate_from_attributes_and_uses_exact_background_colors() {
    let mut app = App::default();
    app.preview_content = crate::preview::ContentSize::Large;
    let mut terminal = Terminal::new(TestBackend::new(120, 36)).unwrap();
    render(&mut terminal, &mut app, &Config::default());
    let buffer = terminal.backend().buffer();
    let find = |needle: &str| {
        (0..36)
            .find_map(|y| row(buffer, y).find(needle).map(|x| (x as u16, y)))
            .unwrap()
    };
    let (x, y) = find("Reverse    ");
    let reversed = &buffer[(x + 11, y)];
    assert_eq!(reversed.fg, Color::Rgb(0xf7, 0xf7, 0xf7));
    assert_eq!(reversed.bg, Color::Rgb(0x4a, 0x45, 0x43));
    let (x, y) = find("Bold       ");
    assert!(buffer[(x + 11, y)].modifier.contains(Modifier::BOLD));
    let (x, y) = find("Underline  ");
    assert!(buffer[(x + 11, y)].modifier.contains(Modifier::UNDERLINED));
    let (x, y) = find("#e8bbd0");
    assert_eq!(buffer[(x, y)].fg, Color::Rgb(0xe8, 0xbb, 0xd0));
    assert!(!buffer[(x, y)].modifier.contains(Modifier::BOLD));
    let (x, y) = find("ANSI backgrounds");
    let sample = &buffer[(x + 8, y + 1)];
    assert_eq!(sample.bg, Color::Rgb(0x09, 0x03, 0x00));
    assert_eq!(sample.fg, Color::Rgb(0x4a, 0x45, 0x43));
    assert_eq!(buffer[(115, 32)].bg, Color::Rgb(0xf7, 0xf7, 0xf7));
}

#[rstest]
#[case(Preset::Logs, "10:42:00.000", true, 0)]
#[case(Preset::Logs, "DEBUG", false, 5)]
#[case(Preset::Logs, "INFO", false, 6)]
#[case(Preset::Logs, "WARN", false, 3)]
#[case(Preset::Logs, "ERROR", false, 1)]
#[case(Preset::Logs, "OK", false, 2)]
#[case(Preset::Code, "struct", false, 5)]
#[case(Preset::Code, "Job", false, 3)]
#[case(Preset::Code, "report", false, 4)]
#[case(Preset::Code, "println", false, 4)]
#[case(Preset::Code, "Err", false, 4)]
#[case(Preset::Code, "100", false, 6)]
#[case(Preset::Code, "\"queue limit exceeded\"", false, 2)]
#[case(Preset::Code, "// Count work", true, 0)]
fn log_and_code_roles_use_the_selected_theme_palette(
    #[case] preset: Preset,
    #[case] token: &str,
    #[case] bright: bool,
    #[case] index: usize,
) {
    let mut app = App::default();
    app.preset = preset;
    app.preview_content = crate::preview::ContentSize::Large;
    let mut terminal = Terminal::new(TestBackend::new(120, 36)).unwrap();
    for theme_index in [0, 49] {
        for _ in 0..theme_index {
            app.update(Action::Next, 14);
        }
        render(&mut terminal, &mut app, &Config::default());
        let buffer = terminal.backend().buffer();
        let theme = app.theme().unwrap();
        let (x, y) = (0..36)
            .find_map(|y| {
                let line = row(buffer, y);
                line.find(token)
                    .map(|x| (line[..x].chars().count() as u16, y))
            })
            .unwrap();
        let palette = if bright {
            theme.palette.bright
        } else {
            theme.palette.normal
        };
        for x in x..x + token.len() as u16 {
            let cell = &buffer[(x, y)];
            assert_eq!(cell.fg, Color::from(palette[index]));
            assert_eq!(cell.bg, Color::from(theme.background));
            assert!(cell.modifier.is_empty());
        }
    }
}

#[rstest]
#[case(crate::preview::ContentSize::Small, 40)]
#[case(crate::preview::ContentSize::Medium, 51)]
#[case(crate::preview::ContentSize::Large, 68)]
fn tui_backgrounds_share_sample_width_and_reset_outside_highlighted_rows(
    #[case] size: crate::preview::ContentSize,
    #[case] width: u16,
) {
    let mut app = App::default();
    app.preset = Preset::Tui;
    app.preview_content = size;
    let mut terminal = Terminal::new(TestBackend::new(120, 36)).unwrap();
    render(&mut terminal, &mut app, &Config::default());
    let buffer = terminal.backend().buffer();
    let find = |needle: &str| {
        (0..36)
            .find_map(|y| {
                let line = row(buffer, y);
                line.find(needle)
                    .map(|x| (line[..x].chars().count() as u16, y))
            })
            .unwrap()
    };
    let theme = app.theme().unwrap();
    let foreground = Color::from(theme.foreground);
    let background = Color::from(theme.background);
    let (title_x, title_y) = find("Job monitor");
    for x in title_x..title_x + width {
        assert_eq!(
            buffer[(x, title_y)].bg,
            Color::from(theme.palette.normal[6])
        );
        assert_eq!(buffer[(x, title_y)].fg, foreground);
    }
    assert_eq!(buffer[(title_x + width, title_y)].bg, background);
    let (x, y) = find("> worker");
    for x in x..x + width {
        assert_eq!(buffer[(x, y)].bg, Color::from(theme.palette.normal[4]));
        assert_eq!(buffer[(x, y)].fg, foreground);
    }
    assert_eq!(buffer[(x + width, y)].bg, background);
    if size == crate::preview::ContentSize::Large {
        let (x, y) = find("State    running");
        assert_eq!(buffer[(x, y)].bg, background);
    }
    let (x, y) = find("All  Running");
    assert!(buffer[(x, y)].modifier.contains(Modifier::BOLD));
    assert!(buffer[(x + 5, y)].modifier.is_empty());
    let (x, y) = find("queued");
    assert_eq!(buffer[(x, y)].fg, Color::from(theme.palette.normal[3]));
    assert_eq!(buffer[(x, y)].bg, background);
    assert!(buffer[(x, y)].modifier.is_empty());

    app.preset = Preset::Code;
    render(&mut terminal, &mut app, &Config::default());
    let buffer = terminal.backend().buffer();
    for y in title_y..title_y + 24 {
        for x in title_x..title_x + 80 {
            assert_eq!(buffer[(x, y)].bg, background);
        }
    }
    // Trailing whitespace inherits the theme's Foreground after switching previews.
    assert_eq!(buffer[(title_x + width - 1, title_y)].fg, foreground);
}

#[rstest]
#[case(InputContext::Colors, false)]
#[case(InputContext::ThemeInfo, false)]
#[case(InputContext::ThemeInfo, true)]
fn focus_labels_follow_visible_controls_without_changing_focus_on_resize(
    #[case] context: InputContext,
    #[case] credits_pin: bool,
    #[values(Direction::Horizontal, Direction::Vertical)] direction: Direction,
    #[values(false, true)] list_visible: bool,
) {
    let mut app = crate::test_support::comparison_in_context(context);
    app.compare_direction = direction;
    if list_visible {
        app.update(Action::ToggleThemeList, 14);
    }
    if credits_pin {
        app.update(Action::ValueNext, 14);
        app.update(Action::SwitchThemeTarget, 14);
    }
    let config = Config::default();
    let mut terminal = Terminal::new(TestBackend::new(120, 52)).unwrap();
    for (width, height) in [(120, 52), (60, 22), (120, 36), (120, 52)] {
        terminal.backend_mut().resize(width, height);
        terminal.resize(Size::new(width, height).into()).unwrap();
        for focused in [true, false] {
            render(&mut terminal, &mut app, &config);
            assert_eq!(
                app.context(),
                if focused {
                    context
                } else {
                    InputContext::Browse
                }
            );
            let buffer = terminal.backend().buffer();
            let screen = layout::screen(buffer.area).unwrap();
            let (themes, preview, label, label_text) = if let Some(info) = screen.info(&app) {
                (
                    info.themes,
                    info.preview,
                    (info.area.x + 13, info.area.y),
                    if credits_pin { "Credits" } else { "Overview" },
                )
            } else {
                let colors = screen.colors(&app).unwrap();
                (
                    colors.themes,
                    colors.preview,
                    (colors.area.x, colors.area.y),
                    "Color details",
                )
            };
            let expected = if focused {
                Some((label, label_text))
            } else if let Some(themes) = themes {
                Some(((themes.x, themes.y), "Themes"))
            } else {
                preview.map(|preview| {
                    let current = layout::comparison(preview, direction)[1];
                    ((current.x, current.y), "Current")
                })
            };
            let mut active = Vec::new();
            for y in screen.main.y..screen.main.bottom() {
                for x in screen.main.x..screen.main.right() {
                    let cell = &buffer[(x, y)];
                    if cell.symbol() != " "
                        && cell.fg == Color::Cyan
                        && cell.modifier.contains(Modifier::BOLD)
                    {
                        active.push((x, y));
                    }
                }
            }
            let expected: Vec<_> = expected
                .into_iter()
                .flat_map(|((x, y), text)| {
                    text.chars()
                        .enumerate()
                        .filter_map(move |(offset, symbol)| {
                            (symbol != ' ').then_some((x + offset as u16, y))
                        })
                })
                .collect();
            assert_eq!(
                active, expected,
                "{width}x{height}, {direction:?}, {context:?}"
            );
            app.update(Action::FieldNext, 14);
        }
    }
}
