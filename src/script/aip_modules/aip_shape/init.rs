use crate::Result;
use crate::runtime::Runtime;
use mlua::{Lua, Table};

use super::shape_records::{columnar_to_records, record_to_values, records_to_columnar, to_record, to_records};
use crate::script::aip_modules::aip_shape::shape_keys::{extract_keys, omit_keys, remove_keys, select_keys};

pub fn init_module(lua: &Lua, _runtime: &Runtime) -> Result<Table> {
	let table = lua.create_table()?;

	let to_record_fn =
		lua.create_function(move |lua, (names, values): (Table, Table)| to_record(lua, names, values))?;
	let to_records_fn = lua.create_function(move |lua, (names, rows): (Table, Table)| to_records(lua, names, rows))?;

	let record_to_values_fn =
		lua.create_function(move |lua, (rec, names): (Table, Option<Table>)| record_to_values(lua, rec, names))?;

	let columnar_to_records_fn = lua.create_function(move |lua, cols: Table| columnar_to_records(lua, cols))?;
	let records_to_columnar_fn = lua.create_function(move |lua, recs: Table| records_to_columnar(lua, recs))?;

	let select_keys_fn = lua.create_function(move |lua, (rec, keys): (Table, Table)| select_keys(lua, rec, keys))?;
	let omit_keys_fn = lua.create_function(move |lua, (rec, keys): (Table, Table)| omit_keys(lua, rec, keys))?;
	let extract_keys_fn = lua.create_function(move |lua, (rec, keys): (Table, Table)| extract_keys(lua, rec, keys))?;
	let remove_keys_fn = lua.create_function(move |lua, (rec, keys): (Table, Table)| remove_keys(lua, rec, keys))?;

	// -- Records
	table.set("to_record", to_record_fn)?;
	table.set("to_records", to_records_fn)?;

	// -- Values
	table.set("record_to_values", record_to_values_fn)?;

	// -- Columnar
	table.set("columnar_to_records", columnar_to_records_fn)?;
	table.set("records_to_columnar", records_to_columnar_fn)?;

	// -- Keys
	table.set("select_keys", select_keys_fn)?;
	table.set("omit_keys", omit_keys_fn)?;
	table.set("extract_keys", extract_keys_fn)?;
	table.set("remove_keys", remove_keys_fn)?;

	Ok(table)
}
