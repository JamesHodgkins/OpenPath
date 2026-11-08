//! Preset and template cards.

use egui::{Align2, CornerRadius, FontId, Response, Sense, Stroke, Ui};

use super::focus_ring;
use crate::Theme;

const CARD_SIZE: f32 = 148.0;

/// Preset card: large icon over a title and subtitle. A selected card gets a
/// dashed blue outline.
pub fn preset_card(
    ui: &mut Ui,
    theme: &Theme,
    icon: char,
    title: &str,
    subtitle: &str,
    selected: bool,
) -> Response {
    let c = &theme.colors;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(CARD_SIZE, CARD_SIZE), Sense::click());
    let radius = CornerRadius::same(theme.radius);
    if selected {
        ui.painter().rect_filled(rect, radius, c.input_bg);
        let corners = [
            rect.left_top(),
            rect.right_top(),
            rect.right_bottom(),
            rect.left_bottom(),
            rect.left_top(),
        ];
        ui.painter().extend(egui::Shape::dashed_line(
            &corners,
            Stroke::new(2.0_f32, c.accent_hover),
            3.0,
            2.0,
        ));
    } else if response.hovered() {
        ui.painter().rect_filled(rect, radius, c.panel_raised);
    }
    let cx = rect.center().x;
    crate::icon_set::paint(
        ui.painter(),
        egui::pos2(cx, rect.top() + 54.0),
        Align2::CENTER_CENTER,
        icon,
        if selected { 30.0 } else { 56.0 },
        c.text_weak,
    );
    ui.painter().text(
        egui::pos2(cx, rect.top() + 108.0),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(theme.font_size_small),
        c.text,
    );
    ui.painter().text(
        egui::pos2(cx, rect.top() + 125.0),
        Align2::CENTER_CENTER,
        subtitle,
        FontId::proportional(theme.font_size_small - 1.0),
        c.text_weak,
    );
    focus_ring(ui, theme, &response, rect);
    response
}

/// Template card: a preview area (here a flat tint) over a caption bar with
/// the title and a small badge.
pub fn template_card(
    ui: &mut Ui,
    theme: &Theme,
    tint: egui::Color32,
    title: &str,
    badge: &str,
) -> Response {
    let c = &theme.colors;
    let caption_h = 30.0;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(CARD_SIZE, CARD_SIZE), Sense::click());
    let preview = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.right(), rect.bottom() - caption_h),
    );
    let caption = egui::Rect::from_min_max(egui::pos2(rect.left(), preview.bottom()), rect.max);
    let top = CornerRadius {
        nw: theme.radius,
        ne: theme.radius,
        sw: 0,
        se: 0,
    };
    let bottom = CornerRadius {
        nw: 0,
        ne: 0,
        sw: theme.radius,
        se: theme.radius,
    };
    ui.painter().rect_filled(preview, top, tint);
    ui.painter().rect_filled(
        caption,
        bottom,
        if response.hovered() {
            c.hover
        } else {
            c.panel_raised
        },
    );
    let font = FontId::proportional(theme.font_size_small);
    let badge_w = ui
        .painter()
        .layout_no_wrap(badge.to_owned(), font.clone(), c.text_weak)
        .size()
        .x;
    ui.painter().text(
        egui::pos2(caption.right() - 10.0, caption.center().y),
        Align2::RIGHT_CENTER,
        badge,
        font.clone(),
        c.text_weak,
    );
    let mut job = egui::text::LayoutJob::simple(
        title.to_owned(),
        font,
        c.text,
        caption.width() - 20.0 - badge_w - 8.0,
    );
    job.wrap.max_rows = 1;
    let galley = ui.painter().layout_job(job);
    let pos = egui::pos2(
        caption.left() + 10.0,
        caption.center().y - galley.size().y / 2.0,
    );
    ui.painter().galley(pos, galley, c.text);
    focus_ring(ui, theme, &response, rect);
    response
}
