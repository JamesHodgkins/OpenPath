//! Theme tokens (colors, metrics) and their translation to egui's style.
//!
//! Style: dark, flat, Adobe-like. Depth comes from tone, not shadows; a
//! single blue accent marks primary actions, selection and focus.
//! No widget should hard-code a color: read it from `Theme` instead.

use std::sync::Arc;

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Shadow, Stroke,
    TextStyle,
};

/// `0xRRGGBB` to `Color32`.
const fn hex(rgb: u32) -> Color32 {
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Font family holding the Lucide icon font. Draw icons with this family and
/// a codepoint from [`crate::icons`].
pub fn icon_family() -> FontFamily {
    FontFamily::Name("icons".into())
}

/// Bundled fonts, so the app looks identical on every OS: Inter for text
/// (egui's defaults stay as fallback for missing glyphs) and Lucide for icons.
fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Regular.otf"
        ))),
    );
    fonts.font_data.insert(
        "Inter Bold".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/Inter-Bold.ttf"
        ))),
    );
    fonts.font_data.insert(
        "Lucide".into(),
        Arc::new(FontData::from_static(include_bytes!(
            "../assets/fonts/lucide.ttf"
        ))),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter".into());
    // Bold family: the proportional stack with Inter Bold in Inter's place.
    let bold: Vec<String> = fonts.families[&FontFamily::Proportional]
        .iter()
        .map(|n| {
            if n == "Inter" {
                "Inter Bold".into()
            } else {
                n.clone()
            }
        })
        .collect();
    fonts.families.insert(FontFamily::Name("bold".into()), bold);
    fonts.families.insert(icon_family(), vec!["Lucide".into()]);
    fonts
}

/// Colors, organised as four tones of elevation plus a few shared accents.
///
/// Both reference apps (Adobe's dialogs, AutoCAD's ribbon) use the same
/// structure, so every background in the app is one of these four tones:
///
/// | tone          | used for                                                        |
/// |---------------|-----------------------------------------------------------------|
/// | `base`        | canvas, title/status bars, tab rows, input wells (sunken)        |
/// | `surface`     | side panels, inactive tabs, popups                              |
/// | `raised`      | ribbon body, active tabs, cards, buttons                        |
/// | `raised_high` | hover state of raised things                                    |
///
/// The role-named fields below are aliases of those tones; use the role name
/// at the call site so the intent is readable, and never introduce a new
/// background color: pick a tone.
#[derive(Clone, Debug)]
pub struct Colors {
    pub base: Color32,
    pub surface: Color32,
    pub raised: Color32,
    pub raised_high: Color32,

    // Role aliases.
    /// Title bar, document-tab row, status bar, dialog backdrop. (`base`)
    pub window_bg: Color32,
    /// Drawing canvas. (`base`)
    pub canvas_bg: Color32,
    /// Text fields and dropdown wells. (`base`)
    pub input_bg: Color32,
    /// Default panel and popup fill. (`surface`)
    pub panel_bg: Color32,
    /// Properties/layers side panels. (`surface`)
    pub sidebar_bg: Color32,
    /// Ribbon body and active tabs. (`raised`)
    pub ribbon_bg: Color32,
    /// Buttons, cards, headers. (`raised`)
    pub panel_raised: Color32,
    /// Hover fill. (`raised_high`)
    pub hover: Color32,

    /// Outline of inputs and buttons (lighter than the surface it sits on).
    pub border: Color32,
    pub border_strong: Color32,
    /// Separator on a raised surface (ribbon panel dividers): dark, like AutoCAD.
    pub divider: Color32,
    pub pressed: Color32,

    pub accent: Color32,
    pub accent_hover: Color32,
    pub accent_pressed: Color32,
    /// Accent-coloured text on dark fills (command prompt, option letters):
    /// a lighter blue than `accent`, which is too dim for small text.
    pub accent_text: Color32,

    pub text: Color32,
    pub text_weak: Color32,
    pub text_disabled: Color32,
    pub text_on_accent: Color32,
}

