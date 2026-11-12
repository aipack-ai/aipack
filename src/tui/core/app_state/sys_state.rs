use crate::Result;
use crate::error::Error;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind, get_current_pid};

pub struct SysState {
	pid: Pid,
	sys: System,
}

/// Constructor & Refresher
impl SysState {
	pub fn new() -> Result<Self> {
		let pid = get_current_pid().map_err(|err| format!("Fail to get current process id.\nCause: {err}"))?;
		let sys = System::new().map_err(Error::custom)?;
		Ok(SysState { pid, sys })
	}

	fn refresh(&mut self) {
		self.sys.refresh_processes_specifics(
			ProcessesToUpdate::Some(&[self.pid]),
			true,
			ProcessRefreshKind::nothing()
				.with_memory()
				.with_cpu()
				.with_disk_usage()
				.with_exe(UpdateKind::OnlyIfNotSet)
				.with_tasks(),
		);
	}
}

/// Getters
impl SysState {
	/// Returns the current memory in bytes, and cpu in f32
	pub fn memory_and_cpu(&mut self) -> (u64, f64) {
		self.refresh();
		if let Some(process) = self.sys.process(self.pid) {
			(process.memory(), process.cpu_usage() as f64)
		} else {
			(0, 0.)
		}
	}
}
