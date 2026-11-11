//! The row of tabs above the drawing: a Home tab (a house, no name, never
//! closed) then one tab per open drawing, and a `+` for a new one.

use egui::{Align2, CornerRadius, FontId, Sense, Ui};

use crate::widgets::icon_button;
use crate::{icons, Theme};

/// What the user did in [`document_tabs`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentTabEvent {
    /// The Home tab was clicked.
    Home,
    /// Drawing tab `i` was clicked.
    Select(usize),
    Close(usize),
    New,
}

/// Row of tabs with close buttons and a `+` button. `selected` is the open
/// drawing whose tab is raised, or `None` while Home shows. The raised tab
/// takes the tone of the canvas, like AutoCAD's active document tab.
pub fn document_tabs(
    ui: &mut Ui,
    theme: &Theme,
    titles: &[String],
    selected: Option<usize>,
) -> Option<DocumentTabEvent> {
    let c = &theme.colors;
    let mut event = None;
    let height = 26.0;
    let font = FontId::proportional(theme.font_size_body);
    let radius = CornerRadius {
        nw: 6,
        ne: 6,
        sw: 0,
        se: 0,
    };
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        let (rect, response) = ui.allocate_exact_size(egui::vec2(34.0, height), Sense::click());
        let home = selected.is_none();
        let fill = if home {
            c.panel_raised
        } else if response.hovered() {
            c.hover
        } else {
            c.panel_bg
        };
        ui.painter().rect_filled(rect, radius, fill);
        crate::icon_set::paint(
            ui.painter(),
            rect.center(),
            Align2::CENTER_CENTER,
            icons::DOCUMENT_TAB_HOME,
            15.0,
            if home { c.text } else { c.text_weak },
        );
        crate::widgets::focus_ring(ui, theme, &response, rect);
        if response.on_hover_text("Home").clicked() {
            event = Some(DocumentTabEvent::Home);
        }
        for (i, title) in titles.iter().enumerate() {
            let is_selected = selected == Some(i);
            let text_w = ui
                .painter()
                .layout_no_wrap(title.clone(), font.clone(), c.text)
                .size()
                .x;
            let width = 14.0 + text_w + 8.0 + 18.0 + 8.0;
            let (rect, response) =
                ui.allocate_exact_size(egui::vec2(width, height), Sense::click());
            let fill = if is_selected {
                c.panel_raised
            } else if response.hovered() {
                c.hover
            } else {
                c.panel_bg
            };
            ui.painter().rect_filled(rect, radius, fill);
            ui.painter().text(
                egui::pos2(rect.left() + 14.0, rect.center().y),
                Align2::LEFT_CENTER,
                title,
                font.clone(),
                if is_selected { c.text } else { c.text_weak },
            );

            let close_rect = egui::Rect::from_center_size(
                egui::pos2(rect.right() - 17.0, rect.center().y),
                egui::vec2(18.0, 18.0),
            );
            let close = ui.interact(close_rect, response.id.with("close"), Sense::click());
            if is_selected || response.hovered() || close.hovered() {
                if close.hovered() {
                    ui.painter().rect_filled(close_rect, 3.0, c.hover);
                }
                crate::icon_set::paint(
                    ui.painter(),
                    close_rect.center(),
                    Align2::CENTER_CENTER,
                    icons::DOCUMENT_TAB_CLOSE,
                    12.0,
                    c.text,
                );
            }
            crate::widgets::focus_ring(ui, theme, &response, rect);
            crate::widgets::focus_ring(ui, theme, &close, close_rect);
            let close = close.on_hover_text(format!("Close {title}"));
            if close.clicked() {
                event = Some(DocumentTabEvent::Close(i));
            } else if response.on_hover_text(title).clicked() {
                event = Some(DocumentTabEvent::Select(i));
            }
        }
        if icon_button(ui, theme, icons::DOCUMENT_TAB_NEW, "New drawing").clicked() {
            event = Some(DocumentTabEvent::New);
        }
    });
    event
}
