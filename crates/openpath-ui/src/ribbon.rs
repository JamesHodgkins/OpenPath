//! Ribbon toolbar in the AutoCAD layout: a tab strip above a body of panels.
//!
//! The ribbon is declared as data ([`Ribbon`] -> [`Tab`] -> [`Panel`] ->
//! [`Column`] -> [`Item`]); each item names a command id. [`Ribbon::show`]
//! draws it and returns the id of the command the user clicked, if any. The
//! ribbon knows nothing about what commands do: whether an item is enabled
//! and what shortcut to show come from a [`CommandSource`].
//!
//! A panel may carry `extra` items. The arrow beside its title opens a
//! flyout under the panel with those items as an icon-only grid and the title
//! along its bottom edge, as AutoCAD's expandable panels do.
//!
//! A [`Split`] button runs the item last picked from its list (the first to
//! begin with); its arrow opens the list. A [`Cell::Slot`] leaves room for a
//! control the app draws itself, such as the layer list: [`Ribbon::show`]
//! says where each slot went.
//!
//! Not implemented yet: responsive collapse, galleries, pinning an expanded
//! panel open.

use std::collections::HashMap;

use egui::{pos2, vec2, Align2, CornerRadius, FontId, Frame, Margin, Rect, Sense, Stroke, Ui};

mod buttons;
mod rows;
mod split;

use crate::Theme;
use buttons::{icon_button, large_button, paint_hover, stack, text_width};

pub(super) const SMALL_ROW_H: f32 = 22.0;
/// A large button is as tall as a stack of three small ones.
pub(super) const LARGE_H: f32 = SMALL_ROW_H * 3.0;
pub(super) const ICON_LARGE: f32 = 32.0;
/// Space between a large icon and its label.
pub(super) const LARGE_GAP: f32 = 3.0;
/// Line pitch of a large button's label.
pub(super) const LABEL_LINE_H: f32 = 13.5;
pub(super) const ICON_SMALL: f32 = 14.0;
/// Total space beside a large button's widest line.
pub(super) const LARGE_PAD: f32 = 8.0;
/// Every large button is at least this wide, so they are alike; only a longer
/// label makes one wider.
pub(super) const LARGE_W: f32 = 48.0;
/// A small button: space before its icon, between icon and label, and after
/// the label.
pub(super) const SMALL_PAD: f32 = 4.0;
pub(super) const SMALL_GAP: f32 = 5.0;
pub(super) const SMALL_END: f32 = 6.0;
pub(super) const LABEL_SIZE: f32 = 11.5;
const PANEL_TITLE_H: f32 = 15.0;
/// Side of an icon-only button in a panel flyout.
pub(super) const FLY_BTN: f32 = 26.0;

/// A clickable ribbon button.
#[derive(Clone, Copy, Debug)]
pub struct Item {
    /// Command id, returned from [`Ribbon::show`] when clicked.
    pub command: &'static str,
    pub label: &'static str,
    /// Codepoint from [`crate::icons`].
    pub icon: char,
}

impl Item {
    pub const fn new(command: &'static str, label: &'static str, icon: char) -> Self {
        Self {
            command,
            label,
            icon,
        }
    }
}

/// A button with a list of items under its arrow. Its face runs the item
/// last picked from the list, the first to begin with.
#[derive(Clone, Debug)]
pub struct Split {
    /// Shown whichever item was picked; `None` shows the picked item's label.
    pub label: Option<&'static str>,
    pub items: Vec<Item>,
}

impl Split {
    /// Labelled with the item picked last, as AutoCAD's Trim/Extend.
    pub fn new(items: Vec<Item>) -> Self {
        Self { label: None, items }
    }

    /// Always labelled `label`, as AutoCAD's Paste.
    pub fn labelled(label: &'static str, items: Vec<Item>) -> Self {
        Self {
            label: Some(label),
            items,
        }
    }
}

