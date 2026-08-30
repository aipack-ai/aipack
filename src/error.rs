use derive_more::From;
use derive_more::derive::Display;
use genai::ModelIden;
use tokio::runtime::TryCurrentError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, From, Display)]
#[display("{self:?}")]
pub enum Error {
	// -- Cli Command
	#[display("Command Agent not found at: {_0}")]
	CommandAgentNotFound(String),

	#[display("User cancelled command.")]
	UserInterrupted,

	// -- Agent
	#[display("Model is missing for agent path: {agent_path}")]
	ModelMissing {
		agent_path: String,
	},

	// -- Config
	#[display("Config invalid (config path: {path})\n  reason: {reason}")]
	Config {
		path: String,
		reason: String,
	},

	// -- Pack
	#[display("Pack Identity '{origin_path}' is not valid.\nCause: {cause}")]
	InvalidPackIdentity {
		origin_path: String,
		cause: String,
	},
	#[display("Pack namespace '{namespace}' is not valid.\nCause: {cause}")]
	InvalidNamespace {
		namespace: String,
		cause: &'static str,
	},
	#[display("Pack Name '{name}' is not valid.\nCause: {cause}")]
	InvalidPackName {
		name: String,
		cause: &'static str,
	},

	// -- Channel
	#[display("Send on channel '{name}' fail.\nCause: {cause}")]
	ChannelTx {
		name: &'static str,
		cause: String,
	},
	#[display("Recieve on channel '{name}' fail.\nCause: {cause}")]
	ChannelRx {
		name: &'static str,
		cause: String,
	},

	// -- WebC
	#[display("Fail to download url '{url}' to file '{dest_file}'.\nCause: {cause}")]
	FailToDownload {
		url: String,
		dest_file: String,
		cause: String,
	},

	// -- Packer & Installer
	#[display("pack.toml file is missing at '{_0}'")]
	AipackTomlMissing(String),

	#[display("version field is missing or empty in '{_0}'")]
	VersionMissing(String),

	#[display("namespace field is missing or empty in '{_0}'")]
	NamespaceMissing(String),

	#[display("name field is missing or empty in '{_0}'")]
	NameMissing(String),

	#[display("Fail to install pack: {aipack_ref}\nCause: {cause}")]
	FailToInstall {
		aipack_ref: String,
		cause: String,
	},

	#[display(
		"Cannot install version {new_version} because installed version {installed_version} is newer.\nUse --force to force install."
	)]
	InstallFailInstalledVersionAbove {
		installed_version: String,
		new_version: String,
	},

	#[display("Invalid prerelease format in version {version}. Prereleases must end with .number (e.g., -alpha.1)")]
	InvalidPrereleaseFormat {
		version: String,
	},

	// -- Run
	#[display("Before All Lua block did not return a valid structure.\nCause: {cause}")]
	BeforeAllFailWrongReturn {
		cause: String,
	},
	#[display("Data Lua block did not return a valid structure.\nCause: {cause}")]
	DataFailWrongReturn {
		cause: String,
	},

	// -- Genai
	#[display("Environment API KEY missing: {env_name}")]
	GenAIEnvKeyMissing {
		model_iden: ModelIden,
		env_name: String,
	},
	#[display("Fail to make AI Request.\nCause: {_0}")]
	GenAI(genai::Error),

	// -- Store
	#[from]
	Store(crate::model::Error),

	// -- TokioSync
	TokioTryCurrent(TryCurrentError),

	// -- Externals / custom
	Zip {
		zip_file: String,
		cause: String,
	},
	ZipContent {
		zip_file: String,
		content_path: String,
		cause: String,
	},
	ZipFileNotFound {
		zip_file: String,
		content_path: String,
	},
	ZipFail {
		zip_dir: String,
		cause: String,
	},
	UnzipZipFail {
		zip_file: String,
		cause: String,
	},

	// -- Externals auto froms
	#[from]
	Udiffx(udiffx::Error),
	#[from]
	FlumeRecv(flume::RecvError),
	FlumeSend(String),
	#[from]
	Serde(serde_json::Error),
	#[from]
	#[display("{_0}")]
	Toml(toml::de::Error),
	#[from]
	#[display("{_0}")]
	Yaml(serde_yaml_ng::Error),
	#[from]
	JsonValueExt(value_ext::JsonValueExtError),
	#[from]
	Handlebars(handlebars::RenderError),
	#[from]
	SimpleFs(simple_fs::Error),
	#[from]
	Clap(clap::error::Error),
	#[from]
	Reqwest(reqwest::Error),
	#[from]
	Io(std::io::Error),

	// -- Custom
	#[display("{_0}")]
	#[from]
	Custom(String),

	#[display("Error: {_0}\n\tCause: {_1}")]
	CustomAndCause(String, String),
}

impl From<genai::Error> for Error {
	fn from(genai_error: genai::Error) -> Self {
		match genai_error {
			genai::Error::Resolver {
				model_iden,
				resolver_error,
			} => {
				if let genai::resolver::Error::ApiKeyEnvNotFound { env_name } = resolver_error {
					Error::GenAIEnvKeyMissing { model_iden, env_name }
				} else {
					Error::GenAI(genai::Error::Resolver {
						model_iden,
						resolver_error,
					})
				}
			}
			other => Error::GenAI(other),
		}
	}
}

// region:    --- Custom display

// trait CustomDisplay {
// 	fn c_display(&self) -> String;
// }

// endregion: --- Custom display

// region:    --- Custom

impl Error {
	pub fn custom(val: impl std::fmt::Display) -> Self {
		Self::Custom(val.to_string())
	}

	pub fn custom_and_cause(context: impl Into<String>, cause: impl std::fmt::Display) -> Self {
		Self::CustomAndCause(context.into(), cause.to_string())
	}

	/// Same as custom_and_cause (just a "cute" shorcut)
	pub fn cc(context: impl Into<String>, cause: impl std::fmt::Display) -> Self {
		Self::CustomAndCause(context.into(), cause.to_string())
	}
}

impl From<&str> for Error {
	fn from(val: &str) -> Self {
		Self::Custom(val.to_string())
	}
}

// endregion: --- Custom

// region:    --- Error Boilerplate

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
