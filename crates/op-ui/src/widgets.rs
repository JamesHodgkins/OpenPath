//! Themed widgets. These wrap egui widgets with theme-driven styling.

use egui::{
    Align2, Button, CornerRadius, FontId, Margin, Rect, Response, RichText, Sense, Stroke, Ui,
};

use crate::Theme;

mod cards;
mod containers;

pub use cards::{preset_card, template_card};
pub use containers::{dialog, group_box, tabbed_page};

/// Draw the shared, high-contrast keyboard-focus indicator around a custom
/// painted control. Native egui widgets draw their own focus state; controls
/// built with `allocate_exact_size` or `interact` must call this after painting.
pub fn focus_ring(ui: &Ui, theme: &Theme, response: &Response, rect: Rect) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.shrink(1.0),
            CornerRadius::same(theme.radius),
            Stroke::new(2.0_f32, theme.colors.accent_text),
            egui::StrokeKind::Inside,
        );
    }
}

/// Filled accent-blue button for the primary action in a dialog or panel.
pub fn primary_button(ui: &mut Ui, theme: &Theme, text: &str) -> Response {
    let c = &theme.colors;
    ui.scope(|ui| {
        let w = &mut ui.style_mut().visuals.widgets;
        let states = [
            (&mut w.inactive, c.accent),
            (&mut w.hovered, c.accent_hover),
            (&mut w.active, c.accent_pressed),
        ];
        for (state, fill) in states {
            state.weak_bg_fill = fill;
            state.bg_fill = fill;
            state.bg_stroke = Stroke::NONE;
            state.fg_stroke = Stroke::new(1.0_f32, c.text_on_accent);
        }
        ui.add(
            Button::new(RichText::new(text).strong())
                .corner_radius(CornerRadius::same(theme.pill_radius))
                .min_size(egui::vec2(theme.unit * 9.0, theme.control_height)),
        )
    })
    .inner
}

/// Outlined, transparent pill button for secondary actions (Close, Cancel).
pub fn secondary_button(ui: &mut Ui, theme: &Theme, text: &str) -> Response {
    let c = &theme.colors;
    ui.scope(|ui| {
        let w = &mut ui.style_mut().visuals.widgets;
        w.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
        w.inactive.bg_stroke = Stroke::new(1.5_f32, c.text);
        w.hovered.bg_stroke = Stroke::new(1.5_f32, c.text);
        w.active.bg_stroke = Stroke::new(1.5_f32, c.text);
        ui.add(
            Button::new(RichText::new(text).strong())
                .corner_radius(CornerRadius::same(theme.pill_radius))
                .min_size(egui::vec2(theme.unit * 9.0, theme.control_height)),
        )
    })
    .inner
}

pub use crate::icon_set::{icon_button, icon_text};

/// Text tabs with an underline on the active one. Returns true if the
/// selection changed.
pub fn tab_strip(ui: &mut Ui, theme: &Theme, selected: &mut usize, labels: &[&str]) -> bool {
    let c = &theme.colors;
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for (i, label) in labels.iter().enumerate() {
            let is_selected = *selected == i;
            let font = FontId::proportional(theme.font_size_tab);
            let galley = ui.painter().layout_no_wrap(
                (*label).to_owned(),
                font,
                if is_selected { c.text } else { c.text_weak },
            );
            let pad = egui::vec2(theme.unit * 1.25, theme.unit * 0.5);
            let (rect, response) =
                ui.allocate_exact_size(galley.size() + pad * 2.0, egui::Sense::click());
            let color = if is_selected || response.hovered() {
                c.text
            } else {
                c.text_weak
            };
            ui.painter().galley(rect.min + pad, galley, color);
            if is_selected {
                let underline = egui::Rect::from_min_max(
                    egui::pos2(rect.left() + theme.unit, rect.bottom() - 2.0),
                    egui::pos2(rect.right() - theme.unit, rect.bottom()),
                );
                ui.painter().rect_filled(underline, 0.0, c.text);
            }
            focus_ring(ui, theme, &response, rect);
            if response.clicked() && !is_selected {
                *selected = i;
                changed = true;
            }
        }
    });
    changed
}

/// Style the current (scoped) `ui` so inputs look like dark wells with a thin
/// border and a blue focus ring. Call inside `ui.scope`.
fn input_well_style(ui: &mut Ui, theme: &Theme) {
    let c = &theme.colors;
    let v = ui.visuals_mut();
    v.selection.stroke = Stroke::new(1.0_f32, c.accent_hover);
    let w = &mut v.widgets;
    let states = [
        (&mut w.inactive, c.border),
        (&mut w.hovered, c.border_strong),
        (&mut w.active, c.accent_hover),
        (&mut w.open, c.accent_hover),
    ];
    for (state, border) in states {
        state.weak_bg_fill = c.input_bg;
        state.bg_fill = c.input_bg;
        state.bg_stroke = Stroke::new(1.0_f32, border);
    }
}

