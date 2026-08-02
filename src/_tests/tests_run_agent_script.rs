type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>; // For tests.

use super::*;
use crate::_test_support::{assert_contains, load_inline_agent, load_test_agent, run_test_agent_with_input};
use crate::types::FileInfo;
use simple_fs::SPath;

#[tokio::test]
async fn test_run_agent_script_hello_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let agent = load_test_agent("./agent-script/agent-hello.aip", &runtime)?;

	// -- Execute
	let res = run_test_agent_with_input(&runtime, &agent, "input-01").await?;

	// -- Check
	// Note here '' because input is null
	assert_contains(
		res.as_str().ok_or("Should have output result")?,
		"Hello 'input-01' from agent-hello.aip",
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_four_backticks_ok() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let agent = load_test_agent("./agent-script/agent-four-backticks.aip", &runtime)?;

	// -- Execute
	let res = run_test_agent_with_input(&runtime, &agent, "input-four-backticks").await?;

	// -- Check
	assert_eq!(
		res.as_str().ok_or("Should have output result")?,
		"Hello 'input-four-backticks' from agent-four-backticks.aip"
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_require_lua() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let agent = load_test_agent("./other/demo", &runtime)?;

	// -- Exec
	let res = run_test_agent_with_input(&runtime, &agent, Value::Null).await?;

	// -- Check
	let res = res.as_str().ok_or("Should be string")?;
	assert_eq!(res, "demo.name_one is 'Demo One'");

	Ok(())
}

/// NOTE: For now disable the HubCapture test see below
///
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_agent_script_data_response_full_ov() -> Result<()> {
	// -- Setup & Fixtures
	let content = r#"
# Data
```lua
return aip.flow.data_response({
	input   = "new input",
  data    = "hello",
	options = { model = "another-model"}
})
```
# Output
```lua
return {
  input   = input,
	data    = data,
	options = options,
}
```
	"#;
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let dir_context = runtime.dir_context();
	let agent = Agent::mock_from_content(content)?;

	// -- Execute
	let on_path = SPath::new("./some-random/file.txt");
	let path_ref = FileInfo::new(dir_context, on_path, false);
	let inputs = vec![serde_json::to_value(path_ref)?];

	let res = run_agent(&runtime, None, agent, Some(inputs), &RunBaseOptions::default(), true).await?;

	// -- Check
	let outputs = res.outputs.ok_or("Should have output values")?;
	let output = outputs.first().expect("should have one output");
	assert_eq!(output.x_get_str("data")?, "hello");
	assert_eq!(output.x_get_str("input")?, "new input");
	assert_eq!(output.x_get_str("/options/model")?, "another-model");

	Ok(())
}

/// NOTE: For now disable the HubCapture test see below
///
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_run_agent_script_before_all_response_simple() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let dir_context = runtime.dir_context();
	let agent = load_test_agent("./agent-script/agent-before-all.aip", &runtime)?;

	// -- Execute
	let on_path = SPath::new("./some-random/file.txt");
	let path_ref = FileInfo::new(dir_context, on_path, false);
	let inputs = vec![serde_json::to_value(path_ref)?];

	let res = run_agent(&runtime, None, agent, Some(inputs), &RunBaseOptions::default(), true).await?;

	// -- Check
	let outputs = res.outputs.ok_or("Should have output values")?;
	assert_eq!(outputs.len(), 1, "should have one and only one output");
	let output = outputs.into_iter().next().ok_or("Should have one output")?;
	assert_eq!(
		output.as_str().ok_or("Output should be string")?,
		"Some Before All - Some Data - ./some-random/file.txt"
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_with_options_read() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let agent = load_test_agent("./agent-script/agent-options.aip", &runtime)?;

	// -- Exec
	let inputs = vec!["one".into(), "two".into()];
	let res = run_agent(&runtime, None, agent, Some(inputs), &RunBaseOptions::default(), true).await?;

	let outputs = res.outputs.ok_or("Should have output values")?;
	let first_output = outputs
		.first()
		.ok_or("Should have at least one output")?
		.as_str()
		.ok_or("after_all should be str")?;

	let after_all = res.after_all.ok_or("should have after_all")?;
	let after_all = after_all.as_str().ok_or("after_all should be str")?;

	// -- Check First Output
	assert_contains(first_output, "b_r_model: deepseek-chat");
	assert_contains(first_output, "i_model: cost-saver");
	assert_contains(first_output, "i_r_model: deepseek-chat");
	assert_contains(first_output, "o_model: cost-saver");
	assert_contains(first_output, "o_r_model: deepseek-chat");

	// -- Check After All
	assert_contains(after_all, "a_r_model: deepseek-chat");
	assert_contains(after_all, "a_b_r_model: deepseek-chat");

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_before_all_inputs_reshape() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let agent = load_test_agent("./agent-script/agent-before-all-inputs-reshape.aip", &runtime)?;

	// -- Exec
	let inputs = vec!["one".into(), "two".into()];
	let res = run_agent(&runtime, None, agent, Some(inputs), &RunBaseOptions::default(), true)
		.await?
		.outputs
		.ok_or("Should have output values")?;

	// -- Check
	let res = res.iter().map(|v| v.as_str().unwrap_or_default()).collect::<Vec<_>>();
	assert_eq!(res[0], "Data with input: 'one-0'");
	assert_eq!(res[1], "Data with input: 'two-1'");
	assert_eq!(res[2], "Data with input: 'C'");

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_before_all_inputs_gen() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let agent = load_test_agent("./agent-script/agent-before-all-inputs-gen.aip", &runtime)?;

	// -- Exec
	let res = run_agent(&runtime, None, agent, None, &RunBaseOptions::default(), true).await?;

	// -- Check
	let res_value = serde_json::to_value(res)?;

	// check the null values (because of skip or return)
	assert!(
		matches!(res_value.x_get::<Value>("/outputs/1")?, Value::Null),
		"the 2nd input should be null per agent md"
	);
	assert!(
		matches!(res_value.x_get::<Value>("/outputs/3")?, Value::Null),
		"the 4th input should be null per agent md"
	);
	assert!(
		matches!(res_value.x_get::<Value>("/outputs/4")?, Value::Null),
		"the 5th input should be null per agent md"
	);

	// lazy checks with the json string
	let res_pretty = res_value.x_pretty()?.to_string();
	assert_contains(&res_pretty, r#""data": "Data with input: 'one'""#);
	assert_contains(&res_pretty, r#""rexported_inputs": ["#);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_skip_simple() -> Result<()> {
	common_test_run_agent_script_skip(None).await
}

#[tokio::test]
async fn test_run_agent_script_skip_reason() -> Result<()> {
	common_test_run_agent_script_skip(Some("Some reason")).await
}

async fn common_test_run_agent_script_skip(reason: Option<&str>) -> Result<()> {
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;

	let reason_str = reason.map(|v| format!("\"{v}\"")).unwrap_or_default();
	// -- Setup & Fixtures
	let fx_inputs = &["one", "two", "three"];
	let fx_agent = format!(
		r#"
# Data
```lua
if input == "one" then
  return aipack.skip({reason_str})
end
```

# Output

```lua
return "output for: " .. input
```
	"#
	);

	let agent = load_inline_agent("./dummy/path.aip", fx_agent)?;

	// -- Execute
	let inputs = fx_inputs.iter().map(|v| Value::String(v.to_string())).collect();
	let res = run_agent(&runtime, None, agent, Some(inputs), &RunBaseOptions::default(), true)
		.await?
		.outputs
		.ok_or("Should have output result")?;

	// -- Check
	// check the result
	assert_eq!(res.first().ok_or("Should have input 0")?, &Value::Null);
	assert_eq!(
		res.get(1)
			.ok_or("Should have input 1")?
			.as_str()
			.ok_or("input 1 should be string")?,
		"output for: two"
	);
	assert_eq!(
		res.get(2)
			.ok_or("Should have input 2")?
			.as_str()
			.ok_or("input 2 should be string")?,
		"output for: three"
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_data_redo_run_fails() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let fx_agent = r#"
# Data
```lua
return aip.flow.redo_run()
```
	"#;
	let agent = load_inline_agent("./dummy/path.aip", fx_agent)?;

	// -- Execute
	let err = run_agent(
		&runtime,
		None,
		agent,
		Some(vec![Value::String("one".to_string())]),
		&RunBaseOptions::default(),
		true,
	)
	.await
	.expect_err("should fail when redo_run is returned from # Data");

	// -- Check
	assert_contains(
		&err.to_string(),
		"aip.flow.redo_run() can be returned only from # Before All or # After All stages. Returned from # Data stage.",
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_output_redo_run_fails() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let fx_agent = r#"
# Data
```lua
return { value = input }
```

# Output
```lua
return aip.flow.redo_run()
```
	"#;
	let agent = load_inline_agent("./dummy/path.aip", fx_agent)?;

	// -- Execute
	let err = run_agent(
		&runtime,
		None,
		agent,
		Some(vec![Value::String("one".to_string())]),
		&RunBaseOptions::default(),
		true,
	)
	.await
	.expect_err("should fail when redo_run is returned from # Output");

	// -- Check
	assert_contains(
		&err.to_string(),
		"aip.flow.redo_run() can be returned only from # Before All or # After All stages. Returned from # Output stage.",
	);

	Ok(())
}

#[tokio::test]
async fn test_run_agent_script_redo_chain_keeps_one_loop() -> Result<()> {
	// -- Setup & Fixtures
	let runtime = Runtime::new_test_runtime_sandbox_01().await?;
	let redo_agent = Agent::mock_from_content(
		r#"
# Before All
```lua
return aip.flow.redo_run()
```

# Data
```lua
return "redo"
```

# Output
```lua
return data
```
	"#,
	)?;
	let final_agent = Agent::mock_from_content(
		r#"
# Data
```lua
return "done"
```

# Output
```lua
return data
```
	"#,
	)?;
	let run_options = RunBaseOptions::default();

	// -- Exec
	let first = crate::run::run_agent_with_identity(
		&runtime,
		None,
		redo_agent.clone(),
		None,
		&run_options,
		true,
		None,
		false,
	)
	.await?;
	let loop_id = first.loop_id.ok_or("The first redo should create a loop")?;

	let standalone = crate::run::run_agent_with_identity(
		&runtime,
		None,
		final_agent.clone(),
		None,
		&run_options,
		true,
		None,
		false,
	)
	.await?;

	let second = crate::run::run_agent_with_identity(
		&runtime,
		None,
		redo_agent,
		None,
		&run_options,
		true,
		Some(loop_id),
		false,
	)
	.await?;

	let third = crate::run::run_agent_with_identity(
		&runtime,
		None,
		final_agent,
		None,
		&run_options,
		true,
		Some(loop_id),
		false,
	)
	.await?;

	let loop_info = crate::model::LoopBmc::get(runtime.mm(), loop_id)?;
	let members = crate::model::LoopBmc::list_members(runtime.mm(), loop_id)?;
	let standalone_run = crate::model::RunBmc::get(runtime.mm(), standalone.run_id)?;

	// -- Check
	assert!(first.response.redo_requested);
	assert!(second.response.redo_requested);
	assert!(!third.response.redo_requested);
	assert_eq!(first.loop_id, Some(loop_id));
	assert_eq!(second.loop_id, Some(loop_id));
	assert_eq!(third.loop_id, Some(loop_id));
	assert_eq!(loop_info.first_run_id, first.run_id);
	assert_eq!(loop_info.last_run_id, third.run_id);
	assert!(!loop_info.pending);
	assert!(standalone_run.loop_id.is_none());
	assert_eq!(members.len(), 3);
	assert_eq!(
		members.iter().map(|run| run.id).collect::<Vec<_>>(),
		vec![first.run_id, second.run_id, third.run_id]
	);

	Ok(())
}
