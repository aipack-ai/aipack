// region:    --- Modules

mod as_strs_ext;
mod cow_lines;
mod str_ext;
mod vec_ext;

pub use as_strs_ext::*;
pub use cow_lines::*;
#[allow(unused)]
pub use str_ext::*;
pub use vec_ext::*;

pub mod code;
pub mod consts;
pub mod csvs;
pub mod docx;
pub mod editor;
pub mod files;
pub mod hbs;
pub mod html;
pub mod jsons;
pub mod md;
pub mod os;
pub mod paths;
pub mod pdf;
pub mod proc;
pub mod text;
pub mod time;
pub mod tomls;
pub mod webc;
pub mod yamls;
pub mod zip;

// endregion: --- Modules

/// Generic wrapper for a NewType Pattern
pub struct W<T>(pub T);
