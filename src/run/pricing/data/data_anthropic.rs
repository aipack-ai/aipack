use crate::run::pricing::{ModelPricing, ProviderPricing};

pub const ANTHROPIC: ProviderPricing = ProviderPricing {
	name: "anthropic",
	models: ANTHROPIC_MODELS,
};

pub const ANTHROPIC_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "claude-opus-4-6",
		input_cached: Some(0.5),
		input_normal: 5.0,
		output_normal: 25.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-opus-4-5",
		input_cached: Some(0.5),
		input_normal: 5.0,
		output_normal: 25.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-opus-4-1",
		input_cached: Some(1.5),
		input_normal: 15.0,
		output_normal: 75.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-opus-4",
		input_cached: Some(1.5),
		input_normal: 15.0,
		output_normal: 75.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-sonnet-4-5",
		input_cached: Some(0.3),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-sonnet-4",
		input_cached: Some(0.3),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-haiku-4-5",
		input_cached: Some(0.1),
		input_normal: 1.0,
		output_normal: 5.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-3-5-haiku",
		input_cached: Some(0.08),
		input_normal: 0.8,
		output_normal: 4.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "claude-haiku-3",
		input_cached: Some(0.03),
		input_normal: 0.25,
		output_normal: 1.25,
		output_reasoning: None,
	},
];
