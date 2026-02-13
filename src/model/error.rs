use derive_more::{Display, From};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Display, From)]
#[display("{self:?}")]
pub enum Error {
	#[from(String, &String, &str)]
	Custom(String),

	MutexPoison,

	// -- Externals
	#[from]
	Rusqlite(rusqlite::Error),
}

// region:    --- Froms

impl<T> From<std::sync::PoisonError<T>> for Error {
	fn from(_val: std::sync::PoisonError<T>) -> Self {
		Self::MutexPoison
	}
}

// endregion: --- Froms

// region:    --- Custom

impl Error {
	pub fn custom_from_err(err: impl std::error::Error) -> Self {
		Self::Custom(err.to_string())
	}

	pub fn custom(val: impl Into<String>) -> Self {
		Self::Custom(val.into())
	}

	/// Same as custom_and_cause (just a "cute" shorcut)
	pub fn cc(context: impl Into<String>, cause: impl std::fmt::Display) -> Self {
		Self::Custom(format!("{}.\nCause: {}", context.into(), cause))
	}
}

// endregion: --- Custom

// region:    --- Error Boilerplate

impl std::error::Error for Error {}

// endregion: --- Error Boilerplate
