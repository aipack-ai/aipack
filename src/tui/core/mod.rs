// region:    --- Modules

mod app_event_handlers;
mod event;
mod term_reader;
mod tui_loop;

// -- For cherry
mod app_state;
mod ping_timer;
mod tui_impl;

// -- Public
mod types;

// -- Cherry Flatten
pub use app_state::{AppStage, AppState, ConfigTab};
pub use ping_timer::{PingTimerTx, start_ping_timer};
pub use tui_impl::{AppTx, ExitTx, start_tui};
// -- Public flatten
pub use types::*;

// endregion: --- Modules
