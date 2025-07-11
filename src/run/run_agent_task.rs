use super::pricing::price_it;
use crate::Result;
use crate::agent::{Agent, AgentOptions, PromptPart, parse_prompt_part_options};
use crate::hub::get_hub;
use crate::run::AiResponse;
use crate::run::literals::Literals;
use crate::run::{DryMode, RunBaseOptions};
use crate::runtime::Runtime;
use crate::script::{AipackCustom, DataResponse, FromValue};
use crate::store::rt_model::{LogKind, RuntimeCtx};
use crate::store::{Id, Stage};
use crate::support::hbs::hbs_render;
use crate::support::text::{self, format_duration, format_usage};
use genai::ModelIden;
use genai::chat::{CacheControl, ChatMessage, ChatRequest, ChatResponse};
use serde_json::Value;
use std::borrow::Cow;
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
pub async fn run_agent_task(
	runtime: &Runtime,
	run_id: Id,
	task_id: Id,
	agent: &Agent,
	before_all_result: Value,
	label: &str,
	input: Value,
	literals: &Literals,
	run_base_options: &RunBaseOptions,
) -> Result<Option<RunAgentInputResponse>> {
	let hub = get_hub();
	let client = runtime.genai_client();

	// -- Build Base Rt Context
	let rt_ctx = RuntimeCtx::from_run_task_ids(runtime, Some(run_id), Some(task_id))?;

	// -- Extract the run model resolved
	// (for comparison later)
	let run_model_resolved = agent.model_resolved();

	// -- Build the scope
	// Note: Probably way to optimize the number of lua engine we create
	//       However, nice to be they are fully scoped.
	let lua_engine = runtime.new_lua_engine_with_ctx(literals, rt_ctx.with_stage(Stage::Data))?;

	let lua_scope = lua_engine.create_table()?;
	lua_scope.set("input", lua_engine.serde_to_lua_value(input.clone())?)?;
	lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all_result.clone())?)?;
	lua_scope.set("options", agent.options_as_ref())?;

	let agent_dir = agent.file_dir()?;
	let agent_dir_str = agent_dir.as_str();

	// -- Execute data
	let DataResponse { input, data, options } = if let Some(data_script) = agent.data_script().as_ref() {
		// -- Rt Step - Data Start
		runtime.step_task_data_start(run_id, task_id).await?;

		// -- Exec
		let lua_value = lua_engine.eval(data_script, Some(lua_scope), Some(&[agent_dir_str]))?;
		let data_res = serde_json::to_value(lua_value)?;

		// -- Rt Step - Start Dt Start
		runtime.step_task_data_end(run_id, task_id).await?;

		// -- Post Process
		// skip input if aipack action is sent
		match AipackCustom::from_value(data_res)? {
			// If it is not a AipackCustom the data is the orginal value
			FromValue::OriginalValue(data) => DataResponse {
				data: Some(data),
				input: Some(input),
				..Default::default()
			},

			// If we have a skip, we can skip
			FromValue::AipackCustom(AipackCustom::Skip { reason }) => {
				let reason_txt = reason.map(|r| format!(" (Reason: {r})")).unwrap_or_default();

				runtime
					.rec_log_data(
						run_id,
						task_id,
						format!("Aipack Skip input at Data stage: {label}{reason_txt}"),
						Some(LogKind::SysInfo),
					)
					.await?;

				return Ok(None);
			}

			// We have a `return aip.flow.data_response(...)``
			FromValue::AipackCustom(AipackCustom::DataResponse(DataResponse {
				input: input_ov,
				data,
				options,
			})) => DataResponse {
				input: input_ov.or(Some(input)),
				data,
				options,
			},

			FromValue::AipackCustom(other) => {
				return Err(format!(
					"Aipack Custom '{other_ref}' is not supported at the Data stage",
					other_ref = other.as_ref()
				)
				.into());
			}
		}
	} else {
		DataResponse {
			input: Some(input),
			data: None,
			options: None,
		}
	};

	// -- Normalize the context
	let input = input.unwrap_or(Value::Null);
	let data = data.unwrap_or(Value::Null);
	// here we use cow, not not clone the agent if no options
	let agent: Cow<Agent> = if let Some(options_to_merge) = options {
		let options_to_merge: AgentOptions = serde_json::from_value(options_to_merge)?;
		let options_ov = agent.options_as_ref().merge_new(options_to_merge)?;
		Cow::Owned(agent.new_merge(options_ov)?)
	} else {
		Cow::Borrowed(agent)
	};

	let data_scope = HashMap::from([
		// The hbs scope data
		// Note: for now, we do not add the before all
		("data".to_string(), data.clone()),
		("input".to_string(), input.clone()),
	]);

	// -- Execute genai if we have an instruction
	let mut chat_messages: Vec<ChatMessage> = Vec::new();
	let data_scope = serde_json::to_value(data_scope)?;
	for prompt_part in agent.prompt_parts() {
		let PromptPart {
			kind,
			content,
			options_str,
		} = prompt_part;

		// Note: If we have an options_str, then add it as the first line
		//       this way it can take advantage of being rendered
		//       and then, we will extract it later
		let (options_line, content) = if let Some(options_str) = options_str {
			(true, Cow::Owned(format!("{options_str}\n{content}")))
		} else {
			(false, Cow::Borrowed(content))
		};

		let rendered_content = hbs_render(content.as_str(), &data_scope)?;

		// If options_line, then we extract it
		let (options_str, rendered_content) = if options_line {
			text::extract_first_line(rendered_content)
		} else {
			(String::new(), rendered_content)
		};
		let options_str = options_str.trim();

		// For now, only add if not empty
		if !rendered_content.trim().is_empty() {
			let options = if !options_str.is_empty() {
				parse_prompt_part_options(options_str)?
			} else {
				None
			};
			let options = if options.as_ref().map(|v| v.cache).unwrap_or(false) {
				Some(CacheControl::Ephemeral.into())
			} else {
				None
			};
			chat_messages.push(ChatMessage {
				role: kind.into(),
				content: rendered_content.into(),
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
				"-- {role}:\n{content}",
				role = msg.role,
				content = msg.content.text().unwrap_or_default()
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
	if run_model_resolved != model_resolved {
		// -- Rt Update Task - Model
		runtime.update_task_model_ov(run_id, task_id, model_resolved).await?;
	}

	let ai_response: Option<AiResponse> = if !is_inst_empty {
		let chat_req = ChatRequest::from_messages(chat_messages);

		hub.publish(format!("-> Sending rendered instruction to {model_resolved} ..."))
			.await;

		// -- Rt Step - start AI
		runtime.step_task_ai_start(run_id, task_id).await?;

		let start = Instant::now();
		let chat_res = client
			.exec_chat(model_resolved, chat_req, Some(agent.genai_chat_options()))
			.await?;
		let duration = start.elapsed();

		// -- Rt Step - end AI
		runtime.step_task_ai_end(run_id, task_id).await?;

		// region:    --- First Info Part

		let duration_msg = format!("Duration: {duration_str}", duration_str = format_duration(duration));
		// this is for the duration in second with 3 digit for milli (for the AI Response)
		let duration_sec = duration.as_secs_f64(); // Convert to f64
		let duration_sec = (duration_sec * 1000.0).round() / 1000.0; // Round to 3 decimal places

		let mut info = duration_msg;

		// Compute the price
		let price_usd = get_price(&chat_res);

		// -- Rt Rec - Update Cost
		if let Some(price_usd) = price_usd {
			let _ = runtime.update_task_cost(run_id, task_id, price_usd).await;
		}

		// add to info
		if let Some(price_usd) = price_usd {
			info = format!("{info} | ~${price_usd}")
		}

		let usage_msg = format_usage(&chat_res.usage);
		info = format!("{info} | {usage_msg}");

		// endregion: --- First Info Part

		hub.publish(format!(
			"<- ai_response content received - {model_name} | {info}",
			model_name = chat_res.provider_model_iden.model_name
		))
		.await;

		let ChatResponse {
			content,
			reasoning_content,
			usage,
			model_iden: res_model_iden,
			provider_model_iden: res_provider_model_iden,
			..
		} = chat_res;

		// -- Rt Rec - Update Task Usage
		runtime.update_task_usage(run_id, task_id, &usage).await?;

		let content = content
			.into_iter()
			.filter_map(|c| c.into_text())
			.collect::<Vec<_>>()
			.join("\n\n");

		let ai_response_content = if content.is_empty() { None } else { Some(content) };
		let ai_response_reasoning_content = reasoning_content;

		let model_info = format_model(&agent, &res_model_iden, &res_provider_model_iden, &agent.options());
		if run_base_options.verbose() {
			hub.publish(format!(
				"\n-- AI Output ({model_info})\n\n{content}\n",
				content = ai_response_content.as_deref().unwrap_or_default()
			))
			.await;
		}

		let info = format!("{info} | {model_info}",);

		Some(AiResponse {
			content: ai_response_content,
			reasoning_content: ai_response_reasoning_content,
			model_name: res_model_iden.model_name,
			adapter_kind: res_model_iden.adapter_kind,
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
		// -- Rt Step - start output
		runtime.step_task_output_start(run_id, task_id).await?;

		// -- Create the Output Lua Engine
		let lua_engine = runtime.new_lua_engine_with_ctx(literals, rt_ctx.with_stage(Stage::Output))?;

		// -- Create the scope
		let lua_scope = lua_engine.create_table()?;
		lua_scope.set("input", lua_engine.serde_to_lua_value(input)?)?;
		lua_scope.set("data", lua_engine.serde_to_lua_value(data)?)?;
		lua_scope.set("before_all", lua_engine.serde_to_lua_value(before_all_result)?)?;
		lua_scope.set("ai_response", ai_response)?;
		lua_scope.set("options", agent.options_as_ref())?;

		let lua_value = lua_engine.eval(output_script, Some(lua_scope), Some(&[agent_dir_str]))?;
		let output_response = serde_json::to_value(lua_value)?;

		// -- Rt Step - end output
		runtime.step_task_output_end(run_id, task_id).await?;

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

/// Model: gemini-2.0-flash | Adapter: Gemini
/// TODO: Might want to use the agent model somehow
fn format_model(
	_agent: &Agent,
	res_model_iden: &ModelIden,
	res_provider_model_iden: &ModelIden,
	agent_options: &AgentOptions,
) -> String {
	// let model_iden = agent.model_resolved();
	let model_section = if *res_model_iden.model_name != *res_provider_model_iden.model_name {
		format!(
			"Model: {model_name} ({provider_model_name}) ",
			model_name = res_model_iden.model_name,
			provider_model_name = res_provider_model_iden.model_name
		)
	} else {
		format!("Model: {model_name} ", model_name = res_model_iden.model_name)
	};

	let temp_section = if let Some(temp) = agent_options.temperature() {
		Cow::Owned(format!(" | Temperature: {temp}"))
	} else {
		Cow::Borrowed("")
	};

	let top_p_section = if let Some(top_p) = agent_options.top_p() {
		Cow::Owned(format!(" | top_p: {top_p}"))
	} else {
		Cow::Borrowed("")
	};

	format!(
		"{model_section}| Adapter: {adapter_kind}{temp_section}{top_p_section}",
		adapter_kind = res_model_iden.adapter_kind,
	)
}

// endregion: --- Support
