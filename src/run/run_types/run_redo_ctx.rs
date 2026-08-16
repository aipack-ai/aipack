use crate::agent::Agent;
use crate::model::Id;
use crate::run::RunTopAgentParams;
use crate::runtime::Runtime;
use std::sync::Arc;

// #[derive(From)]
// pub enum RedoCtx {
// 	RunRedoCtx(Arc<RunRedoCtx>),
// }

// impl From<RunRedoCtx> for RedoCtx {
// 	fn from(run_redo_ctx: RunRedoCtx) -> Self {
// 		RedoCtx::RunRedoCtx(run_redo_ctx.into())
// 	}
// }

// impl RedoCtx {
// 	pub fn get_agent(&self) -> Option<&Agent> {
// 		match self {
// 			RedoCtx::RunRedoCtx(redo_ctx) => Some(redo_ctx.agent()),
// 		}
// 	}
// }

#[derive(Debug, Clone)]
pub struct RunRedoCtx {
	inner: Arc<CtxInner>,
}

/// constructor
impl RunRedoCtx {
	pub fn new(
		runtime: Runtime,
		agent: Agent,
		run_options: RunTopAgentParams,
		redo_requested: bool,
		flow_redo_count: i32,
	) -> Self {
		Self {
			inner: Arc::new(CtxInner {
				runtime,
				agent,
				run_options,
				run_id: None,
				loop_id: None,
				redo_requested,
				flow_redo_count,
				retryable: false,
			}),
		}
	}

	pub fn with_identity(
		runtime: Runtime,
		agent: Agent,
		run_options: RunTopAgentParams,
		run_id: Id,
		loop_id: Option<Id>,
		redo_requested: bool,
		flow_redo_count: i32,
	) -> Self {
		Self {
			inner: Arc::new(CtxInner {
				runtime,
				agent,
				run_options,
				run_id: Some(run_id),
				loop_id,
				redo_requested,
				flow_redo_count,
				retryable: false,
			}),
		}
	}
}

/// getters
impl RunRedoCtx {
	pub fn runtime(&self) -> &Runtime {
		&self.inner.runtime
	}

	pub fn agent(&self) -> &Agent {
		&self.inner.agent
	}

	#[allow(dead_code)]
	pub fn run_id(&self) -> Option<Id> {
		self.inner.run_id
	}

	pub fn loop_id(&self) -> Option<Id> {
		self.inner.loop_id
	}

	pub fn run_options(&self) -> &RunTopAgentParams {
		&self.inner.run_options
	}

	pub fn redo_requested(&self) -> bool {
		self.inner.redo_requested
	}

	pub fn flow_redo_count(&self) -> i32 {
		self.inner.flow_redo_count
	}

	#[allow(dead_code)]
	pub fn retryable(&self) -> bool {
		self.inner.retryable
	}

	pub fn with_flow_redo_count(self, flow_redo_count: i32) -> Self {
		let mut inner = (*self.inner).clone();
		inner.flow_redo_count = flow_redo_count;
		inner.run_options = inner.run_options.clone().with_flow_redo_count(flow_redo_count);
		Self { inner: Arc::new(inner) }
	}

	#[allow(dead_code)]
	pub fn with_retryable(self, retryable: bool) -> Self {
		let mut inner = (*self.inner).clone();
		inner.retryable = retryable;
		Self { inner: Arc::new(inner) }
	}

	#[allow(dead_code)]
	pub fn with_loop_id(self, loop_id: Option<Id>) -> Self {
		let mut inner = (*self.inner).clone();
		inner.loop_id = loop_id;
		Self { inner: Arc::new(inner) }
	}

	#[allow(dead_code)]
	pub fn with_redo_requested(self, redo_requested: bool) -> Self {
		let mut inner = (*self.inner).clone();
		inner.redo_requested = redo_requested;
		Self { inner: Arc::new(inner) }
	}

	#[allow(dead_code)]
	pub fn reset_loop_and_redo(self) -> Self {
		let mut inner = (*self.inner).clone();
		inner.loop_id = None;
		inner.redo_requested = false;
		inner.flow_redo_count = 0;
		inner.run_options = inner.run_options.clone().with_flow_redo_count(0);
		inner.retryable = false;
		Self { inner: Arc::new(inner) }
	}
}

/// A Context that hold the information to redo this run
#[derive(Debug, Clone)]
struct CtxInner {
	runtime: Runtime,
	agent: Agent,
	run_options: RunTopAgentParams,
	#[allow(dead_code)]
	run_id: Option<Id>,
	loop_id: Option<Id>,
	redo_requested: bool,
	flow_redo_count: i32,
	retryable: bool,
}

// region:    --- Tests

#[cfg(test)]
mod tests {
	type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

	use super::*;
	use clap::Parser;

	#[tokio::test]
	async fn test_run_redo_ctx_reset_loop_and_redo() -> Result<()> {
		// -- Setup & Fixtures
		let runtime = Runtime::new_test_runtime_sandbox_01().await?;
		let agent = Agent::mock_from_content(
			r#"
# Data
```lua
return "ok"
```
"#,
		)?;
		let run_args = crate::exec::cli::RunArgs::try_parse_from(["aip", "test-agent"])?;
		let run_options = RunTopAgentParams::new(run_args)?;

		let ctx = RunRedoCtx::with_identity(
			runtime,
			agent,
			run_options,
			1.into(),
			Some(10.into()),
			true,
			3,
		);

		// -- Exec
		let reset_ctx = ctx.reset_loop_and_redo();

		// -- Check
		assert!(reset_ctx.loop_id().is_none());
		assert!(!reset_ctx.redo_requested());
		assert_eq!(reset_ctx.flow_redo_count(), 0);
		assert!(!reset_ctx.retryable());

		Ok(())
	}
}

// endregion: --- Tests