impl Colors {
    #[allow(clippy::too_many_arguments)]
    const fn from_tones(
        base: Color32,
        surface: Color32,
        raised: Color32,
        raised_high: Color32,
        border: Color32,
        border_strong: Color32,
        pressed: Color32,
    ) -> Self {
        Self {
            base,
            surface,
            raised,
            raised_high,

            window_bg: base,
            canvas_bg: base,
            input_bg: base,
            panel_bg: surface,
            sidebar_bg: surface,
            ribbon_bg: raised,
            panel_raised: raised,
            hover: raised_high,

            border,
            border_strong,
            divider: base,
            pressed,

            // OpenPath brand accent is blue; exact shade TBD (spec Appendix G).
            accent: hex(0x1565c0),
            accent_hover: hex(0x1976d2),
            accent_pressed: hex(0x0d47a1),
            accent_text: hex(0x82b1ff),

            // Slightly softer than the reference's pure white: light-on-dark
            // text looks heavier than it is.
            text: hex(0xdcdcdc),
            text_weak: hex(0xb4b4b4),
            text_disabled: hex(0x646464),
            text_on_accent: hex(0xffffff),
        }
    }

    /// Neutral grays, measured from Adobe's New Document dialog.
    pub const fn neutral() -> Self {
        Self::from_tones(
            hex(0x252525),
            hex(0x323232),
            hex(0x3e3e3e),
            hex(0x4c4c4c),
            hex(0x474747),
            hex(0x6a6a6a),
            hex(0x585858),
        )
    }

    /// Blue-tinted slate, measured from AutoCAD 2020's dark theme.
    pub const fn slate() -> Self {
        Self::from_tones(
            hex(0x222933),
            hex(0x313946),
            hex(0x3b4453),
            hex(0x4f5a6e),
            hex(0x4a5466),
            hex(0x6b778c),
            hex(0x5d6980),
        )
    }
}
/// Which color palette a [`Theme`] uses. Persisted by key, so keys are stable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Palette {
    /// Neutral grays (Adobe-style).
    #[default]
    Neutral,
    /// Blue-tinted slate (AutoCAD-style).
    Slate,
}

impl Palette {
    pub const ALL: [Palette; 2] = [Palette::Neutral, Palette::Slate];

    pub fn label(self) -> &'static str {
        match self {
            Palette::Neutral => "Neutral gray",
            Palette::Slate => "Blue slate",
        }
    }

    /// Stable identifier for saving to disk.
    pub fn key(self) -> &'static str {
        match self {
            Palette::Neutral => "neutral",
            Palette::Slate => "slate",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == key)
    }
}
#[derive(Clone, Debug)]
pub struct Theme {
    pub colors: Colors,
    /// Corner radius for inputs, buttons and popups.
    pub radius: u8,
    /// Corner radius for pill-shaped outlined buttons.
    pub pill_radius: u8,
    /// Base spacing unit; everything is a multiple of this (8px grid).
    pub unit: f32,
    /// Standard control height.
    pub control_height: f32,
    pub font_size_body: f32,
    /// Field labels, section headers, captions.
    pub font_size_small: f32,
    pub font_size_tab: f32,
    pub font_size_heading: f32,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Neutral gray palette (the default).
    pub const fn dark() -> Self {
        Self::with_colors(Colors::neutral())
    }

    pub const fn with_palette(palette: Palette) -> Self {
        match palette {
            Palette::Neutral => Self::dark(),
            Palette::Slate => Self::slate(),
        }
    }

    /// Blue-tinted slate palette.
    pub const fn slate() -> Self {
        Self::with_colors(Colors::slate())
    }

    const fn with_colors(colors: Colors) -> Self {
        Self {
            colors,
            radius: 4,
            pill_radius: 13,
            unit: 8.0,
            control_height: 26.0,
            font_size_body: 12.5,
            font_size_small: 11.0,
            font_size_tab: 13.0,
            font_size_heading: 15.0,
        }
    }

    /// Install the theme into an egui context. Call once at startup.
    pub fn apply(&self, ctx: &egui::Context) {
        // Always dark; do not follow the OS light/dark setting.
        ctx.options_mut(|o| o.theme_preference = egui::ThemePreference::Dark);
        ctx.set_fonts(fonts());
        ctx.set_style_of(egui::Theme::Dark, self.style());
    }

