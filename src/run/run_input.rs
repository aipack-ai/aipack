use crate::Result;
use crate::agent::{Agent, PromptPart};
use crate::hub::get_hub;
use crate::pricing::price_it;
use crate::run::AiResponse;
use crate::run::literals::Literals;
use crate::run::{DryMode, RunBaseOptions, Runtime};
use crate::script::{AipackCustom, FromValue};
use crate::support::hbs::hbs_render;
use crate::support::text::{format_duration, format_num};
use genai::chat::{CacheControl, ChatMessage, ChatRequest, ChatResponse, Usage};
use serde_json::Value;
use std::collections::HashMap;
use tokio::time::Instant;

// region:    --- RunAgentInputResponse

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum RunAgentInputResponse {
	AiReponse(AiResponse),
	OutputResponse(Value),
}

impl RunAgentInputResponse {
	pub fn as_str(&self) -> Option<&str> {
		match self {
			RunAgentInputResponse::AiReponse(ai_response) => ai_response.content.as_deref(),
			RunAgentInputResponse::OutputResponse(value) => value.as_str(),
		}
	}

	/// Note: for now, we do like this. Might want to change that.
	/// Note: There is something to do about AI being able to structured output and manage it her
	/// - If AiResposne take the String as value or Null
	/// - If OutputResponse, then, the value is result
	pub fn into_value(self) -> Value {
		match self {
			RunAgentInputResponse::AiReponse(ai_response) => ai_response.content.into(),
			RunAgentInputResponse::OutputResponse(value) => value,
		}
	}
}

// endregion: --- RunAgentInputResponse

