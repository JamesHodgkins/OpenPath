//! OpenPath UI toolkit: theme and widgets built on egui (scaffolded from `od-ui`).

mod doc_tabs;
pub mod icon_set;
pub mod icons;
pub mod menu;
pub mod ribbon;
pub mod shortcuts;
mod theme;
#[cfg(test)]
mod theme_contrast_tests;
pub mod widgets;

pub use doc_tabs::{document_tabs, DocumentTabEvent};
pub use theme::{icon_family, Colors, Palette, Theme};
