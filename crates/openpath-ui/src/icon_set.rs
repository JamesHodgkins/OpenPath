//! Icon drawing: monochrome Lucide outline glyphs from the bundled icon font
//! (House Style §5). Scaffolded from `od-ui`; the SVG/`resvg` redraw system was removed
//! in favour of the Lucide font alone — [`crate::icons`] maps each icon identifier to its
//! glyph.

use egui::{
    Align2, Color32, FontId, Painter, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2,
};

/// Draw `icon` `size` points square, placed at `at` by `align`, in `color`. Returns the
/// rectangle it occupied.
pub fn paint(
    painter: &Painter,
    at: Pos2,
    align: Align2,
    icon: char,
    size: f32,
    color: Color32,
) -> Rect {
    painter.text(
        at,
        align,
        crate::icons::glyph(icon),
        FontId::new(size, crate::icon_family()),
        color,
    )
}

/// An icon glyph as rich text, sized and in the icon font family, for egui widgets that
/// take text.
pub fn icon_text(icon: char, size: f32) -> RichText {
    RichText::new(crate::icons::glyph(icon).to_string())
        .family(crate::icon_family())
        .size(size)
}

/// Square, frameless icon button with a tooltip; the frame appears on hover.
pub fn icon_button(
    ui: &mut egui::Ui,
    theme: &crate::Theme,
    icon: char,
    tooltip: &str,
) -> egui::Response {
    let side = theme.control_height;
    ui.scope(|ui| {
        let inactive = &mut ui.style_mut().visuals.widgets.inactive;
        inactive.weak_bg_fill = Color32::TRANSPARENT;
        inactive.bg_stroke = Stroke::NONE;
        let padding = ((side - 15.0) / 2.0).max(0.0);
        ui.add(Icon::new(icon, 15.0).button().padding(padding))
            .on_hover_text(tooltip)
    })
    .inner
}

/// How an [`Icon`] widget looks behind its icon.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Look {
    /// Nothing: a label, which may still be clicked.
    Plain,
    /// Lit when hovered or selected, as a selectable label.
    Toggle(bool),
    /// A button's frame.
    Button,
}

/// An icon as a widget, `size` points square with `padding` round it. Disabled in a
/// disabled `Ui` (`add_enabled`).
#[derive(Clone, Copy, Debug)]
pub struct Icon {
    icon: char,
    size: f32,
    padding: f32,
    sense: Sense,
    look: Look,
    color: Option<Color32>,
}

impl Icon {
    /// A plain icon that does nothing when clicked.
    pub fn new(icon: char, size: f32) -> Self {
        Self {
            icon,
            size,
            padding: 2.0,
            sense: Sense::hover(),
            look: Look::Plain,
            color: None,
        }
    }

    /// Clickable, still with no background.
    pub fn clickable(mut self) -> Self {
        self.sense = Sense::click();
        self
    }

    /// Clickable with a button's frame.
    pub fn button(mut self) -> Self {
        self.sense = Sense::click();
        self.look = Look::Button;
        self
    }

    /// Clickable and lit while `selected`, as a selectable label.
    pub fn toggle(mut self, selected: bool) -> Self {
        self.sense = Sense::click();
        self.look = Look::Toggle(selected);
        self
    }

    /// The icon's colour; the widget text colour otherwise.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }
}

impl egui::Widget for Icon {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let side = self.size + 2.0 * self.padding;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), self.sense);
        if !ui.is_rect_visible(rect) {
            return response;
        }
        let selected = matches!(self.look, Look::Toggle(true));
        let visuals = ui.style().interact_selectable(&response, selected);
        let lit = match self.look {
            Look::Plain => false,
            Look::Toggle(on) => on || response.hovered(),
            Look::Button => true,
        };
        if lit {
            ui.painter().rect(
                rect,
                visuals.corner_radius,
                visuals.weak_bg_fill,
                visuals.bg_stroke,
                StrokeKind::Inside,
            );
        }
        let color = self.color.unwrap_or_else(|| visuals.text_color());
        paint(
            ui.painter(),
            rect.center(),
            Align2::CENTER_CENTER,
            self.icon,
            self.size,
            color,
        );
        crate::widgets::focus_ring(ui, &crate::Theme::default(), &response, rect);
        response
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn icons_have_their_own_identifier_and_a_font_glyph() {
        let all = crate::icons::ALL;
        let ids: std::collections::HashSet<char> = all.iter().map(|(_, c)| *c).collect();
        assert_eq!(ids.len(), all.len(), "two icons share an identifier");
        for (name, icon) in all {
            let glyph = crate::icons::glyph(*icon);
            assert_ne!(glyph, *icon, "{name} has no glyph");
            assert!(
                ('\u{e000}'..='\u{f8ff}').contains(&glyph),
                "{name}: {glyph:?}"
            );
        }
        assert_eq!(crate::icons::glyph('x'), 'x');
    }
}
