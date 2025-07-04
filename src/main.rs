// region:    --- Modules

mod agent;
mod dir_context;
mod error;
mod event;
mod exec;
mod hub;
mod run;
mod runtime;
mod script;
mod store;
mod support;
mod term;
mod tui_v1;
mod types;

#[cfg(test)]
mod _test_support;

use crate::exec::Executor;
use crate::exec::cli::CliArgs;
use crate::hub::{HubEvent, get_hub};
use crate::store::OnceModelManager;
use crate::tui_v1::TuiAppV1;
use clap::{Parser, crate_version};
use error::{Error, Result};

pub static VERSION: &str = crate_version!();

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
	// -- Command arguments
	let args = CliArgs::parse(); // Will fail early, but that’s okay.

	// -- The OnceModelManager
	// This way, ModelManager is only created when needed
	let once_mm = OnceModelManager;

	// -- Start executor
	let executor = Executor::new(once_mm);
	let exec_sender = executor.sender();
	// TODO: Probably want to move the spawn inside executor.start
	tokio::spawn(async move {
		// NOTE: This will consume the excecutor (make sure to get exec_sender before start)
		if let Err(err) = executor.start().await {
			let hub = get_hub();
			hub.publish(HubEvent::Error { error: err.into() }).await;
			hub.publish(HubEvent::Quit).await;
		}
	});

	// -- Start UI
	let tui = TuiAppV1::new(exec_sender);
	// This will wait until all done
	tui.start_with_args(args).await?;

	// -- End
	// Tokio wait for 100ms
	// Note: This will allow the hub message to drain.
	//       This is a short-term trick before we get the whole TUI app.
	// Note: Might have a more reliable way.
	tokio::time::sleep(std::time::Duration::from_millis(100)).await;
	println!("\n---- Until next time, happy coding! ----");

	Ok(())
}
