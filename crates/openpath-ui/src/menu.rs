//! Popup menus anchored under a widget: one general popup and a list menu
//! built on it.

use egui::{Align2, FontId, Id, Order, Response, Sense, Stroke, Ui};

use crate::{icons, Theme};

const ROW_HEIGHT: f32 = 22.0;
/// Lists longer than this scroll.
const MAX_VISIBLE_ROWS: usize = 12;

/// A popup under `anchor` that opens when the anchor is clicked. `content`
/// draws the popup and returns `Some` to pick something, which closes it.
/// Clicking elsewhere or pressing Escape closes it too. Open state lives in
/// egui memory under `id`.
pub fn popup<R>(
    ui: &Ui,
    theme: &Theme,
    id: Id,
    anchor: &Response,
    min_width: f32,
    content: impl FnOnce(&mut Ui) -> Option<R>,
) -> Option<R> {
    popup_at(
        ui,
        theme,
        id,
        anchor.clicked(),
        anchor.rect,
        min_width,
        content,
    )
}

/// [`popup`] opened and closed by `toggle` (the click on part of a widget,
/// such as a split button's arrow), lined up under the whole `anchor`.
pub fn popup_at<R>(
    ui: &Ui,
    theme: &Theme,
    id: Id,
    toggle: bool,
    anchor: egui::Rect,
    min_width: f32,
    content: impl FnOnce(&mut Ui) -> Option<R>,
) -> Option<R> {
    let mut open = ui.data(|d| d.get_temp::<bool>(id)).unwrap_or(false);
    if toggle {
        open = !open;
    }
    let mut picked = None;
    if open {
        let area = egui::Area::new(id.with("area"))
            .order(Order::Foreground)
            .fixed_pos(anchor.left_bottom() + egui::vec2(0.0, 2.0))
            .show(ui.ctx(), |ui| {
                egui::Frame::popup(ui.style())
                    .fill(theme.colors.panel_raised)
                    .inner_margin(egui::Margin::same(2))
                    .show(ui, |ui| {
                        ui.set_width(min_width.max(anchor.width() - 4.0));
                        ui.spacing_mut().item_spacing.y = 0.0;
                        picked = content(ui);
                    });
            });
        // Judged by position, not by `contains_pointer`, which is false
        // whenever the pointer is over a widget inside the popup.
        let outside = ui.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| !area.response.rect.contains(p) && !anchor.contains(p))
        });
        let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
        if picked.is_some() || outside || escape {
            open = false;
        }
    }
    ui.data_mut(|d| d.insert_temp(id, open));
    picked
}

/// Paints a small picture at the right of a menu row: the row's index, the
/// area to paint in and the row's text colour.
pub type Sample<'a> = &'a dyn Fn(&egui::Painter, egui::Rect, usize, egui::Color32);

/// A popup list of text entries; returns the index picked. The entry at
/// `current` is highlighted.
pub fn list_menu(
    ui: &Ui,
    theme: &Theme,
    id: Id,
    anchor: &Response,
    entries: &[String],
    current: Option<usize>,
) -> Option<usize> {
    list_menu_sampled(ui, theme, id, anchor, entries, current, None)
}

/// [`list_menu`] with a picture of `width` at the right of every row.
pub fn list_menu_sampled(
    ui: &Ui,
    theme: &Theme,
    id: Id,
    anchor: &Response,
    entries: &[String],
    current: Option<usize>,
    sample: Option<(f32, Sample<'_>)>,
) -> Option<usize> {
    popup(
        ui,
        theme,
        id,
        anchor,
        sample.map_or(0.0, |(w, _)| w + 150.0),
        |ui| {
            let rows = |ui: &mut Ui| {
                let mut picked = None;
                for (i, label) in entries.iter().enumerate() {
                    let selected = current == Some(i);
                    let drawn = sample.map(|(width, paint)| (width, paint, i));
                    if row(ui, theme, label, selected, drawn) {
                        picked = Some(i);
                    }
                }
                picked
            };
            if entries.len() > MAX_VISIBLE_ROWS {
                egui::ScrollArea::vertical()
                    .max_height(ROW_HEIGHT * MAX_VISIBLE_ROWS as f32)
                    .show(ui, rows)
                    .inner
            } else {
                rows(ui)
            }
        },
    )
}

/// One selectable text row; returns whether it was clicked.
pub fn menu_row(ui: &mut Ui, theme: &Theme, label: &str, selected: bool) -> bool {
    row(ui, theme, label, selected, None)
}

/// `label` laid out on one line no wider than `room`, ending in an ellipsis
/// when it does not fit; and whether it was cut.
fn ellipsized(
    ui: &Ui,
    label: &str,
    font: FontId,
    color: egui::Color32,
    room: f32,
) -> (std::sync::Arc<egui::Galley>, bool) {
    let full = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), color);
    let cut = full.size().x > room;
    let mut job = egui::text::LayoutJob::simple(label.to_owned(), font, color, room);
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    (ui.painter().layout_job(job), cut)
}