    pub fn style(&self) -> egui::Style {
        let c = &self.colors;
        let radius = CornerRadius::same(self.radius);
        let mut style = egui::Style::default();

        // Text sizes.
        let proportional = |size| FontId::new(size, FontFamily::Proportional);
        style.text_styles = [
            (TextStyle::Heading, proportional(self.font_size_heading)),
            (TextStyle::Body, proportional(self.font_size_body)),
            (TextStyle::Button, proportional(self.font_size_body)),
            (TextStyle::Small, proportional(self.font_size_small)),
            (
                TextStyle::Monospace,
                FontId::new(self.font_size_body, FontFamily::Monospace),
            ),
        ]
        .into();

        // Spacing (8px grid).
        let u = self.unit;
        let sp = &mut style.spacing;
        sp.item_spacing = egui::vec2(u * 0.75, u * 0.5);
        sp.button_padding = egui::vec2(u * 1.25, u * 0.375);
        sp.interact_size = egui::vec2(u * 3.5, self.control_height);
        sp.window_margin = Margin::same((u * 1.25) as i8);
        sp.menu_margin = Margin::same((u * 0.5) as i8);
        sp.indent = u * 1.5;
        // Always-visible slim scrollbar with a rounded handle.
        sp.scroll = egui::style::ScrollStyle {
            bar_width: 6.0,
            handle_min_length: 24.0,
            dormant_background_opacity: 0.0,
            active_background_opacity: 0.0,
            interact_background_opacity: 0.0,
            dormant_handle_opacity: 0.45,
            active_handle_opacity: 0.7,
            interact_handle_opacity: 0.9,
            ..egui::style::ScrollStyle::solid()
        };

        // Visuals.
        let v = &mut style.visuals;
        v.dark_mode = true;
        v.override_text_color = None;
        v.panel_fill = c.panel_bg;
        v.window_fill = c.panel_bg;
        v.extreme_bg_color = c.input_bg;
        v.faint_bg_color = c.panel_raised;
        v.code_bg_color = c.input_bg;
        v.hyperlink_color = c.accent_hover;
        v.window_stroke = Stroke::new(1.0_f32, c.border);
        v.window_corner_radius = CornerRadius::same(self.radius + 2);
        v.menu_corner_radius = radius;
        v.window_shadow = Shadow {
            offset: [0, 6],
            blur: 18,
            spread: 0,
            color: Color32::from_black_alpha(110),
        };
        v.popup_shadow = v.window_shadow;
        v.selection.bg_fill = c.accent;
        v.selection.stroke = Stroke::new(1.0_f32, c.text_on_accent);

        let w = &mut v.widgets;
        // Static surfaces and labels. `bg_stroke` here draws separators.
        w.noninteractive.bg_fill = c.panel_bg;
        w.noninteractive.weak_bg_fill = c.panel_bg;
        w.noninteractive.bg_stroke = Stroke::new(1.0_f32, c.border);
        w.noninteractive.fg_stroke = Stroke::new(1.0_f32, c.text);
        w.noninteractive.corner_radius = radius;

        // Interactive controls: flat fills, thin border, no expansion.
        let states = [
            (&mut w.inactive, c.panel_raised, c.border, c.text),
            (&mut w.hovered, c.hover, c.border_strong, c.text),
            (&mut w.active, c.pressed, c.border_strong, c.text),
            (&mut w.open, c.hover, c.border_strong, c.text),
        ];
        for (state, fill, border, text) in states {
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.bg_stroke = Stroke::new(1.0_f32, border);
            state.fg_stroke = Stroke::new(1.0_f32, text);
            state.corner_radius = radius;
            state.expansion = 0.0;
        }

        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_keys_round_trip() {
        for palette in Palette::ALL {
            assert_eq!(Palette::from_key(palette.key()), Some(palette));
        }
        assert_eq!(Palette::from_key("nope"), None);
        assert_eq!(Palette::default(), Palette::Neutral);
    }

    #[test]
    fn every_palette_builds_a_style() {
        for palette in Palette::ALL {
            let _ = Theme::with_palette(palette).style();
        }
    }
    /// Applying the theme and running two frames (fonts load on the first
    /// pass) must not panic, and the icon font must produce a glyph.
    #[test]
    fn applies_and_lays_out_icons() {
        let ctx = egui::Context::default();
        Theme::dark().apply(&ctx);
        for _ in 0..2 {
            let _ = ctx.run(egui::RawInput::default(), |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let galley = ui.painter().layout_no_wrap(
                        crate::icons::glyph(crate::icons::FILE_SAVE).to_string(),
                        FontId::new(16.0, icon_family()),
                        Color32::WHITE,
                    );
                    assert!(galley.size().x > 0.0);
                });
            });
        }
    }
}