/// One control in a row of a [`Column::Rows`].
#[derive(Clone, Debug)]
pub enum Cell {
    /// Small icon and label.
    Button(Item),
    /// Small icon alone; the label is its tooltip.
    Icon(Item),
    /// Small icon and label, with an arrow.
    Split(Split),
    /// Small icon alone, with an arrow.
    IconSplit(Split),
    /// Room `width` wide for a control the app draws, named by `id`.
    Slot { id: &'static str, width: f32 },
}

/// One column of a panel.
#[derive(Clone, Debug)]
pub enum Column {
    /// Big icon over a (up to) two-line label.
    Large(Item),
    /// Big split button: the icon runs the item, the label opens the list.
    LargeSplit(Split),
    /// Up to three small icon + label rows.
    Stack(Vec<Item>),
    /// Up to three rows of small controls.
    Rows(Vec<Vec<Cell>>),
}

#[derive(Clone, Debug, Default)]
pub struct Panel {
    pub title: &'static str,
    pub columns: Vec<Column>,
    /// Items in the flyout under the title; none means no arrow.
    pub extra: Vec<Item>,
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub title: &'static str,
    pub panels: Vec<Panel>,
}

#[derive(Clone, Debug, Default)]
pub struct Ribbon {
    pub tabs: Vec<Tab>,
}

/// Per-frame UI state the app keeps alongside its [`Ribbon`].
#[derive(Clone, Debug, Default)]
pub struct RibbonState {
    pub active_tab: usize,
    /// (tab, panel) whose flyout is open.
    pub open_panel: Option<(usize, usize)>,
    /// The item each split button last ran, by the command of its first item.
    pub split_picks: HashMap<&'static str, &'static str>,
}

/// What one frame of the ribbon produced.
#[derive(Clone, Debug, Default)]
pub struct RibbonOutput {
    /// The command id of a clicked, enabled item.
    pub clicked: Option<&'static str>,
    /// Where each [`Cell::Slot`] of the shown tab was laid out, for the app
    /// to draw its control in.
    pub slots: Vec<(&'static str, Rect)>,
}

/// What the ribbon needs to know about commands, supplied by the app so the
/// ribbon does not depend on the command registry.
pub trait CommandSource {
    /// Disabled items are drawn dimmed and ignore clicks. Unknown commands
    /// should report `false`.
    fn is_enabled(&self, command: &str) -> bool;
    /// Shortcut text for tooltips, e.g. `"Ctrl+Z"`.
    fn shortcut_text(&self, command: &str) -> Option<&str>;
    /// Toggled-on items (e.g. the armed drawing tool) are drawn pressed.
    fn is_active(&self, _command: &str) -> bool {
        false
    }
}

/// Bundle passed down while drawing.
struct Ctx<'a> {
    pub(super) theme: &'a Theme,
    pub(super) source: &'a dyn CommandSource,
}

impl Ctx<'_> {
    pub(super) fn tooltip(&self, item: &Item) -> String {
        match self.source.shortcut_text(item.command) {
            Some(shortcut) => format!("{} ({shortcut})", item.label),
            None => item.label.to_owned(),
        }
    }
}

impl Ribbon {
    /// Draw the tab strip and the active tab's panels. Returns the command id
    /// of a clicked, enabled item and where the slots went.
    pub fn show(
        &self,
        ui: &mut Ui,
        theme: &Theme,
        state: &mut RibbonState,
        source: &dyn CommandSource,
    ) -> RibbonOutput {
        let c = &theme.colors;
        let cx = Ctx { theme, source };
        let mut out = RibbonOutput::default();
        state.active_tab = state.active_tab.min(self.tabs.len().saturating_sub(1));
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        let titles: Vec<&str> = self.tabs.iter().map(|t| t.title).collect();
        let before = state.active_tab;
        ribbon_tabs(ui, theme, &mut state.active_tab, &titles);
        if state.active_tab != before {
            state.open_panel = None;
        }

        let Some(tab) = self.tabs.get(state.active_tab) else {
            return out;
        };
        Frame::new()
            .fill(c.ribbon_bg)
            .inner_margin(Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal_top(|ui| {
                    for (index, panel) in tab.panels.iter().enumerate() {
                        let id = (state.active_tab, index);
                        if let Some(command) = panel_ui(ui, &cx, panel, id, state, &mut out.slots) {
                            out.clicked = Some(command);
                        }
                    }
                });
            });
        // Dark edge under the ribbon, as in AutoCAD.
        let (edge, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
        ui.painter().rect_filled(edge, 0.0, c.divider);
        out
    }
}