/// Small gray caption above a field, e.g. `Width`.
pub fn field_label(ui: &mut Ui, theme: &Theme, text: &str) -> Response {
    ui.label(
        RichText::new(text)
            .size(theme.font_size_small)
            .color(theme.colors.text_weak),
    )
}

/// Single-line text input as a dark well. Fills the available width.
pub fn text_field(ui: &mut Ui, theme: &Theme, text: &mut String, hint: &str) -> Response {
    ui.scope(|ui| {
        input_well_style(ui, theme);
        ui.add(
            egui::TextEdit::singleline(text)
                .hint_text(hint)
                .margin(Margin::symmetric(8, 5))
                .desired_width(f32::INFINITY),
        )
    })
    .inner
}

/// Numeric drag field with a unit suffix, e.g. `12.5 mm`, at the default width.
pub fn number_field(
    ui: &mut Ui,
    theme: &Theme,
    value: &mut f64,
    suffix: &str,
    speed: f64,
) -> Response {
    number_field_width(ui, theme, value, suffix, speed, theme.unit * 9.0)
}

/// Like [`number_field`] with an explicit minimum width; pass
/// `ui.available_width()` to fill a row.
pub fn number_field_width(
    ui: &mut Ui,
    theme: &Theme,
    value: &mut f64,
    suffix: &str,
    speed: f64,
    width: f32,
) -> Response {
    ui.scope(|ui| {
        input_well_style(ui, theme);
        ui.spacing_mut().interact_size.x = width;
        ui.add(
            egui::DragValue::new(value)
                .speed(speed)
                .suffix(if suffix.is_empty() {
                    String::new()
                } else {
                    format!(" {suffix}")
                }),
        )
    })
    .inner
}

/// Dropdown styled as an input well with a chevron. `width` is the total
/// width of the closed control; pass `ui.available_width()` to fill a row.
pub fn dropdown(
    ui: &mut Ui,
    theme: &Theme,
    id_salt: impl std::hash::Hash,
    selected: &str,
    width: f32,
    add_items: impl FnOnce(&mut Ui),
) -> Response {
    ui.scope(|ui| {
        input_well_style(ui, theme);
        ui.spacing_mut().button_padding = egui::vec2(theme.unit * 1.25, theme.unit * 0.5);
        egui::ComboBox::from_id_salt(id_salt)
            .selected_text(selected)
            .width(width)
            .icon(|ui, rect, visuals, _is_open, _above_or_below| {
                crate::icon_set::paint(
                    ui.painter(),
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    crate::icons::DROPDOWN_ARROW,
                    14.0,
                    visuals.fg_stroke.color,
                );
            })
            .show_ui(ui, add_items)
            .response
    })
    .inner
}

/// Square icon toggle (e.g. orientation). Selected state is filled blue.
pub fn icon_toggle(
    ui: &mut Ui,
    theme: &Theme,
    icon: char,
    selected: bool,
    tooltip: &str,
) -> Response {
    let c = &theme.colors;
    let side = theme.control_height;
    let (rect, response) = ui.allocate_exact_size(egui::vec2(side, side), Sense::click());
    let fill = if selected {
        c.accent
    } else if response.hovered() {
        c.hover
    } else {
        c.panel_raised
    };
    let radius = CornerRadius::same(theme.radius);
    ui.painter().rect_filled(rect, radius, fill);
    if !selected {
        ui.painter().rect_stroke(
            rect,
            radius,
            Stroke::new(1.0_f32, c.border),
            egui::StrokeKind::Inside,
        );
    }
    crate::icon_set::paint(
        ui.painter(),
        rect.center(),
        Align2::CENTER_CENTER,
        icon,
        18.0,
        if selected { c.text_on_accent } else { c.text },
    );
    focus_ring(ui, theme, &response, rect);
    response.on_hover_text(tooltip)
}

/// One `label  [editor]` row for property panels. The label column has a
/// fixed width so editors line up across rows.
pub fn property_row<R>(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    add_editor: impl FnOnce(&mut Ui) -> R,
) -> R {
    ui.horizontal(|ui| {
        ui.add_sized(
            [theme.unit * 12.0, theme.control_height],
            egui::Label::new(RichText::new(label).color(theme.colors.text_weak)).truncate(),
        );
        add_editor(ui)
    })
    .inner
}

/// Collapsible group with a small all-caps header.
pub fn section<R>(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    default_open: bool,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    egui::CollapsingHeader::new(
        RichText::new(title.to_uppercase())
            .small()
            .color(theme.colors.text_weak),
    )
    .id_salt(title)
    .default_open(default_open)
    .show(ui, add_contents)
    .body_returned
}

/// Small all-caps gray heading, e.g. `PRESET DETAILS`.
pub fn section_header(ui: &mut Ui, theme: &Theme, text: &str) -> Response {
    ui.label(
        RichText::new(text.to_uppercase())
            .small()
            .color(theme.colors.text_weak),
    )
}
