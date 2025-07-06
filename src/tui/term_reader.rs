use super::AppTx;
use crate::Result;
use crossterm::event::EventStream;
use futures::{FutureExt, StreamExt};
use futures_timer::Delay;
use std::time::Duration;
use tokio::select;
use tokio::task::JoinHandle;

pub fn run_term_read(app_tx: AppTx) -> Result<JoinHandle<()>> {
	let handle = tokio::spawn(async move {
		let mut reader = EventStream::new();

		loop {
			let delay = Delay::new(Duration::from_millis(200)).fuse();
			let event = reader.next().fuse();

			select! {
				_ = delay => {  },
				maybe_event = event => {
					match maybe_event {
						Some(Ok(event)) => {
							if let Err(err) = app_tx.send(event).await {
								println!("run_term_read - Cannot send app_txt.send. Cause: {err}");
								break;
							}
						}
						Some(Err(e)) => println!("Error: {e:?}\r"),
						None => break,
					}
				}
			};
		}
	});

	Ok(handle)
}
