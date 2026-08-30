//! Module about AI support functions.

use crate::Result;
use genai::adapter::AdapterKind;
use genai::chat::ChatOptions;
use genai::resolver::AuthData;
use genai::{Client, ModelIden};

pub fn new_genai_client() -> Result<genai::Client> {
	let options = ChatOptions::default().with_normalize_reasoning_content(true);
	let client = Client::builder()
		.with_chat_options(options)
		.with_auth_resolver_fn(|model: ModelIden| {
			// -- Get the key_name, if none, then, could be ollama, so return None
			let Some(key_name) = model.adapter_kind.default_key_env_name() else {
				return Ok(None);
			};

			// -- Try to get it from the env variable
			let key_from_env = std::env::var(key_name).ok();

			if let Some(key) = key_from_env {
				Ok(Some(AuthData::from_single(key)))
			}
			// -- Otherwise, get it with keyring
			else {
				// NOTE: For now, we are disabling Mac keychain support.
				//       Apple seems to have deprecated this and is promoting passwords,
				//       which do not appear to be supported by keyring.
				//
				// NOTE: We might bring this feature back, but with a settings in the config or somewhere
				//
				// #[cfg(target_os = "macos")]
				// {
				// 	use crate::support::cred::get_or_prompt_api_key;
				// 	// TODO: need to pass the model
				// 	let key = get_or_prompt_api_key(key_name);

				// 	match key {
				// 		Ok(key) => Ok(Some(AuthData::from_single(key))),
				// 		Err(err) => {
				// 			// Eventually need to handle his
				// 			Err(genai::resolver::Error::Custom(err.to_string()))
				// 		}
				// 	}
				// }
				// #[cfg(not(target_os = "macos"))]
				{
					// NOTE: for now, for omlx, we allow that if we did not find the env, it might be ok.
					if model.adapter_kind == AdapterKind::Omlx {
						Ok(None)
					} else {
						Err(genai::resolver::Error::ApiKeyEnvNotFound {
							env_name: key_name.to_string(),
						})
					}
				}
			}
		})
		.build()?;

	Ok(client)
}
