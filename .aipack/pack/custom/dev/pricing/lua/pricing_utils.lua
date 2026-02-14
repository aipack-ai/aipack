local base_dir    = "data/models-pricing"
local config_path = "config.jsonc"

local function load_config()
  local file = aip.file.load(config_path, { base_dir = CTX.AGENT_FILE_DIR })
  return aip.json.parse(file.content)
end

local function load_inputs()
  local config = load_config()
  local providers = config.providers or {}

  for _, provider in ipairs(providers) do
    provider._display = provider.name
    provider.md_path = base_dir .. "/.data/pricing-" .. provider.name .. ".md"
    provider.html_path = base_dir .. "/.data/pricing-" .. provider.name .. ".html"
    provider.json_path = base_dir .. "/.data/pricing-" .. provider.name .. ".json"
  end

  return providers
end

return {
  base_dir = base_dir,
  json_all_path = base_dir .. "/pricing-all.json",
  load_config = load_config,
  load_inputs = load_inputs
}
