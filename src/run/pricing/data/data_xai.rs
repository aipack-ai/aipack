use crate::run::pricing::{ModelPricing, ProviderPricing};

pub const XAI_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "grok-4",
		input_cached: Some(0.75),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-3",
		input_cached: Some(0.75),
		input_normal: 3.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-3-mini",
		input_cached: Some(0.075),
		input_normal: 0.3,
		output_normal: 0.5,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-3-fast",
		input_cached: Some(1.25),
		input_normal: 5.0,
		output_normal: 25.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-3-mini-fast",
		input_cached: Some(0.15),
		input_normal: 0.6,
		output_normal: 4.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-beta",
		input_cached: None,
		input_normal: 5.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-2-image-gen",
		input_cached: None,
		input_normal: 0.0,
		output_normal: 0.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-2-vision-1212",
		input_cached: None,
		input_normal: 2.0,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-2-1212",
		input_cached: None,
		input_normal: 2.0,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-vision-beta",
		input_cached: None,
		input_normal: 5.0,
		output_normal: 15.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "grok-2-image-1212",
		input_cached: None,
		input_normal: 0.0,
		output_normal: 0.07,
		output_reasoning: None,
	},
];

pub const XAI: ProviderPricing = ProviderPricing {
	name: "xai",
	models: XAI_MODELS,
};
