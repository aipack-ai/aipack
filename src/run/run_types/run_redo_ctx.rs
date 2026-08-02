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

	pub fn retryable(&self) -> bool {
		self.inner.retryable
	}

	pub fn with_flow_redo_count(self, flow_redo_count: i32) -> Self {
		let mut inner = (*self.inner).clone();
		inner.flow_redo_count = flow_redo_count;
		inner.run_options = inner.run_options.clone().with_flow_redo_count(flow_redo_count);
		Self { inner: Arc::new(inner) }
	}

	pub fn with_retryable(self, retryable: bool) -> Self {
		let mut inner = (*self.inner).clone();
		inner.retryable = retryable;
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
