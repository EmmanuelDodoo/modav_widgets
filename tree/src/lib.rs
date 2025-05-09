//! Vertical tree widget capable of collapsing
//!
//! Functionality:
//!
//! - Left click: Select and toggle collapse.
//! - Tab: Moves the current selection `down` skipping any collapsed sections.
//! - Shift + Tab: Moves the current selection `up` skipping any collapsed sections.
//! - ArrowDown: Moves the current selection `down`, expanding collapsed sections.
//! - ArrowUp: Moves the current selection `up`, expanding collapsed sections.
//! - Enter: Toggles collapse on the current selection.

pub mod base;
mod style;
mod tree;

pub use style::*;
pub use tree::*;