/// AutoCAD-style tab strip: the active tab takes the ribbon body's color so
/// it merges into the body below it.
fn ribbon_tabs(ui: &mut Ui, theme: &Theme, selected: &mut usize, titles: &[&str]) {
    let c = &theme.colors;
    let top_rounded = CornerRadius {
        nw: 4,
        ne: 4,
        sw: 0,
        se: 0,
    };
    Frame::new()
        .fill(c.window_bg)
        .inner_margin(Margin {
            left: 8,
            right: 8,
            top: 3,
            bottom: 0,
        })
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for (i, title) in titles.iter().enumerate() {
                    let width = text_width(ui, title, theme.font_size_tab) + 24.0;
                    let (rect, response) =
                        ui.allocate_exact_size(vec2(width, 25.0), Sense::click());
                    let is_selected = *selected == i;
                    if is_selected {
                        ui.painter().rect_filled(rect, top_rounded, c.ribbon_bg);
                    } else if response.hovered() {
                        ui.painter().rect_filled(rect, top_rounded, c.surface);
                    }
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        title,
                        FontId::proportional(theme.font_size_tab),
                        if is_selected || response.hovered() {
                            c.text
                        } else {
                            c.text_weak
                        },
                    );
                    crate::widgets::focus_ring(ui, theme, &response, rect);
                    if response.clicked() {
                        *selected = i;
                    }
                }
            });
        });
}
/// The panel title centered in `rect`, with the flyout arrow beside it when
/// the panel is expandable. Returns true when the bar is clicked.
fn title_bar(ui: &mut Ui, cx: &Ctx, panel: &Panel, rect: Rect, open: bool) -> bool {
    let expandable = !panel.extra.is_empty();
    let response = ui.interact(
        rect,
        ui.id().with(("ribbon_title", panel.title, open)),
        if expandable {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if expandable && (open || response.hovered()) {
        paint_hover(ui, cx.theme, &response, rect, open);
    }
    let arrow_w = if expandable { 14.0 } else { 0.0 };
    let title_w = text_width(ui, panel.title, LABEL_SIZE - 1.0);
    let left = rect.center().x - (title_w + arrow_w) / 2.0;
    let color = cx.theme.colors.text_weak;
    let painter = ui.painter();
    painter.text(
        pos2(left, rect.center().y),
        Align2::LEFT_CENTER,
        panel.title,
        FontId::proportional(LABEL_SIZE - 1.0),
        color,
    );
    if expandable {
        crate::icon_set::paint(
            painter,
            pos2(left + title_w + 8.0, rect.center().y),
            Align2::CENTER_CENTER,
            if open {
                crate::icons::RIBBON_PANEL_CLOSE
            } else {
                crate::icons::RIBBON_PANEL_OPEN
            },
            10.0,
            color,
        );
    }
    expandable && response.clicked()
}

fn panel_ui(
    ui: &mut Ui,
    cx: &Ctx,
    panel: &Panel,
    id: (usize, usize),
    state: &mut RibbonState,
    slots: &mut Vec<(&'static str, Rect)>,
) -> Option<&'static str> {
    let c = &cx.theme.colors;
    let mut clicked = None;
    let open = &mut state.open_panel;
    let picks = &mut state.split_picks;
    let is_open = *open == Some(id);
    let mut title_rect = Rect::NOTHING;
    let mut toggled = false;
    let panel_rect = ui
        .vertical(|ui| {
            ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
            let row = ui.horizontal_top(|ui| {
                // Every panel is as tall as a large button, whatever it holds.
                ui.set_min_height(LARGE_H);
                for column in &panel.columns {
                    let hit = match column {
                        Column::Large(item) => large_button(ui, cx, item),
                        Column::LargeSplit(s) => split::large(ui, cx, s, picks),
                        Column::Stack(items) => ui.vertical(|ui| stack(ui, cx, items)).inner,
                        Column::Rows(rows) => {
                            ui.vertical(|ui| rows::show(ui, cx, rows, picks, slots))
                                .inner
                        }
                    };
                    if hit.is_some() {
                        clicked = hit;
                    }
                }
            });

            // Panel title, centered under the buttons.
            let arrow_w = if panel.extra.is_empty() { 0.0 } else { 14.0 };
            let title_w = text_width(ui, panel.title, LABEL_SIZE - 1.0);
            let width = row.response.rect.width().max(title_w + arrow_w + 10.0);
            let (rect, _) = ui.allocate_exact_size(vec2(width, PANEL_TITLE_H), Sense::hover());
            title_rect = rect;
            if !is_open {
                toggled = title_bar(ui, cx, panel, rect, false);
            }
        })
        .response
        .rect;

    if is_open {
        // The flyout covers the title and carries it at its bottom edge.
        let per_row = ((title_rect.width() - 8.0) / FLY_BTN).floor().max(1.0) as usize;
        let area = egui::Area::new(egui::Id::new(("ribbon_panel_extra", id)))
            .order(egui::Order::Foreground)
            .fixed_pos(title_rect.left_top())
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                Frame::new()
                    .fill(c.ribbon_bg)
                    .stroke(Stroke::new(1.0_f32, c.border))
                    .show(ui, |ui| {
                        ui.set_width(title_rect.width());
                        let mut hit = None;
                        ui.add_space(4.0);
                        for chunk in panel.extra.chunks(per_row) {
                            ui.horizontal(|ui| {
                                ui.add_space(4.0);
                                for item in chunk {
                                    let size = vec2(FLY_BTN, FLY_BTN);
                                    if icon_button(ui, cx, item, size, 16.0) {
                                        hit = Some(item.command);
                                    }
                                }
                            });
                        }
                        ui.add_space(2.0);
                        let (rect, _) = ui.allocate_exact_size(
                            vec2(title_rect.width(), PANEL_TITLE_H),
                            Sense::hover(),
                        );
                        toggled = title_bar(ui, cx, panel, rect, true);
                        hit
                    })
                    .inner
            });
        if let Some(command) = area.inner {
            clicked = Some(command);
            *open = None;
        }
        let pressed_outside = ui.input(|i| {
            i.pointer.any_pressed()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| !area.response.rect.contains(p))
        });
        if pressed_outside || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            *open = None;
        }
    }
    if toggled {
        *open = if is_open { None } else { Some(id) };
    }

    // Divider after the panel.
    ui.add_space(4.0);
    let (divider, _) = ui.allocate_exact_size(vec2(1.0, panel_rect.height()), Sense::hover());
    ui.painter().rect_filled(divider, 0.0, c.divider);
    ui.add_space(4.0);
    clicked
}
