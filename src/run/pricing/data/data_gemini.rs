use crate::run::pricing::{ModelPricing, ProviderPricing};

pub const GEMINI_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "gemini-3-pro",
		input_cached: Some(0.2),
		input_normal: 2.0,
		output_normal: 12.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-3-flash",
		input_cached: Some(0.05),
		input_normal: 0.5,
		output_normal: 3.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.5-pro",
		input_cached: Some(0.125),
		input_normal: 1.25,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.5-flash",
		input_cached: Some(0.03),
		input_normal: 0.3,
		output_normal: 2.5,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.5-flash-lite",
		input_cached: Some(0.01),
		input_normal: 0.1,
		output_normal: 0.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.0-flash",
		input_cached: Some(0.025),
		input_normal: 0.1,
		output_normal: 0.4,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.0-flash-lite",
		input_cached: None,
		input_normal: 0.075,
		output_normal: 0.3,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-flash-latest",
		input_cached: Some(0.03),
		input_normal: 0.3,
		output_normal: 2.5,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-pro-latest",
		input_cached: Some(0.125),
		input_normal: 1.25,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-embedding-001",
		input_cached: None,
		input_normal: 0.15,
		output_normal: 0.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-robotics-er-1.5",
		input_cached: None,
		input_normal: 0.3,
		output_normal: 2.5,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemini-2.5-computer-use",
		input_cached: None,
		input_normal: 1.25,
		output_normal: 10.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemma-3",
		input_cached: Some(0.0),
		input_normal: 0.0,
		output_normal: 0.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemma-3n",
		input_cached: Some(0.0),
		input_normal: 0.0,
		output_normal: 0.0,
		output_reasoning: None,
	},
];

pub const GEMINI: ProviderPricing = ProviderPricing {
	name: "gemini",
	models: GEMINI_MODELS,
};
