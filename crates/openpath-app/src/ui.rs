//! Application shell: toolbar, task sheet (left), Gantt (right), status bar.

use chrono::Local;
use egui::{vec2, Align, Layout, RichText, Sense};
use openpath_core::sample::sample_project;
use openpath_ui::Theme;

use crate::gantt::{self, CRITICAL, HEADER_H, ROW_H};
use crate::model::Plan;

const COL_ID: f32 = 32.0;
const COL_NAME: f32 = 190.0;
const COL_DUR: f32 = 70.0;
const COL_DATE: f32 = 112.0;
const COL_SLACK: f32 = 64.0;
const TABLE_W: f32 = COL_ID + COL_NAME + COL_DUR + COL_DATE * 2.0 + COL_SLACK;

pub struct OpenPathApp {
    theme: Theme,
    plan: Plan,
    selected: Option<usize>,
    px_per_day: f32,
}

impl OpenPathApp {
    pub fn new(ctx: &egui::Context) -> Self {
        let theme = Theme::default();
        theme.apply(ctx);
        OpenPathApp {
            theme,
            plan: Plan::new(sample_project()),
            selected: None,
            px_per_day: 28.0,
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(self.plan.project.name.clone().unwrap_or_default())
                    .size(self.theme.font_size_heading)
                    .strong(),
            );
            ui.separator();
            if ui.button("Add task").clicked() {
                self.selected = Some(self.plan.add_task(self.selected));
            }
            if ui
                .add_enabled(self.selected.is_some(), egui::Button::new("Delete task"))
                .clicked()
            {
                if let Some(i) = self.selected.take() {
                    self.plan.delete_task(i);
                }
            }
            ui.separator();
            ui.label("Zoom");
            ui.add(egui::Slider::new(&mut self.px_per_day, 6.0..=64.0).show_value(false));
        });
    }

    fn sheet(&mut self, ui: &mut egui::Ui) {
        let c = self.theme.colors.clone();
        ui.set_width(TABLE_W);
        ui.spacing_mut().item_spacing = vec2(0.0, 0.0);

        // Header row.
        let (hdr, _) = ui.allocate_exact_size(vec2(TABLE_W, HEADER_H), Sense::hover());
        ui.painter().rect_filled(hdr, 0.0, c.panel_raised);
        let mut x = hdr.min.x;
        for (label, w) in [
            ("ID", COL_ID),
            ("Task name", COL_NAME),
            ("Duration", COL_DUR),
            ("Start", COL_DATE),
            ("Finish", COL_DATE),
            ("Slack", COL_SLACK),
        ] {
            ui.painter().text(
                egui::pos2(x + 6.0, hdr.center().y),
                egui::Align2::LEFT_CENTER,
                label,
                egui::FontId::proportional(self.theme.font_size_small),
                c.text_weak,
            );
            x += w;
        }

        let mut changed_dur: Option<(usize, f64)> = None;
        let mut renamed: Option<(usize, String)> = None;
        let mut clicked: Option<usize> = None;

        for i in 0..self.plan.project.tasks.len() {
            let task = &self.plan.project.tasks[i];
            let sched = task.uid.and_then(|u| self.plan.scheduled(u));
            let critical = sched.is_some_and(|s| s.is_critical);
            let (row, resp) = ui.allocate_exact_size(vec2(TABLE_W, ROW_H), Sense::click());
            if Some(i) == self.selected {
                ui.painter()
                    .rect_filled(row, 0.0, c.accent.gamma_multiply(0.25));
            } else if i % 2 == 1 {
                ui.painter()
                    .rect_filled(row, 0.0, c.surface.gamma_multiply(0.4));
            }
            if resp.clicked() {
                clicked = Some(i);
            }
            let cell = |x0: f32, w: f32| {
                egui::Rect::from_min_size(
                    egui::pos2(row.min.x + x0, row.min.y + 1.0),
                    vec2(w - 2.0, ROW_H - 2.0),
                )
            };
            let text_col = if critical { CRITICAL } else { c.text };

            ui.painter().text(
                egui::pos2(row.min.x + 6.0, row.center().y),
                egui::Align2::LEFT_CENTER,
                task.id.map_or(String::new(), |n| n.to_string()),
                egui::FontId::proportional(self.theme.font_size_small),
                c.text_weak,
            );

            let mut name = task.name.clone().unwrap_or_default();
            let r = ui.put(
                cell(COL_ID, COL_NAME),
                egui::TextEdit::singleline(&mut name)
                    .frame(false)
                    .text_color(text_col),
            );
            if r.changed() {
                renamed = Some((i, name));
            }
            if r.gained_focus() {
                clicked = Some(i);
            }

            let mut days = self.plan.duration_days(task);
            let r = ui.put(
                cell(COL_ID + COL_NAME, COL_DUR),
                egui::DragValue::new(&mut days)
                    .speed(0.1)
                    .range(0.0..=3650.0)
                    .max_decimals(2)
                    .suffix(" d"),
            );
            if r.changed() {
                changed_dur = Some((i, days));
            }

            let fmt = |d: Option<chrono::NaiveDateTime>| {
                d.map_or(String::new(), |d| d.format("%a %d %b %H:%M").to_string())
            };
            let mut cx = COL_ID + COL_NAME + COL_DUR;
            for text in [fmt(task.start), fmt(task.finish)] {
                ui.painter().text(
                    egui::pos2(row.min.x + cx + 6.0, row.center().y),
                    egui::Align2::LEFT_CENTER,
                    text,
                    egui::FontId::proportional(self.theme.font_size_small),
                    c.text,
                );
                cx += COL_DATE;
            }
            if let Some(s) = sched {
                let days = s.total_slack_minutes as f64 / self.plan.minutes_per_day() as f64;
                ui.painter().text(
                    egui::pos2(row.min.x + cx + 6.0, row.center().y),
                    egui::Align2::LEFT_CENTER,
                    format!("{days:.1} d"),
                    egui::FontId::proportional(self.theme.font_size_small),
                    if critical { CRITICAL } else { c.text_weak },
                );
            }
        }

        if let Some(i) = clicked {
            self.selected = Some(i);
        }
        if let Some((i, n)) = renamed {
            self.plan.rename(i, n);
        }
        if let Some((i, d)) = changed_dur {
            self.plan.set_duration_days(i, d);
        }
    }

    fn status_bar(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let p = &self.plan;
            ui.label(format!("{} tasks", p.project.tasks.len()));
            ui.separator();
            ui.label(RichText::new(format!("{} critical", p.critical_count())).color(CRITICAL));
            ui.separator();
            if let Some(f) = p.project.finish_date {
                ui.label(format!("Finish {}", f.format("%a %d %b %Y")));
            }
            for d in &p.output.diagnostics {
                ui.separator();
                ui.label(RichText::new(format!("[{}] {}", d.code, d.message)).color(CRITICAL));
            }
        });
    }
}

impl eframe::App for OpenPathApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| self.toolbar(ui));
        egui::TopBottomPanel::bottom("status").show(ctx, |ui| self.status_bar(ui));

        let today = Local::now().naive_local();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 0.0);
                            ui.allocate_ui_with_layout(
                                vec2(TABLE_W, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    self.sheet(ui);
                                },
                            );
                            egui::ScrollArea::horizontal()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    if let Some(row) = gantt::show(
                                        ui,
                                        &self.theme,
                                        &self.plan,
                                        self.selected,
                                        self.px_per_day,
                                        today,
                                    ) {
                                        self.selected = Some(row);
                                    }
                                });
                        });
                    });
            });
    }
}
