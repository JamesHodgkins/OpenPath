//! OpenPath desktop app: task sheet + Gantt chart over the scheduling engine.

#![forbid(unsafe_code)]

mod gantt;
mod model;
mod timeline;
mod ui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("OpenPath")
            .with_inner_size([1200.0, 700.0])
            .with_min_inner_size([720.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "OpenPath",
        options,
        Box::new(|cc| Ok(Box::new(ui::OpenPathApp::new(&cc.egui_ctx)))),
    )
}
