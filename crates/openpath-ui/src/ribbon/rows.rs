//! A column of up to three rows of small controls: labelled and icon-only
//! buttons, split buttons, and slots the app fills with its own controls.

use egui::{vec2, Rect, Sense, Ui};

use super::buttons::{icon_button, labelled_width, small_button};
use super::split::{self, Picks};
use super::{Cell, Ctx, ICON_SMALL, SMALL_ROW_H};

/// Width of an icon-only button in a row.
const ICON_W: f32 = 22.0;

fn natural_width(ui: &Ui, cell: &Cell, picks: &Picks) -> f32 {
    match cell {
        Cell::Button(item) => labelled_width(ui, item.label),
        Cell::Icon(_) => ICON_W,
        Cell::Split(s) => split::small_width(ui, s, picks, false),
        Cell::IconSplit(s) => split::small_width(ui, s, picks, true),
        Cell::Slot { width, .. } => *width,
    }
}

/// Draw the rows; returns the command clicked. Each slot's rect is added to
/// `slots`.
pub(super) fn show(
    ui: &mut Ui,
    cx: &Ctx,
    rows: &[Vec<Cell>],
    picks: &mut Picks,
    slots: &mut Vec<(&'static str, Rect)>,
) -> Option<&'static str> {
    ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
    // A horizontal row is at least this tall; keep it to a small button's.
    ui.spacing_mut().interact_size.y = SMALL_ROW_H;
    let widths: Vec<Vec<f32>> = rows
        .iter()
        .map(|row| row.iter().map(|c| natural_width(ui, c, picks)).collect())
        .collect();
    let column_w = widths
        .iter()
        .map(|w| w.iter().sum::<f32>())
        .fold(0.0, f32::max);
    let mut clicked = None;
    for (row, widths) in rows.iter().zip(&widths) {
        let spare = column_w - widths.iter().sum::<f32>();
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
            for (i, (cell, &width)) in row.iter().zip(widths).enumerate() {
                // The last labelled control takes the room left over, so
                // stacked buttons line up and hover alike.
                let stretch =
                    i + 1 == row.len() && !matches!(cell, Cell::Icon(_) | Cell::IconSplit(_));
                let width = if stretch { width + spare } else { width };
                let hit = match cell {
                    Cell::Button(item) => small_button(ui, cx, item, width).then_some(item.command),
                    Cell::Icon(item) => {
                        let size = vec2(width, SMALL_ROW_H);
                        icon_button(ui, cx, item, size, ICON_SMALL + 1.0).then_some(item.command)
                    }
                    Cell::Split(s) => split::small(ui, cx, s, false, width, picks),
                    Cell::IconSplit(s) => split::small(ui, cx, s, true, width, picks),
                    Cell::Slot { id, .. } => {
                        let (rect, _) =
                            ui.allocate_exact_size(vec2(width, SMALL_ROW_H), Sense::hover());
                        slots.push((*id, rect.shrink2(vec2(2.0, 1.0))));
                        None
                    }
                };
                if hit.is_some() {
                    clicked = hit;
                }
            }
        });
    }
    clicked
}
