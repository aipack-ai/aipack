use crate::agent::{Agent, find_agent};
use crate::exec::cli::RunArgs;
use crate::hub::{HubEvent, get_hub};
use crate::run::{RunRedoCtx, RunTopAgentParams, run_agent};
use crate::runtime::Runtime;
use crate::support::jsons::into_values;
use crate::support::{editor, text};
use crate::types::{FileInfo, RunAgentResponse};
use crate::{Error, Result, term};
use simple_fs::{SEventKind, SPath, list_files, watch};
use tracing::info;

/// Exec for the Run command
/// Might do a single run or a watch
/// Returns (redo_ctx, redo_requested)
pub async fn exec_run(run_args: RunArgs, runtime: Runtime) -> Result<(RunRedoCtx, bool)> {
	// NOTE - We might be able to remove this now. Will test later.
	tokio::task::yield_now().await;

	// -- First exec
	let (redo_ctx, redo_requested) = exec_run_first(run_args, runtime).await?;

	// -- If watch, we start the watch (will be spawned and return immediately)
	if redo_ctx.run_options().base_run_options().watch() {
		exec_run_watch(redo_ctx.clone());
	}

	Ok((redo_ctx, redo_requested))
}

pub async fn exec_run_first(run_args: RunArgs, runtime: Runtime) -> Result<(RunRedoCtx, bool)> {
	let hub = get_hub();

	let cmd_agent_name = &run_args.cmd_agent_name;

	let agent = find_agent(cmd_agent_name, &runtime, None)?;

	let run_options = RunTopAgentParams::new(run_args)?;

	// Open agent if flag is set to open it (with `-o`)
	if run_options.base_run_options().open() {
		let path = SPath::from(agent.file_path());
		if let Err(err) = editor::open_file_auto(&path) {
			info!("Cannot open agent file.\nCause: {err}")
		}
	}

	// set the terminal name if possible
	let agent_win_name = text::truncate_left_with_ellipsis(agent.name(), 22, "..");
	term::set_window_name(&agent_win_name);

	// Run
	match do_run(&run_options, &runtime, &agent).await {
		Ok(run_agent_res) => {
			let redo_requested = run_agent_res.redo_requested;
			return Ok((RunRedoCtx::new(runtime, agent, run_options), redo_requested));
		}
		Err(err) => hub.publish(err).await,
	};

	Ok((RunRedoCtx::new(runtime, agent, run_options), false))
}

/// Redo the exec_run, with its context
/// NOTE: The redo pattern just take one ctx arg, and handle its own error
pub async fn exec_run_redo(run_redo_ctx: &RunRedoCtx) -> Option<RunRedoCtx> {
	let hub = get_hub();

	let runtime = run_redo_ctx.runtime();
	let agent = run_redo_ctx.agent();
	let run_options = run_redo_ctx.run_options();

	// make sure to reload the agent
	let agent = match find_agent(agent.name(), runtime, None) {
		Ok(agent) => agent,
		Err(err) => {
			hub.publish(err).await;
			return None;
		}
	};

	match do_run(run_options, runtime, &agent).await {
		Ok(_) => Some(RunRedoCtx::new(runtime.clone(), agent, run_options.clone())),
		Err(err) => {
			hub.publish(err).await;
			None
		}
	}
}

/// Exec the run watch.
/// NOTE: This is not async, because we want to have it run in parallel
///       so it will spawn it's own tokio task
pub fn exec_run_watch(redo_ctx: RunRedoCtx) {
	tokio::spawn(async move {
		let watcher = match watch(redo_ctx.agent().file_path()) {
			Ok(watcher) => watcher,
			Err(err) => {
				get_hub().publish(Error::from(err)).await;
				return;
			}
		};
		loop {
			// Block until a message is received
			match watcher.rx.recv_async().await {
				Ok(events) => {
					// Process each event in the vector
					// TODO: Here we probably do not need to loop through the event, just check that there is at least one Modify
					for event in events {
						match event.skind {
							SEventKind::Modify => {
								let hub = get_hub();
								hub.publish("\n==== Agent file modified, running command agent again\n").await;
								// We go through the hub -> executor to be exactly as "redo"
								hub.publish(HubEvent::DoExecRedo).await;
							}
							_ => {
								// NOTE: No need to notify for now
							}
						}
					}
				}
				Err(e) => {
					// Handle any errors related to receiving the message
					get_hub().publish(format!("Error receiving event: {e:?}")).await;
					break;
				}
			}
		}
	});
}

/// Do one run
async fn do_run(run_command_options: &RunTopAgentParams, runtime: &Runtime, agent: &Agent) -> Result<RunAgentResponse> {
	let inputs = if let Some(on_inputs) = run_command_options.on_inputs() {
		Some(into_values(on_inputs)?)
	} else if let Some(on_file_globs) = run_command_options.on_file_globs() {
		// -- First, normalize the globs
		// Note: here we add the eventual `./` for relative globs so that it works both ways
		//       when we do a `-f "./src/*.rs"` or `-f "src/*.rs"`
		let on_file_globs: Vec<String> = on_file_globs
			.iter()
			.map(|&glob| {
				if !glob.starts_with('/') && !glob.starts_with("./") {
					format!("./{glob}")
				} else {
					glob.to_string()
				}
			})
			.collect();
		let on_file_globs: Vec<&str> = on_file_globs.iter().map(|s| s.as_str()).collect();

		let files = list_files("./", Some(&on_file_globs), None)?;

		// -- Second, normalize the path relative to workspace_dir
		let wks_dir = runtime.dir_context().aipack_paths().wks_dir()
								  // TODO: Eventually needs to support running agent without workspace
		              .ok_or("Cannot do an 'aip run ...' as no workspace was found.\nDo a 'aip init' in your project folder to initialize a '.aipack/' folder.")?;

		let files: Vec<SPath> = files
			.into_iter()
			.filter_map(|file| {
				let absolute_file = file.canonicalize().ok()?;
				let absolute_file = absolute_file.try_diff(wks_dir).ok()?;
				Some(absolute_file)
			})
			.collect();

		let dir_context = runtime.dir_context();
		let file_infos = files
			.into_iter()
			.map(|f| FileInfo::new(dir_context, f, true))
			.collect::<Vec<_>>();
		Some(into_values(file_infos)?)
	} else {
		None
	};

	let res = run_agent(
		runtime,
		None,
		agent.clone(),
		inputs,
		run_command_options.base_run_options(),
		false,
	)
	.await?;

	Ok(res)
}
