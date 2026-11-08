//! Dialogs, group boxes and tab folders: the widgets that hold others.

use egui::{Align2, CornerRadius, FontId, Margin, Rect, Sense, Stroke, Ui};

use super::icon_button;
use crate::{icons::DIALOG_CLOSE, Theme};

/// Modal-style dialog window with a themed header (left-aligned title and a
/// close button), a divider and a padded body. Draggable, not resizable, and
/// centered each time it opens. It is held by its top-left corner, so content
/// that grows or shrinks (a warning appearing) moves the bottom edge only.
/// Sets `*open = false` when closed with the X button or Escape. Returns the
/// closure's result while the dialog is visible.
pub fn dialog<R>(
    ctx: &egui::Context,
    theme: &Theme,
    title: &str,
    open: &mut bool,
    width: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    let c = &theme.colors;
    let radius = theme.radius + 3;
    let frame = egui::Frame::new()
        .fill(c.panel_bg)
        .stroke(Stroke::new(1.0_f32, c.border))
        .corner_radius(CornerRadius::same(radius))
        .shadow(ctx.style().visuals.window_shadow);
    let header_height = 36.0;
    let mut close = false;
    let mut close_rect = Rect::NOTHING;
    let mut header_rect = Rect::NOTHING;

    let window_id = egui::Id::new(title);
    let layer_id = egui::LayerId::new(egui::Order::Middle, window_id);
    let was_visible = ctx.memory(|memory| memory.areas().visible_last_frame(&layer_id));
    let mut window = egui::Window::new(title)
        .title_bar(false)
        .frame(frame)
        .resizable(false)
        .collapsible(false);
    // Its size as last laid out, and the size it was last centered by.
    let size_id = egui::Id::new(("dialog_size", title));
    let centered_id = egui::Id::new(("dialog_centered_by", title));
    let known = ctx.data(|d| d.get_temp::<egui::Vec2>(size_id));
    let centered_by = ctx.data(|d| d.get_temp::<egui::Vec2>(centered_id));
    // Positions are remembered by title, so override the old one whenever a
    // dialog reappears, and keep centering until the size it was centered by
    // is the size it turned out to be (the first guess is corrected by the
    // frame that lays it out). `current_pos` (not an anchor) keeps it
    // draggable once it has settled.
    let settled_id = egui::Id::new(("dialog_settled", title));
    let settled = was_visible && ctx.data(|d| d.get_temp::<bool>(settled_id)) == Some(true);
    if !settled {
        let size = known.unwrap_or(egui::vec2(width, 400.0));
        window = window.current_pos(ctx.screen_rect().center() - size / 2.0);
        ctx.data_mut(|d| {
            d.insert_temp(centered_id, size);
            d.insert_temp(settled_id, was_visible && known == centered_by);
        });
    }

    let result = window
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            ui.set_width(width);

            // The header's row is only reserved here. It is drawn once the body
            // has made the window as wide as it is going to be, so the close
            // button is always at the window's corner, whatever the content.
            let top_left = ui.cursor().min;
            let backdrop = ui.painter().add(egui::Shape::Noop);
            ui.allocate_space(egui::vec2(width, header_height));
            let (_, divider) = ui.allocate_space(egui::vec2(width, 1.0));

            let body = egui::Frame::new()
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = egui::vec2(theme.unit, theme.unit * 0.5);
                    ui.set_width(width - 32.0);
                    add_contents(ui)
                })
                .inner;

            let full = ui.min_rect().width().max(width);
            header_rect = Rect::from_min_size(top_left, egui::vec2(full, header_height));
            ui.painter().set(
                backdrop,
                egui::epaint::RectShape::filled(
                    header_rect,
                    CornerRadius {
                        nw: radius,
                        ne: radius,
                        sw: 0,
                        se: 0,
                    },
                    c.panel_raised,
                ),
            );
            ui.painter().rect_filled(
                Rect::from_min_size(divider.min, egui::vec2(full, 1.0)),
                0.0,
                c.border,
            );
            ui.painter().text(
                header_rect.left_center() + egui::vec2(16.0, 0.0),
                Align2::LEFT_CENTER,
                title,
                FontId::proportional(theme.font_size_body + 0.5),
                c.text,
            );
            let mut corner = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_max(
                        header_rect.min,
                        header_rect.max - egui::vec2(6.0, 0.0),
                    ))
                    .layout(egui::Layout::right_to_left(egui::Align::Center)),
            );
            let button = icon_button(&mut corner, theme, DIALOG_CLOSE, "Close");
            close_rect = button.rect;
            close |= button.clicked();
            body
        })
        .and_then(|response| {
            ctx.data_mut(|d| d.insert_temp(size_id, response.response.rect.size()));
            response.inner
        });

    // Where the header and its close button ended up, for the tests below.
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::new(("dialog_geometry", title)),
            (header_rect, close_rect),
        );
    });
    if close
        || (ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !ctx.memory(|m| m.any_popup_open()))
    {
        *open = false;
    }
    result
}

