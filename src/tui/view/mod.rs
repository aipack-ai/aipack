// region:    --- modules

mod facade;

mod action_view;
mod config_view;
mod install_view;
mod main_view;
mod popup_view;
mod run_main_view;
mod run_overview;
mod run_tasks_view;
mod runs_nav_view;
mod runs_view;
mod sum_view;
mod support;
mod task_view;

pub use action_view::*;
pub use config_view::*;
pub use install_view::*;
pub use main_view::*;
pub use popup_view::*;
pub use run_main_view::*;
pub use run_overview::*;
pub use run_tasks_view::*;
pub use runs_nav_view::*;
pub use runs_view::*;
pub use sum_view::*;
pub use task_view::*;

pub mod comp;
pub mod style;

// endregion: --- modules
