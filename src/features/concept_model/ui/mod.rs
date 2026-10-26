//! Modular UI components for Concept Model Studio.

pub mod inspector;
pub mod palette;
pub mod quick_create;
pub mod view;

pub use quick_create::{view_quick_create_modal, QuickCreateState};
pub use view::view;
