use crate::Error;
use crate::exec::ExecStatusEvent;
use crate::store::rt_model::RuntimeCtx;
use crate::tui_v1::{PrintEvent, PromptParams};
use derive_more::derive::From;
use std::sync::Arc;

/// HubEvent is sent by any part of the system that wants to share some information with the rest of the system.
/// For now, it is managed by the OutHub, which is a broadcast channel (to allow multiple listeners).
/// The types of events are:
/// - Message: Log message
/// - Error: Error occurred during some actions
///
/// Later, we will probably add the stage event:
/// - Stage(StageEvent): With StageEvent::BeforeAll, ...
/// - and others as they come along
///
/// Note: Also, more context will be added to those events for better reporting and such.
#[derive(Debug, Clone, From)]
pub enum HubEvent {
	Message(Arc<str>),

	InfoShort(Arc<str>),

	Error {
		error: Arc<Error>,
	},

	#[from]
	Executor(ExecStatusEvent),

	// -- Sent by the lua engine "print override"
	LuaPrint(Arc<str>, RuntimeCtx),

	Print(Arc<PrintEvent>),

	Prompt(Arc<PromptParams>),

	// Used to ping the tui2 AppEvent to refresh
	RtModelChange,

	// -- Action event
	// for now, the watches send and event to the hub,
	// which will trigger the app to send it to the executor.
	DoExecRedo,

	// The quit events
	Quit,
}

// region:    --- Convenient

impl HubEvent {
	pub fn info_short(msg: impl Into<String>) -> Self {
		HubEvent::InfoShort(msg.into().into())
	}
}

// endregion: --- Convenient

// region:    --- Froms

impl<T> From<T> for HubEvent
where
	T: Into<PrintEvent>,
{
	fn from(p: T) -> Self {
		HubEvent::Print(Arc::new(p.into()))
	}
}

// Implementing From trait for Event
impl From<String> for HubEvent {
	fn from(s: String) -> Self {
		HubEvent::Message(s.into())
	}
}

impl From<&str> for HubEvent {
	fn from(s: &str) -> Self {
		HubEvent::Message(s.into())
	}
}

impl From<&String> for HubEvent {
	fn from(s: &String) -> Self {
		HubEvent::Message(s.as_str().into())
	}
}

impl From<Error> for HubEvent {
	fn from(e: Error) -> Self {
		HubEvent::Error { error: Arc::new(e) }
	}
}

// endregion: --- Froms
