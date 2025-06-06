use crate::dir_context::PackDir;
use derive_more::From;
use genai::ModelIden;
use std::collections::HashSet;

#[derive(Debug, From)]
pub enum PrintEvent {
	#[from]
	PackList(Vec<PackDir>),

	/// Single line info
	InfoShort(String),

	ApiKeysStatus {
		all_keys: &'static [&'static str],
		available_keys: HashSet<String>,
	},

	ApiKeyEnvMissing {
		model_iden: ModelIden,
		env_name: String,
	},

	GenericErrorMsg(String),
}
