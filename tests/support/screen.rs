use termlens::{Color, Screen};

pub fn is_active_label(screen: &Screen, label: &str) -> bool {
    screen.find_all(label).into_iter().any(|(row, col)| {
        (col..col + label.len() as u16).all(|col| {
            let style = screen.cell(row, col).unwrap().style();
            style.fg == Color::Indexed(6) && style.bold
        })
    })
}

pub fn preview_name_matches(screen: &Screen, role: &str, expected: &str) -> bool {
    screen.find_all(role).into_iter().any(|(row, col)| {
        let tail: String = screen
            .row_text(row)
            .chars()
            .skip(col as usize + role.chars().count())
            .collect();
        let visible = tail.trim_start().split("  ").next().unwrap_or("").trim();
        if let Some(prefix) = visible.strip_suffix("..") {
            expected.starts_with(prefix)
        } else {
            visible == expected
        }
    })
}

pub fn is_selected(screen: &Screen, label: &str) -> bool {
    screen
        .find_all(label)
        .into_iter()
        .any(|(row, col)| screen.cell(row, col).unwrap().style().reverse)
}

pub fn focused_field_value(screen: &Screen, label: &str, value: &str) -> bool {
    let label = format!("> {label}");
    screen.find_all(&label).into_iter().any(|(row, col)| {
        screen
            .row_text(row)
            .chars()
            .skip(usize::from(col) + label.len())
            .collect::<String>()
            .trim_start()
            .starts_with(value)
    })
}

pub fn matching_themes(screen: &Screen, count: usize) -> bool {
    let summary = screen.row_text(1);
    let summary = summary.trim_start();
    summary.starts_with(&format!("{count} / 348"))
        || count == 348 && summary.starts_with("348 themes")
}

pub fn footer_contains(screen: &Screen, text: &str) -> bool {
    screen.row_text(screen.rows() - 1).contains(text)
}

pub fn theme_list_visible(screen: &Screen) -> bool {
    screen.row_text(4).trim_start().starts_with("Themes")
}

pub fn comparing(screen: &Screen) -> bool {
    screen.row_text(0).contains("Compare")
        && !screen.cursor_visible()
        && !screen.contains("· Draft")
        && !screen.contains("Last valid results")
}

pub fn comparison_layout(screen: &Screen, vertical: bool) -> bool {
    let Some((pin_row, pin_col)) = screen.find("Pin:") else {
        return false;
    };
    let Some((current_row, current_col)) = screen.find("Current:") else {
        return false;
    };
    if vertical {
        current_row > pin_row && current_col == pin_col
    } else {
        current_row == pin_row && current_col > pin_col
    }
}

pub fn shows_current_theme(screen: &Screen, name: &str) -> bool {
    if screen.row_text(0).contains("Compare") {
        preview_name_matches(screen, "Current:", name)
    } else {
        screen.contains(&format!("Current: {name}"))
            || [2, 3].into_iter().any(|divider_row| {
                screen.row_text(divider_row).trim_start().starts_with("──")
                    && screen.row_text(divider_row + 2).contains(name)
            })
    }
}
