//! WCAG 2.2 contrast of the theme's text and focus colours on the surfaces
//! they are drawn on, in both palettes. Normal text needs 4.5:1, and what
//! marks keyboard focus 3:1. The measured ratios are in `ui_plan.md`.

use egui::Color32;

use crate::theme::Colors;

fn channel(value: u8) -> f64 {
    let c = f64::from(value) / 255.0;
    if c <= 0.039_28 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn luminance(c: Color32) -> f64 {
    0.2126 * channel(c.r()) + 0.7152 * channel(c.g()) + 0.0722 * channel(c.b())
}

pub fn contrast(a: Color32, b: Color32) -> f64 {
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

fn palettes() -> [(&'static str, Colors); 2] {
    [("neutral", Colors::neutral()), ("slate", Colors::slate())]
}

#[test]
fn the_reference_ratios_are_right() {
    assert!((contrast(Color32::WHITE, Color32::BLACK) - 21.0).abs() < 1e-9);
    assert!((contrast(Color32::WHITE, Color32::WHITE) - 1.0).abs() < 1e-9);
}

#[test]
fn text_is_readable_on_the_page_panels_and_cards() {
    for (name, c) in palettes() {
        for (surface_name, surface) in [
            ("base", c.base),
            ("surface", c.surface),
            ("raised", c.raised),
        ] {
            for (text_name, text) in [
                ("text", c.text),
                ("text_weak", c.text_weak),
                ("accent_text", c.accent_text),
            ] {
                let ratio = contrast(text, surface);
                assert!(
                    ratio >= 4.5,
                    "{name}: {text_name} on {surface_name} is {ratio:.2}"
                );
            }
        }
    }
}

#[test]
fn white_on_the_accent_and_the_focus_ring_on_cards_are_clear_enough() {
    for (name, c) in palettes() {
        let on_accent = contrast(c.text_on_accent, c.accent);
        assert!(on_accent >= 4.5, "{name}: text on accent {on_accent:.2}");
        for (surface_name, surface) in [("base", c.base), ("raised", c.raised)] {
            let ring = contrast(c.accent_text, surface);
            assert!(
                ring >= 3.0,
                "{name}: focus ring on {surface_name} {ring:.2}"
            );
        }
    }
}
