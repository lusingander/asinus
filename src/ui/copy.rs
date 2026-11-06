use ratatui::{Frame, layout::Rect, text::Line, widgets::Paragraph};

use crate::{
    app::App,
    config::Config,
    copy::CopyFormat,
    ui::{
        help, layout,
        widgets::{draw_modal, padded_label, wrap_text},
    },
};

pub fn draw(frame: &mut Frame, app: &App, config: &Config, main: Rect) {
    let Some(selection) = &app.copy_selection else {
        return;
    };
    let theme = config.ui_theme;
    let width = main.width.min(52).saturating_sub(4);
    let mut lines = vec![Line::styled(
        format!("Target: {}", selection.target.label()),
        theme.active_style(),
    )];
    lines.extend(wrap_text(
        &format!("Theme: {}", app.copy_theme().unwrap().name),
        width,
        2,
    ));
    lines.push(Line::default());
    let formats_row = lines.len() as u16;
    lines.extend([Line::default(), Line::default()]);
    let modal = layout::modal(main, 52, lines.len() as u16);
    draw_modal(frame, &modal, "Copy theme", theme);
    frame.render_widget(
        Paragraph::new(lines).style(theme.base_style()),
        modal.content,
    );
    let formats_y = modal.content.y + formats_row;
    for (index, format) in [CopyFormat::Name, CopyFormat::Json].into_iter().enumerate() {
        frame.render_widget(
            Paragraph::new(padded_label(format.label(), width)).style(
                if selection.format == format {
                    theme.selection_style()
                } else {
                    theme.base_style()
                },
            ),
            Rect::new(modal.content.x, formats_y + index as u16, width, 1),
        );
    }
    frame.render_widget(
        Paragraph::new(help::modal_footer(
            app,
            modal.actions.width,
            &config.keybindings,
            theme,
        )),
        modal.actions,
    );
}
