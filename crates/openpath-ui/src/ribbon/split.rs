//! Split buttons, as in AutoCAD: the face runs the item last picked from the
//! list, and the arrow opens the list.

use std::collections::HashMap;

use egui::{pos2, vec2, Align2, FontId, Id, Rect, Response, Sense, Ui};

use super::buttons::{paint_hover, paint_small, sense, text_width};
use super::{
    Ctx, Item, Split, ICON_LARGE, ICON_SMALL, LABEL_LINE_H, LABEL_SIZE, LARGE_GAP, LARGE_H,
    LARGE_PAD, LARGE_W, SMALL_GAP, SMALL_PAD,
};
use crate::{icons, menu};

/// The item each split last ran, keyed by the command of its first item.
pub(super) type Picks = HashMap<&'static str, &'static str>;

/// Width of the arrow part of a small split button.
pub(super) const ARROW_W: f32 = 13.0;
const LIST_ROW_H: f32 = 24.0;
/// Space above a large split's icon, so icon and label sit where a plain
/// large button's do.
const LARGE_TOP: f32 = (LARGE_H - ICON_LARGE - LARGE_GAP - LABEL_LINE_H) / 2.0;

/// The item the face runs: the one last picked, else the first.
pub(super) fn face(split: &Split, picks: &Picks) -> Option<Item> {
    let first = split.items.first()?;
    let picked = picks
        .get(first.command)
        .and_then(|&c| split.items.iter().find(|i| i.command == c));
    Some(*picked.unwrap_or(first))
}

/// The label shown: the split's own, or that of the item the face runs.
pub(super) fn label(split: &Split, face: &Item) -> &'static str {
    split.label.unwrap_or(face.label)
}

/// A large split button: the icon runs the face item, the label below it
/// (with its arrow) opens the list.
pub(super) fn large(
    ui: &mut Ui,
    cx: &Ctx,
    split: &Split,
    picks: &mut Picks,
) -> Option<&'static str> {
    let face = face(split, picks)?;
    let label = label(split, &face);
    let (line1, line2) = label.split_once(' ').unwrap_or((label, ""));
    let width =
        text_width(ui, line1, LABEL_SIZE).max(text_width(ui, line2, LABEL_SIZE) + 12.0) + LARGE_PAD;
    let width = width.max(LARGE_W);
    let (rect, _) = ui.allocate_exact_size(vec2(width, LARGE_H), Sense::hover());
    let label_y = rect.top() + LARGE_TOP + ICON_LARGE + LARGE_GAP;
    let split_y = label_y - 1.0;
    let top = Rect::from_min_max(rect.min, pos2(rect.right(), split_y));
    let bottom = Rect::from_min_max(pos2(rect.left(), split_y), rect.max);
    let id = Id::new(("ribbon_split", split.items[0].command));
    let enabled = cx.source.is_enabled(face.command);
    let face_response = ui.interact(top, id.with("face"), sense(enabled));
    let arrow_response = ui.interact(bottom, id.with("arrow"), Sense::click());
    if enabled {
        let active = cx.source.is_active(face.command);
        paint_hover(ui, cx.theme, &face_response, top, active);
    }
    paint_hover(ui, cx.theme, &arrow_response, bottom, false);

    let c = &cx.theme.colors;
    let color = if enabled { c.text } else { c.text_disabled };
    // The label opens the list, so it is dimmed only when nothing in it runs.
    let any_enabled = split.items.iter().any(|i| cx.source.is_enabled(i.command));
    let label_color = if any_enabled { c.text } else { c.text_disabled };
    let painter = ui.painter();
    crate::icon_set::paint(
        painter,
        pos2(rect.center().x, rect.top() + LARGE_TOP + ICON_LARGE / 2.0),
        Align2::CENTER_CENTER,
        face.icon,
        ICON_LARGE,
        color,
    );
    let font = FontId::proportional(LABEL_SIZE);
    let y = label_y;
    painter.text(
        pos2(rect.center().x, y),
        Align2::CENTER_TOP,
        line1,
        font.clone(),
        label_color,
    );
    // The arrow goes under a one-line label, or after the second line.
    let arrow_at = if line2.is_empty() {
        pos2(rect.center().x, y + LABEL_LINE_H + 4.0)
    } else {
        let line2_w = text_width(ui, line2, LABEL_SIZE);
        painter.text(
            pos2(rect.center().x - 6.0, y + LABEL_LINE_H),
            Align2::CENTER_TOP,
            line2,
            font,
            label_color,
        );
        pos2(
            rect.center().x - 6.0 + line2_w / 2.0 + 7.0,
            y + LABEL_LINE_H * 1.5,
        )
    };
    paint_arrow(ui, cx, arrow_at);
    let face_clicked = face_response.on_hover_text(cx.tooltip(&face)).clicked();
    run(
        ui,
        cx,
        split,
        picks,
        face,
        enabled && face_clicked,
        &arrow_response,
        rect,
    )
}

/// The natural width of a small split button, which follows the label shown
/// (as AutoCAD's, the panel widens when a longer item is picked).
pub(super) fn small_width(ui: &Ui, split: &Split, picks: &Picks, icon_only: bool) -> f32 {
    let face_w = match face(split, picks) {
        Some(face) if !icon_only => {
            SMALL_PAD
                + ICON_SMALL
                + SMALL_GAP
                + text_width(ui, label(split, &face), LABEL_SIZE)
                + 3.0
        }
        _ => ICON_SMALL + 8.0,
    };
    face_w + ARROW_W
}

