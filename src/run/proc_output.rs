use crate::Result;
use crate::agent::Agent;
use crate::model::{RuntimeCtx, Stage};
use crate::run::run_agent_task::RunAgentInputResponse;
use crate::run::{AiResponse, Literals};
use crate::runtime::Runtime;
use crate::script::{AipackCustom, FromValue};
use serde_json::Value;

#[allow(clippy::too_many_arguments)]
pub async fn process_output(
	runtime: &Runtime,
	base_rt_ctx: &RuntimeCtx,
	agent: Agent,
	literals: &Literals,
	data: Value,
	before_all: Value,
	input: Value,
	ai_response: Option<AiResponse>,
) -> Result<Option<RunAgentInputResponse>> {
	if let Some(output_script) = agent.output_script() {
		// -- Create the Output Lua Engine
		let lua_engine = runtime.new_lua_engine_with_ctx(literals, base_rt_ctx.with_stage(Stage::Output))?;

		// -- Create the scope
		let lua_scope = lua_engine.create_table()?;
		lua_scope.set("input", lua_engine.serde_to_lua_value(input)?)?;
		lua_scope.set("data", lua_engine.serde_to_lua_value(data)?)?;
		lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all)?)?;
		lua_scope.set("ai_response", ai_response)?;
		lua_scope.set("options", agent.options_as_ref())?;

		let lua_value = lua_engine.eval(output_script, Some(lua_scope), Some(&[agent.file_dir()?.as_str()]))?;
		let output_response = serde_json::to_value(lua_value)?;

		if let Ok(FromValue::AipackCustom(AipackCustom::Redo)) = AipackCustom::from_value(output_response.clone()) {
			return Ok(Some(RunAgentInputResponse::Redo));
		}

		Ok(Some(RunAgentInputResponse::OutputResponse(output_response)))
	} else {
		Ok(ai_response.map(RunAgentInputResponse::AiReponse))
	}
}