/// Titled group box: a bordered, rounded frame whose title sits on the top
/// border (like a classic dialog "fieldset"). Fills the available width.
pub fn group_box<R>(
    ui: &mut Ui,
    theme: &Theme,
    title: &str,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> R {
    let c = &theme.colors;
    let galley = ui.painter().layout_no_wrap(
        title.to_owned(),
        FontId::proportional(theme.font_size_body),
        c.text,
    );
    // Room for the title that straddles the top border.
    ui.add_space(galley.size().y / 2.0);
    let inner = egui::Frame::new()
        .stroke(Stroke::new(1.0_f32, c.border))
        .corner_radius(CornerRadius::same(theme.radius))
        .inner_margin(Margin {
            left: 12,
            right: 12,
            top: 14,
            bottom: 10,
        })
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            add_contents(ui)
        });
    let rect = inner.response.rect;
    let pos = egui::pos2(rect.left() + 10.0, rect.top() - galley.size().y / 2.0);
    // Break the border behind the title.
    let gap = Rect::from_min_size(
        pos - egui::vec2(4.0, 0.0),
        galley.size() + egui::vec2(8.0, 0.0),
    );
    ui.painter().rect_filled(gap, 0.0, c.panel_bg);
    ui.painter().galley(pos, galley, c.text);
    inner.inner
}