fn row(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    selected: bool,
    sample: Option<(f32, Sample<'_>, usize)>,
) -> bool {
    let c = &theme.colors;
    let (rect, response) =
        ui.allocate_exact_size(egui::vec2(ui.available_width(), ROW_HEIGHT), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, 2.0, c.accent);
    } else if response.hovered() {
        ui.painter().rect_filled(rect, 2.0, c.hover);
    }
    // The label ends in an ellipsis rather than run under a sample or past the edge.
    let sample_w = sample.as_ref().map_or(0.0, |s| s.0 + 8.0);
    let room = (rect.width() - 16.0 - sample_w).max(0.0);
    let text_color = if selected { c.text_on_accent } else { c.text };
    let font = FontId::proportional(theme.font_size_body);
    let (galley, cut) = ellipsized(ui, label, font, text_color, room);
    let at = egui::pos2(rect.left() + 8.0, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(at, galley, text_color);
    let response = if cut {
        response.on_hover_text(label)
    } else {
        response
    };
    if let Some((width, paint, index)) = sample {
        let area = egui::Rect::from_min_max(
            egui::pos2(rect.right() - width - 8.0, rect.top()),
            egui::pos2(rect.right() - 8.0, rect.bottom()),
        );
        paint(
            ui.painter(),
            area,
            index,
            if selected { c.text_on_accent } else { c.text },
        );
    }
    response.clicked()
}

/// A bordered field showing `text` with a chevron, drawn into `rect`. Look
/// only: the caller senses clicks. Long text ends in an ellipsis; returns
/// whether it was cut.
pub fn paint_combo(ui: &Ui, theme: &Theme, rect: egui::Rect, text: &str, hovered: bool) -> bool {
    let c = &theme.colors;
    ui.painter()
        .rect_filled(rect, 2.0, if hovered { c.hover } else { c.input_bg });
    ui.painter().rect_stroke(
        rect,
        2.0,
        Stroke::new(1.0_f32, c.border),
        egui::StrokeKind::Inside,
    );
    // Leave room for the chevron.
    let room = (rect.width() - 8.0 - 28.0).max(0.0);
    let font = FontId::proportional(theme.font_size_body);
    let (galley, cut) = ellipsized(ui, text, font, c.text, room);
    let at = egui::pos2(rect.left() + 8.0, rect.center().y - galley.size().y / 2.0);
    ui.painter().galley(at, galley, c.text);
    crate::icon_set::paint(
        ui.painter(),
        rect.right_center() - egui::vec2(8.0, 0.0),
        Align2::RIGHT_CENTER,
        icons::COMBO_ARROW,
        12.0,
        c.text_weak,
    );
    cut
}

/// A drop-down field: shows `text`, opens `entries` when clicked and returns
/// the index picked. With no entries it is a plain label field.
pub fn combo(
    ui: &mut Ui,
    theme: &Theme,
    id: Id,
    size: egui::Vec2,
    text: &str,
    entries: &[String],
    current: Option<usize>,
) -> Option<usize> {
    combo_sampled(ui, theme, id, size, text, entries, current, None)
}

/// [`combo`] whose list has a picture at the right of every row.
#[allow(clippy::too_many_arguments)]
pub fn combo_sampled(
    ui: &mut Ui,
    theme: &Theme,
    id: Id,
    size: egui::Vec2,
    text: &str,
    entries: &[String],
    current: Option<usize>,
    sample: Option<(f32, Sample<'_>)>,
) -> Option<usize> {
    let clickable = !entries.is_empty();
    let (rect, response) = ui.allocate_exact_size(
        size,
        if clickable {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    let cut = paint_combo(ui, theme, rect, text, clickable && response.hovered());
    let response = if cut {
        response.on_hover_text(text)
    } else {
        response
    };
    if !clickable {
        return None;
    }
    list_menu_sampled(ui, theme, id, &response, entries, current, sample)
}
