//! Base module for the script engine.
//!
//! NOTE: At this point, Lua is the only planned scripting language for aipack.
//!       It is small, simple, relatively well-known, efficient, and in many ways was made for these kinds of use cases.
//!

// region:    --- Modules

mod aipack_custom;
mod error_lua_support;
mod lua_script;

pub use aipack_custom::*;
pub use lua_script::*;

// endregion: --- Modules