/// Tab folder: tabs attached to a bordered page. The selected tab shares the
/// page's fill so the two read as one piece. `add_page` receives the selected
/// index. `min_page_height` keeps the dialog from resizing between tabs.
pub fn tabbed_page<R>(
    ui: &mut Ui,
    theme: &Theme,
    selected: &mut usize,
    titles: &[&str],
    min_page_height: f32,
    add_page: impl FnOnce(&mut Ui, usize) -> R,
) -> R {
    let c = &theme.colors;
    let font = FontId::proportional(theme.font_size_body);
    let top = CornerRadius {
        nw: theme.radius,
        ne: theme.radius,
        sw: 0,
        se: 0,
    };
    let spacing = ui.spacing().item_spacing;
    let tab_height = 26.0;

    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let mut tab_rects = Vec::with_capacity(titles.len());
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0;
            for (i, title) in titles.iter().enumerate() {
                let text_w = ui
                    .painter()
                    .layout_no_wrap((*title).to_owned(), font.clone(), c.text)
                    .size()
                    .x;
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(text_w + 24.0, tab_height), Sense::click());
                if response.clicked() {
                    *selected = i;
                }
                if *selected != i {
                    let fill = if response.hovered() {
                        c.panel_raised
                    } else {
                        c.window_bg
                    };
                    ui.painter().rect_filled(rect, top, fill);
                    ui.painter().rect_stroke(
                        rect,
                        top,
                        Stroke::new(1.0_f32, c.border),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        rect.center(),
                        Align2::CENTER_CENTER,
                        title,
                        font.clone(),
                        c.text_weak,
                    );
                }
                tab_rects.push(rect);
            }
        });

        let result = egui::Frame::new()
            .fill(c.panel_bg)
            .stroke(Stroke::new(1.0_f32, c.border))
            .corner_radius(CornerRadius {
                nw: 0,
                ne: theme.radius,
                sw: theme.radius,
                se: theme.radius,
            })
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = spacing;
                ui.set_width(ui.available_width());
                ui.set_min_height(min_page_height);
                add_page(ui, *selected)
            })
            .inner;

        // Selected tab drawn last so it overlaps (and hides) the page's top
        // border beneath it.
        if let Some(rect) = tab_rects.get(*selected) {
            let joined =
                Rect::from_min_max(rect.min, egui::pos2(rect.right(), rect.bottom() + 1.0));
            ui.painter().rect_filled(joined, top, c.panel_bg);
            ui.painter().rect_stroke(
                *rect,
                top,
                Stroke::new(1.0_f32, c.border),
                egui::StrokeKind::Inside,
            );
            ui.painter().line_segment(
                [
                    egui::pos2(rect.left() + 1.0, rect.bottom()),
                    egui::pos2(rect.right() - 1.0, rect.bottom()),
                ],
                Stroke::new(1.5_f32, c.panel_bg),
            );
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                titles[*selected],
                font.clone(),
                c.text,
            );
        }
        result
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run a dialog `frames` times in a fresh context and say where its
    /// window, header and close button are.
    fn lay_out(declared: f32, content: f32) -> (Rect, Rect, Rect) {
        let ctx = egui::Context::default();
        let theme = Theme::default();
        // Installs the fonts, the icon font among them.
        theme.apply(&ctx);
        let mut open = true;
        for _ in 0..3 {
            let raw = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600.0, 1000.0),
                )),
                ..Default::default()
            };
            let _ = ctx.run(raw, |ctx| {
                dialog(ctx, &theme, "Test dialog", &mut open, declared, |ui| {
                    ui.allocate_space(egui::vec2(content, 20.0));
                });
            });
        }
        let window = ctx
            .memory(|m| m.area_rect(egui::Id::new("Test dialog")))
            .expect("the dialog was shown");
        let (header, close) = ctx
            .data(|d| d.get_temp::<(Rect, Rect)>(egui::Id::new(("dialog_geometry", "Test dialog"))))
            .expect("the dialog says where its header is");
        (window, header, close)
    }

    /// The header runs the whole width of the window, and the close button
    /// sits at its right end, whatever the content does to the window's width.
    fn assert_header_fits(declared: f32, content: f32) {
        let (window, header, close) = lay_out(declared, content);
        let (w, h, x) = (window.right(), header.right(), close.right());
        assert!(
            (w - h).abs() <= 2.0,
            "declared {declared}, content {content}: the window ends at {w} but its header at {h}"
        );
        assert!(
            (h - x) > 0.0 && (h - x) <= 12.0,
            "declared {declared}, content {content}: the close button ends at {x}, the header at {h}"
        );
        assert!(
            (window.left() - header.left()).abs() <= 2.0,
            "the header starts where the window does"
        );
    }

    #[test]
    fn the_close_button_sits_at_the_corner_when_the_content_is_narrower() {
        assert_header_fits(420.0, 100.0);
    }

    #[test]
    fn it_does_when_the_content_just_fits() {
        assert_header_fits(420.0, 420.0 - 32.0);
    }

    /// Where the top of the dialog is after laying out `content` tall.
    fn top_with_height(ctx: &egui::Context, theme: &Theme, content: f32) -> f32 {
        let raw = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600.0, 1000.0),
            )),
            ..Default::default()
        };
        let mut open = true;
        let _ = ctx.run(raw, |ctx| {
            dialog(ctx, theme, "Test dialog", &mut open, 420.0, |ui| {
                ui.allocate_space(egui::vec2(100.0, content));
            });
        });
        ctx.memory(|m| m.area_rect(egui::Id::new("Test dialog")))
            .expect("the dialog was shown")
            .top()
    }

    #[test]
    fn the_top_stays_put_when_the_content_grows_or_shrinks() {
        let ctx = egui::Context::default();
        let theme = Theme::default();
        theme.apply(&ctx);
        for _ in 0..3 {
            top_with_height(&ctx, &theme, 200.0);
        }
        let before = top_with_height(&ctx, &theme, 200.0);
        let taller = top_with_height(&ctx, &theme, 240.0);
        let shorter = top_with_height(&ctx, &theme, 160.0);
        assert!((before - taller).abs() < 0.5, "{before} then {taller}");
        assert!((before - shorter).abs() < 0.5, "{before} then {shorter}");
    }

    #[test]
    fn it_is_centered_once_it_has_been_laid_out() {
        let ctx = egui::Context::default();
        let theme = Theme::default();
        theme.apply(&ctx);
        for _ in 0..4 {
            top_with_height(&ctx, &theme, 200.0);
        }
        let window = ctx
            .memory(|m| m.area_rect(egui::Id::new("Test dialog")))
            .expect("the dialog was shown");
        assert!((window.center().x - 800.0).abs() < 2.0, "{window:?}");
        assert!((window.center().y - 500.0).abs() < 2.0, "{window:?}");
    }

    #[test]
    fn it_still_does_when_the_content_is_wider_than_the_declared_width() {
        assert_header_fits(420.0, 600.0);
        assert_header_fits(420.0, 1000.0);
    }
}