/// Run the agent for one input
/// - Build the scope
/// - Execute Data
/// - Render the prompt sections
/// - Send the AI
/// - Execute Output
///
/// Note 1: For now, this will create a new Lua engine.
///         This is likely to stay as it creates a strong segregation between input execution
#[allow(clippy::too_many_arguments)]
pub async fn run_agent_input(
	runtime: &Runtime,
	agent: &Agent,
	before_all_result: Value,
	label: &str,
	input: Value,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Option<RunAgentInputResponse>> {
	let hub = get_hub();
	let client = runtime.genai_client();

	// -- Build the scope
	// Fix me: Probably need to get the engine from the arg
	let lua_engine = runtime.new_lua_engine()?;
	let lua_scope = lua_engine.create_table()?;
	lua_scope.set("input", lua_engine.serde_to_lua_value(input.clone())?)?;
	lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all_result.clone())?)?;
	lua_scope.set("CTX", literals.to_lua(&lua_engine)?)?;
	lua_scope.set("options", agent.options_as_ref())?;

	let agent_dir = agent.file_dir()?;
	let agent_dir_str = agent_dir.as_str();

	// -- Execute data
	let data = if let Some(data_script) = agent.data_script().as_ref() {
		let lua_value = lua_engine.eval(data_script, Some(lua_scope), Some(&[agent_dir_str]))?;
		serde_json::to_value(lua_value)?
	} else {
		Value::Null
	};

	// skip input if aipack action is sent
	let data = match AipackCustom::from_value(data)? {
		// If it is not a AipackCustom the data is the orginal value
		FromValue::OriginalValue(data) => data,

		// If we have a skip, we can skip
		FromValue::AipackCustom(AipackCustom::Skip { reason }) => {
			let reason_txt = reason.map(|r| format!(" (Reason: {r})")).unwrap_or_default();

			hub.publish(format!("-! Aipack Skip input at Data stage: {label}{reason_txt}"))
				.await;
			return Ok(None);
		}

		FromValue::AipackCustom(other) => {
			return Err(format!(
				"-! Aipack Custom '{}' is not supported at the Data stage",
				other.as_ref()
			)
			.into());
		}
	};

	let data_scope = HashMap::from([("data".to_string(), data.clone())]);

	// -- Execute genai if we have an instruction
	let mut chat_messages: Vec<ChatMessage> = Vec::new();
	let data_scope = serde_json::to_value(data_scope)?;
	for prompt_part in agent.prompt_parts() {
		let PromptPart { kind, content, options } = prompt_part;

		let options = if options.as_ref().map(|v| v.cache).unwrap_or(false) {
			Some(CacheControl::Ephemeral.into())
		} else {
			None
		};

		let content = hbs_render(content, &data_scope)?;
		// For now, only add if not empty
		if !content.trim().is_empty() {
			chat_messages.push(ChatMessage {
				role: kind.into(),
				content: content.into(),
				options,
			})
		}
	}
	// let inst = hbs_render(agent.inst(), &data_scope)?;

	let is_inst_empty = chat_messages.is_empty();

	// TODO: Might want to handle if no instruction.
	if run_base_options.verbose() {
		hub.publish("\n").await;
		for msg in chat_messages.iter() {
			hub.publish(format!(
				"-- {}:\n{}",
				msg.role,
				msg.content.text_as_str().unwrap_or_default()
			))
			.await;
		}
	}

	// if dry_mode req, we stop
	if matches!(run_base_options.dry_mode(), DryMode::Req) {
		return Ok(None);
	}

	// -- Now execute the instruction
	let model_resolved = agent.model_resolved();

	let ai_response: Option<AiResponse> = if !is_inst_empty {
		let chat_req = ChatRequest::from_messages(chat_messages);

		hub.publish(format!("-> Sending rendered instruction to {model_resolved} ..."))
			.await;

		let start = Instant::now();
		let chat_res = client
			.exec_chat(model_resolved, chat_req, Some(agent.genai_chat_options()))
			.await?;
		let duration = start.elapsed();
		let duration_msg = format!("Duration: {}", format_duration(duration));
		// this is for the duration in second with 3 digit for milli (for the AI Response)
		let duration_sec = duration.as_secs_f64(); // Convert to f64
		let duration_sec = (duration_sec * 1000.0).round() / 1000.0; // Round to 3 decimal places

		let mut info = duration_msg;

		let price_usd = get_price(&chat_res);
		if let Some(price_usd) = price_usd {
			info = format!("{info} | ~${price_usd}")
		}

		let usage_msg = format_usage(&chat_res.usage);
		info = format!("{info} | {usage_msg}");

		hub.publish(format!("<- ai_response content received - {info}")).await;

		let chat_res_mode_iden = chat_res.model_iden.clone();
		let ChatResponse {
			content,
			reasoning_content,
			usage,
			..
		} = chat_res;

		let ai_response_content = content.and_then(|c| c.text_into_string());
		let ai_response_reasoning_content = reasoning_content;

		if run_base_options.verbose() {
			hub.publish(format!(
				"\n-- AI Output (model: {} | adapter: {})\n\n{}\n",
				chat_res_mode_iden.model_name,
				chat_res_mode_iden.adapter_kind,
				ai_response_content.as_deref().unwrap_or_default()
			))
			.await;
		}

		let info = format!(
			"{info} | Model: {} | Adapter: {}",
			chat_res_mode_iden.model_name, chat_res_mode_iden.adapter_kind,
		);

		Some(AiResponse {
			content: ai_response_content,
			reasoning_content: ai_response_reasoning_content,
			model_name: chat_res_mode_iden.model_name,
			adapter_kind: chat_res_mode_iden.adapter_kind,
			duration_sec,
			price_usd,
			usage,
			info,
		})
	}
	// if we do not have an instruction, just return null
	else {
		hub.publish("-! No instruction, skipping genai.").await;
		None
	};

	// -- if dry_mode res, we stop
	if matches!(run_base_options.dry_mode(), DryMode::Res) {
		return Ok(None);
	}

	// -- Exec output
	let res = if let Some(output_script) = agent.output_script() {
		let lua_engine = runtime.new_lua_engine()?;
		let lua_scope = lua_engine.create_table()?;
		lua_scope.set("input", lua_engine.serde_to_lua_value(input)?)?;
		lua_scope.set("data", lua_engine.serde_to_lua_value(data)?)?;
		lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all_result)?)?;
		lua_scope.set("ai_response", ai_response)?;
		lua_scope.set("CTX", literals.to_lua(&lua_engine)?)?;
		lua_scope.set("options", agent.options_as_ref())?;

		let lua_value = lua_engine.eval(output_script, Some(lua_scope), Some(&[agent_dir_str]))?;
		let output_response = serde_json::to_value(lua_value)?;

		Some(RunAgentInputResponse::OutputResponse(output_response))
	} else {
		ai_response.map(RunAgentInputResponse::AiReponse)
	};

	Ok(res)
}

// region:    --- Support

fn get_price(chat_res: &ChatResponse) -> Option<f64> {
	let provider = chat_res.model_iden.adapter_kind.as_lower_str();
	let model_name = &*chat_res.model_iden.model_name;
	price_it(provider, model_name, &chat_res.usage)
}

fn format_usage(usage: &Usage) -> String {
	let mut buff = String::new();

	buff.push_str("Prompt Tokens: ");
	buff.push_str(&format_num(usage.prompt_tokens.unwrap_or_default() as i64));
	if let Some(prompt_tokens_details) = usage.prompt_tokens_details.as_ref() {
		buff.push_str(" (cached: ");
		let cached = prompt_tokens_details.cached_tokens.unwrap_or(0);
		buff.push_str(&format_num(cached as i64));
		if let Some(cache_creation_tokens) = prompt_tokens_details.cache_creation_tokens {
			buff.push_str(", cache_creation: ");
			buff.push_str(&format_num(cache_creation_tokens as i64));
		}
		buff.push(')');
	}

	buff.push_str(" | Completion Tokens: ");
	buff.push_str(&format_num(usage.completion_tokens.unwrap_or_default() as i64));
	if let Some(reasoning) = usage.completion_tokens_details.as_ref().and_then(|v| v.reasoning_tokens) {
		buff.push_str(" (reasoning: ");
		buff.push_str(&format_num(reasoning as i64));
		buff.push(')');
	}

	buff
}

// endregion: --- Support
