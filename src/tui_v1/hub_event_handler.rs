use crate::exec::{ExecActionEvent, ExecStatusEvent, ExecutorTx};
use crate::hub::HubEvent;
use crate::term::safer_println;
use crate::tui_v1::prompter::prompt;
use crate::tui_v1::{ExitTx, PrintEvent, handle_print, tui_elem};
use crate::{Error, Result};

pub async fn handle_hub_event(
	event: HubEvent,
	exec_sender: &ExecutorTx,
	exit_tx: &ExitTx,
	interactive: bool,
) -> Result<()> {
	match event {
		HubEvent::Message(msg) => {
			safer_println(&format!("{msg}"), interactive);
		}

		HubEvent::InfoShort(msg) => {
			handle_print(PrintEvent::InfoShort(msg.to_string()).into(), interactive);
		}

		HubEvent::Error { error } => match &*error {
			Error::GenAIEnvKeyMissing { model_iden, env_name } => handle_print(
				PrintEvent::ApiKeyEnvMissing {
					model_iden: model_iden.clone(),
					env_name: env_name.to_string(),
				}
				.into(),
				interactive,
			),
			other => handle_print(PrintEvent::GenericErrorMsg(other.to_string()).into(), interactive),
		},

		HubEvent::RtModelChange => (), // do nothing in legacy tui.

		HubEvent::LuaPrint(text) => safer_println(&text, interactive),

		HubEvent::Print(print_event) => handle_print(print_event, interactive),

		HubEvent::Prompt(params) => prompt(params).await?,

		HubEvent::Executor(exec_event) => match (exec_event, interactive) {
			(ExecStatusEvent::RunEnd, true) => tui_elem::print_bottom_bar(),
			(ExecStatusEvent::EndExec, false) => exit_tx.send(()).await?,
			(_, _) => (),
		},
		HubEvent::DoExecRedo => exec_sender.send(ExecActionEvent::Redo).await,
		HubEvent::Quit => {
			exit_tx.send(()).await?;
		}
	}

	Ok(())
}
