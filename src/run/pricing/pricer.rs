use super::ModelPricing;
use crate::model::AiPrice;
use genai::ModelIden;
use genai::chat::Usage;

/// Calculates the price for a given provider type, model name, and usage.
///
/// # Arguments
/// * `provider_type` - The type of provider (e.g., "openai", "groq", "gemini", "deepseek", "anthropic")
/// * `model_name` - The name of the model to price
/// * `usage` - The token usage information
///
/// # Returns
/// * `Option<PriceResult>` - The calculated price information, or None if the provider or model was not found
pub fn price_it(provider_type: &str, model_name: &str, usage: &Usage) -> Option<AiPrice> {
	let ai_cost = aicost::compute(provider_type, model_name, usage).ok()?;
	Some(AiPrice {
		cost: ai_cost.total,
		cost_cache_write: (ai_cost.input_cache_write > 0.0).then_some(ai_cost.input_cache_write),
		cost_cache_saving: (ai_cost.input_cache_saving != 0.0).then_some(ai_cost.input_cache_saving),
	})
}

pub fn model_pricing(model_iden: &ModelIden) -> Option<ModelPricing> {
	let pricing = aicost::model_pricing(model_iden)?;
	let prices = pricing.prices;

	Some(ModelPricing {
		name: pricing.name,
		input_cached: prices.input_cached,
		input_normal: prices.input_normal,
		output_normal: prices.output_normal,
		output_reasoning: prices.output_reasoning,
	})
}