/// A small split button `width` wide: the icon (and label, unless
/// `icon_only`) runs the face item, the arrow at the right opens the list.
pub(super) fn small(
    ui: &mut Ui,
    cx: &Ctx,
    split: &Split,
    icon_only: bool,
    width: f32,
    picks: &mut Picks,
) -> Option<&'static str> {
    let face = face(split, picks)?;
    let (rect, _) = ui.allocate_exact_size(vec2(width, super::SMALL_ROW_H), Sense::hover());
    let split_x = rect.right() - ARROW_W;
    let left = Rect::from_min_max(rect.min, pos2(split_x, rect.bottom()));
    let right = Rect::from_min_max(pos2(split_x, rect.top()), rect.max);
    let id = Id::new(("ribbon_split", split.items[0].command));
    let enabled = cx.source.is_enabled(face.command);
    let face_response = ui.interact(left, id.with("face"), sense(enabled));
    let arrow_response = ui.interact(right, id.with("arrow"), Sense::click());
    if enabled {
        let active = cx.source.is_active(face.command);
        paint_hover(ui, cx.theme, &face_response, left, active);
    }
    paint_hover(ui, cx.theme, &arrow_response, right, false);

    let c = &cx.theme.colors;
    let color = if enabled { c.text } else { c.text_disabled };
    if icon_only {
        crate::icon_set::paint(
            ui.painter(),
            left.center(),
            Align2::CENTER_CENTER,
            face.icon,
            ICON_SMALL + 1.0,
            color,
        );
    } else {
        paint_small(ui, left, face.icon, label(split, &face), color);
    }
    paint_arrow(ui, cx, right.center());
    let face_clicked = face_response.on_hover_text(cx.tooltip(&face)).clicked();
    run(
        ui,
        cx,
        split,
        picks,
        face,
        enabled && face_clicked,
        &arrow_response,
        rect,
    )
}

fn paint_arrow(ui: &Ui, cx: &Ctx, at: egui::Pos2) {
    crate::icon_set::paint(
        ui.painter(),
        at,
        Align2::CENTER_CENTER,
        icons::RIBBON_SPLIT_ARROW,
        9.0,
        cx.theme.colors.text_weak,
    );
}

/// Shows the list when the arrow opens it, and returns what runs this frame:
/// the face item if its part was clicked, or an item picked from the list,
/// which becomes the face.
#[allow(clippy::too_many_arguments)]
fn run(
    ui: &Ui,
    cx: &Ctx,
    split: &Split,
    picks: &mut Picks,
    face: Item,
    face_clicked: bool,
    arrow: &Response,
    whole: Rect,
) -> Option<&'static str> {
    let list_id = Id::new(("ribbon_split_list", split.items[0].command));
    let width = split
        .items
        .iter()
        .map(|i| text_width(ui, i.label, cx.theme.font_size_body))
        .fold(0.0, f32::max)
        + 48.0;
    let picked = menu::popup_at(ui, cx.theme, list_id, arrow.clicked(), whole, width, |ui| {
        let mut picked = None;
        for item in &split.items {
            if list_row(ui, cx, item, item.command == face.command) {
                picked = Some(*item);
            }
        }
        picked
    });
    if let Some(item) = picked {
        picks.insert(split.items[0].command, item.command);
        return Some(item.command);
    }
    if face_clicked {
        ui.data_mut(|d| d.insert_temp(list_id, false));
        return Some(face.command);
    }
    None
}

/// One entry in a split's list: icon and label. The face item is marked.
fn list_row(ui: &mut Ui, cx: &Ctx, item: &Item, is_face: bool) -> bool {
    let c = &cx.theme.colors;
    let enabled = cx.source.is_enabled(item.command);
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), LIST_ROW_H), sense(enabled));
    if enabled && response.hovered() {
        ui.painter().rect_filled(rect, 2.0, c.hover);
    } else if is_face {
        ui.painter().rect_filled(rect, 2.0, c.pressed);
    }
    let color = if enabled { c.text } else { c.text_disabled };
    crate::icon_set::paint(
        ui.painter(),
        pos2(rect.left() + 14.0, rect.center().y),
        Align2::CENTER_CENTER,
        item.icon,
        16.0,
        color,
    );
    ui.painter().text(
        pos2(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        item.label,
        FontId::proportional(cx.theme.font_size_body),
        color,
    );
    let clicked = response.on_hover_text(cx.tooltip(item)).clicked();
    enabled && clicked
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trim_extend() -> Split {
        Split::new(vec![
            Item::new("modify.trim", "Trim", 'a'),
            Item::new("modify.extend", "Extend", 'b'),
        ])
    }

    #[test]
    fn the_face_is_the_first_item_until_another_is_picked() {
        let split = trim_extend();
        let mut picks = Picks::new();
        assert_eq!(face(&split, &picks).map(|i| i.command), Some("modify.trim"));
        picks.insert("modify.trim", "modify.extend");
        let shown = face(&split, &picks);
        assert_eq!(shown.map(|i| i.command), Some("modify.extend"));
        assert_eq!(shown.map(|i| label(&split, &i)), Some("Extend"));
    }

    #[test]
    fn a_fixed_label_stays_and_a_stale_pick_falls_back() {
        let mut split = trim_extend();
        split.label = Some("Edit");
        let mut picks = Picks::new();
        picks.insert("modify.trim", "gone");
        let shown = face(&split, &picks);
        assert_eq!(shown.map(|i| i.command), Some("modify.trim"));
        assert_eq!(shown.map(|i| label(&split, &i)), Some("Edit"));
        assert!(face(&Split::new(Vec::new()), &picks).is_none());
    }
}
