//! The kinds of plain button a panel draws, and the painting they share.

use egui::{pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke, Ui, Vec2};

use super::{
    Ctx, Item, ICON_LARGE, ICON_SMALL, LABEL_LINE_H, LABEL_SIZE, LARGE_GAP, LARGE_H, LARGE_PAD,
    LARGE_W, SMALL_END, SMALL_GAP, SMALL_PAD, SMALL_ROW_H,
};
use crate::Theme;

/// An icon-only button of `size` (in a row, or a panel flyout); the label is
/// its tooltip.
pub(super) fn icon_button(ui: &mut Ui, cx: &Ctx, item: &Item, size: Vec2, icon_size: f32) -> bool {
    let enabled = cx.source.is_enabled(item.command);
    let (rect, response) = ui.allocate_exact_size(size, sense(enabled));
    if enabled {
        paint_hover(
            ui,
            cx.theme,
            &response,
            rect,
            cx.source.is_active(item.command),
        );
    }
    let c = &cx.theme.colors;
    crate::icon_set::paint(
        ui.painter(),
        rect.center(),
        Align2::CENTER_CENTER,
        item.icon,
        icon_size,
        if enabled { c.text } else { c.text_disabled },
    );
    let clicked = response.on_hover_text(cx.tooltip(item)).clicked();
    enabled && clicked
}

/// Width of a small button showing an icon and `label`.
pub(super) fn labelled_width(ui: &Ui, label: &str) -> f32 {
    SMALL_PAD + ICON_SMALL + SMALL_GAP + text_width(ui, label, LABEL_SIZE) + SMALL_END
}

pub(super) fn stack(ui: &mut Ui, cx: &Ctx, items: &[Item]) -> Option<&'static str> {
    let mut clicked = None;
    ui.spacing_mut().item_spacing.y = 0.0;
    let width = items
        .iter()
        .map(|i| labelled_width(ui, i.label))
        .fold(0.0, f32::max);
    for item in items {
        if small_button(ui, cx, item, width) {
            clicked = Some(item.command);
        }
    }
    clicked
}

pub(super) fn large_button(ui: &mut Ui, cx: &Ctx, item: &Item) -> Option<&'static str> {
    let c = &cx.theme.colors;
    let enabled = cx.source.is_enabled(item.command);
    let (line1, line2) = item.label.split_once(' ').unwrap_or((item.label, ""));
    let width =
        text_width(ui, line1, LABEL_SIZE).max(text_width(ui, line2, LABEL_SIZE)) + LARGE_PAD;
    let width = width.max(LARGE_W);
    let (rect, response) = ui.allocate_exact_size(vec2(width, LARGE_H), sense(enabled));
    if enabled {
        paint_hover(
            ui,
            cx.theme,
            &response,
            rect,
            cx.source.is_active(item.command),
        );
    }
    let color = if enabled { c.text } else { c.text_disabled };

    // Icon + label lines form one block, centered vertically so the button
    // has no dead space at the bottom (single-line labels included).
    let lines = if line2.is_empty() { 1.0 } else { 2.0 };
    let block_h = ICON_LARGE + LARGE_GAP + lines * LABEL_LINE_H;
    let top = rect.top() + ((LARGE_H - block_h) / 2.0).max(0.0);

    let painter = ui.painter();
    crate::icon_set::paint(
        painter,
        pos2(rect.center().x, top + ICON_LARGE / 2.0),
        Align2::CENTER_CENTER,
        item.icon,
        ICON_LARGE,
        color,
    );
    let font = FontId::proportional(LABEL_SIZE);
    let y = top + ICON_LARGE + LARGE_GAP;
    painter.text(
        pos2(rect.center().x, y),
        Align2::CENTER_TOP,
        line1,
        font.clone(),
        color,
    );
    if !line2.is_empty() {
        painter.text(
            pos2(rect.center().x, y + LABEL_LINE_H),
            Align2::CENTER_TOP,
            line2,
            font,
            color,
        );
    }
    let clicked = response.on_hover_text(cx.tooltip(item)).clicked();
    (enabled && clicked).then_some(item.command)
}

pub(super) fn small_button(ui: &mut Ui, cx: &Ctx, item: &Item, width: f32) -> bool {
    let c = &cx.theme.colors;
    let enabled = cx.source.is_enabled(item.command);
    let (rect, response) = ui.allocate_exact_size(vec2(width, SMALL_ROW_H), sense(enabled));
    if enabled {
        paint_hover(
            ui,
            cx.theme,
            &response,
            rect,
            cx.source.is_active(item.command),
        );
    }
    let color = if enabled { c.text } else { c.text_disabled };
    paint_small(ui, rect, item.icon, item.label, color);
    let clicked = response.on_hover_text(cx.tooltip(item)).clicked();
    enabled && clicked
}

/// A small button's icon and label, from the left of `rect`.
pub(super) fn paint_small(ui: &Ui, rect: Rect, icon: char, label: &str, color: Color32) {
    let painter = ui.painter();
    crate::icon_set::paint(
        painter,
        pos2(rect.left() + SMALL_PAD + ICON_SMALL / 2.0, rect.center().y),
        Align2::CENTER_CENTER,
        icon,
        ICON_SMALL,
        color,
    );
    painter.text(
        pos2(
            rect.left() + SMALL_PAD + ICON_SMALL + SMALL_GAP,
            rect.center().y,
        ),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(LABEL_SIZE),
        color,
    );
}

pub(super) fn sense(enabled: bool) -> Sense {
    if enabled {
        Sense::click()
    } else {
        Sense::hover()
    }
}

pub(super) fn paint_hover(
    ui: &Ui,
    theme: &Theme,
    response: &egui::Response,
    rect: Rect,
    active: bool,
) {
    // An active (selected) button is just a lighter fill, with no outline.
    let (fill, border) = if response.is_pointer_button_down_on() {
        (theme.colors.pressed, Some(theme.colors.border))
    } else if active {
        (theme.colors.pressed, None)
    } else if response.hovered() {
        (theme.colors.hover, Some(theme.colors.border))
    } else {
        return;
    };
    ui.painter().rect_filled(rect, 3.0, fill);
    if let Some(border) = border {
        ui.painter().rect_stroke(
            rect,
            3.0,
            Stroke::new(1.0_f32, border),
            egui::StrokeKind::Inside,
        );
    }
}

pub(super) fn text_width(ui: &Ui, text: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), FontId::proportional(size), Color32::WHITE)
        .size()
        .x
}
