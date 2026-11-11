//! Custom-painted Gantt chart (timescale, bars, milestones, dependency links).

use chrono::{Datelike, NaiveDateTime};
use egui::{pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Sense, Shape, Stroke};
use openpath_core::{RelationType, TaskUid};
use openpath_ui::Theme;

use crate::model::Plan;
use crate::timeline::{is_weekend, week_start, Timeline};

pub const ROW_H: f32 = 24.0;
pub const HEADER_H: f32 = 40.0;
const BAR_H: f32 = 12.0;
pub const CRITICAL: Color32 = Color32::from_rgb(0xE5, 0x53, 0x4B);

/// Paint the chart. Returns the row clicked, if any.
pub fn show(
    ui: &mut egui::Ui,
    theme: &Theme,
    plan: &Plan,
    selected: Option<usize>,
    px_per_day: f32,
    today: NaiveDateTime,
) -> Option<usize> {
    let c = &theme.colors;
    let start = plan.project.start_date.unwrap_or(today).date();
    let origin = week_start(start) - chrono::Duration::days(7);
    let tl = Timeline::new(origin, px_per_day);
    let last = plan.project.finish_date.map_or(start, |f| f.date());
    let width = tl.width_for(last, 21);
    let rows = plan.project.tasks.len();
    let size = vec2(width, HEADER_H + rows as f32 * ROW_H);

    let (resp, painter) = ui.allocate_painter(size, Sense::click());
    let top_left = resp.rect.min;
    let at = |x: f32, y: f32| pos2(top_left.x + x, top_left.y + y);
    let small = FontId::proportional(theme.font_size_small);
    let ndays = (width / px_per_day).ceil() as i64;
    let visible = ui.clip_rect();

    painter.rect_filled(resp.rect, 0.0, c.canvas_bg);

    // Row backgrounds.
    for i in 0..rows {
        let y = HEADER_H + i as f32 * ROW_H;
        let r = Rect::from_min_size(at(0.0, y), vec2(width, ROW_H));
        if Some(i) == selected {
            painter.rect_filled(r, 0.0, c.accent.gamma_multiply(0.25));
        } else if i % 2 == 1 {
            painter.rect_filled(r, 0.0, c.surface.gamma_multiply(0.4));
        }
    }

    // Timescale + day grid.
    let header_bg = Rect::from_min_size(top_left, vec2(width, HEADER_H));
    painter.rect_filled(header_bg, 0.0, c.panel_raised);
    for day in tl.days(ndays) {
        let x = tl.day_x(day);
        if at(x, 0.0).x > visible.max.x || at(x + px_per_day, 0.0).x < visible.min.x {
            continue;
        }
        if is_weekend(day) {
            let r = Rect::from_min_size(at(x, HEADER_H), vec2(px_per_day, rows as f32 * ROW_H));
            painter.rect_filled(r, 0.0, Color32::from_black_alpha(46));
        }
        let monday = day.weekday().num_days_from_monday() == 0;
        painter.line_segment(
            [
                at(x, if monday { 0.0 } else { HEADER_H / 2.0 }),
                at(x, size.y),
            ],
            Stroke::new(
                1.0_f32,
                if monday {
                    c.border_strong
                } else {
                    c.divider.gamma_multiply(0.6)
                },
            ),
        );
        if monday {
            painter.text(
                at(x + 4.0, 4.0),
                Align2::LEFT_TOP,
                day.format("%d %b %Y").to_string(),
                small.clone(),
                c.text,
            );
        }
        if px_per_day >= 16.0 {
            let letter =
                ["M", "T", "W", "T", "F", "S", "S"][day.weekday().num_days_from_monday() as usize];
            painter.text(
                at(x + px_per_day / 2.0, HEADER_H * 0.75),
                Align2::CENTER_CENTER,
                if px_per_day >= 28.0 {
                    format!("{letter} {}", day.day())
                } else {
                    letter.to_string()
                },
                small.clone(),
                c.text_weak,
            );
        }
    }
    painter.line_segment(
        [at(0.0, HEADER_H), at(width, HEADER_H)],
        Stroke::new(1.0_f32, c.border_strong),
    );

    // Today marker.
    let tx = tl.x(today);
    if tx >= 0.0 && tx <= width {
        painter.line_segment(
            [at(tx, HEADER_H), at(tx, size.y)],
            Stroke::new(1.0_f32, c.accent_text),
        );
    }

    // Geometry of each task row, keyed by uid.
    let row_of: std::collections::HashMap<TaskUid, usize> = plan
        .project
        .tasks
        .iter()
        .enumerate()
        .filter_map(|(i, t)| Some((t.uid?, i)))
        .collect();
    let mid_y = |row: usize| HEADER_H + row as f32 * ROW_H + ROW_H / 2.0;

    // Dependency links under the bars.
    let link = Stroke::new(1.0_f32, c.text_weak);
    for d in &plan.project.dependencies {
        let (Some(&pi), Some(&si)) = (row_of.get(&d.predecessor), row_of.get(&d.successor)) else {
            continue;
        };
        let (Some(p), Some(s)) = (plan.scheduled(d.predecessor), plan.scheduled(d.successor))
        else {
            continue;
        };
        let (from_t, to_t) = match d.relation_type {
            RelationType::FinishToStart => (p.early_finish, s.early_start),
            RelationType::StartToStart => (p.early_start, s.early_start),
            RelationType::FinishToFinish => (p.early_finish, s.early_finish),
            RelationType::StartToFinish => (p.early_start, s.early_finish),
        };
        let from_end = matches!(
            d.relation_type,
            RelationType::FinishToStart | RelationType::FinishToFinish
        );
        let to_start = matches!(
            d.relation_type,
            RelationType::FinishToStart | RelationType::StartToStart
        );
        let (fx, tx2) = (tl.x(from_t), tl.x(to_t));
        let (py, sy) = (mid_y(pi), mid_y(si));
        let out_x = if from_end { fx + 6.0 } else { fx - 6.0 };
        let in_x = if to_start { tx2 - 6.0 } else { tx2 + 6.0 };
        let (elbow_x, elbow2_x) = (out_x, in_x);
        // Route via the gap between the two rows so links never run through a bar.
        let gap_y = if si > pi {
            HEADER_H + (pi + 1) as f32 * ROW_H
        } else {
            HEADER_H + pi as f32 * ROW_H
        };
        let pts = [
            at(fx, py),
            at(elbow_x, py),
            at(elbow_x, gap_y),
            at(elbow2_x, gap_y),
            at(elbow2_x, sy),
            at(tx2, sy),
        ];
        painter.add(Shape::line(pts.to_vec(), link));
        let dir = if to_start { 1.0 } else { -1.0 };
        let tip = at(tx2, sy);
        painter.add(Shape::convex_polygon(
            vec![
                tip,
                tip + vec2(-4.0 * dir, -3.0),
                tip + vec2(-4.0 * dir, 3.0),
            ],
            c.text_weak,
            Stroke::NONE,
        ));
    }

    // Bars and milestones.
    for (i, t) in plan.project.tasks.iter().enumerate() {
        let Some(s) = t.uid.and_then(|u| plan.scheduled(u)) else {
            continue;
        };
        let color = if s.is_critical { CRITICAL } else { c.accent };
        let y = mid_y(i);
        if t.is_milestone {
            let p = at(tl.x(s.early_start), y);
            painter.add(Shape::convex_polygon(
                vec![
                    p + vec2(0.0, -7.0),
                    p + vec2(7.0, 0.0),
                    p + vec2(0.0, 7.0),
                    p + vec2(-7.0, 0.0),
                ],
                color,
                Stroke::new(1.0_f32, Color32::BLACK.gamma_multiply(0.5)),
            ));
        } else {
            let x0 = tl.x(s.early_start);
            let x1 = tl.x(s.early_finish).max(x0 + 2.0);
            let r = Rect::from_min_max(at(x0, y - BAR_H / 2.0), at(x1, y + BAR_H / 2.0));
            painter.rect_filled(r, 2.0, color);
            painter.rect_stroke(
                r,
                2.0,
                Stroke::new(1.0_f32, Color32::BLACK.gamma_multiply(0.4)),
                egui::StrokeKind::Inside,
            );
            if let Some(pc) = t.percent_complete.filter(|p| *p > 0.0) {
                let done = Rect::from_min_max(
                    r.min,
                    pos2(r.min.x + r.width() * (pc / 100.0) as f32, r.max.y),
                );
                painter.rect_filled(
                    done.shrink2(vec2(0.0, 3.0)),
                    1.0,
                    Color32::from_white_alpha(150),
                );
            }
        }
    }

    resp.clicked()
        .then(|| resp.interact_pointer_pos())
        .flatten()
        .and_then(|p| row_at(p, top_left, rows))
}

fn row_at(p: Pos2, top_left: Pos2, rows: usize) -> Option<usize> {
    let y = p.y - top_left.y - HEADER_H;
    (y >= 0.0)
        .then(|| (y / ROW_H) as usize)
        .filter(|&i| i < rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_hit_testing() {
        let tl = pos2(100.0, 50.0);
        assert_eq!(row_at(pos2(120.0, 50.0 + HEADER_H + 1.0), tl, 3), Some(0));
        assert_eq!(
            row_at(pos2(120.0, 50.0 + HEADER_H + ROW_H * 2.5), tl, 3),
            Some(2)
        );
        assert_eq!(
            row_at(pos2(120.0, 50.0 + HEADER_H + ROW_H * 3.5), tl, 3),
            None
        );
        assert_eq!(row_at(pos2(120.0, 50.0 + 5.0), tl, 3), None);
    }
}
