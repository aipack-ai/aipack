use crate::run::pricing::{ModelPricing, ProviderPricing};

pub const GROQ_MODELS: &[ModelPricing] = &[
	ModelPricing {
		name: "openai/gpt-oss-20b",
		input_cached: None,
		input_normal: 0.1,
		output_normal: 0.5,
		output_reasoning: None,
	},
	ModelPricing {
		name: "openai/gpt-oss-120b",
		input_cached: None,
		input_normal: 0.15,
		output_normal: 0.75,
		output_reasoning: None,
	},
	ModelPricing {
		name: "moonshotai/kimi-k2-instruct",
		input_cached: None,
		input_normal: 1.0,
		output_normal: 3.0,
		output_reasoning: None,
	},
	ModelPricing {
		name: "meta-llama/llama-4-scout-17b-16e-instruct",
		input_cached: None,
		input_normal: 0.11,
		output_normal: 0.34,
		output_reasoning: None,
	},
	ModelPricing {
		name: "meta-llama/llama-4-maverick-17b-128e-instruct",
		input_cached: None,
		input_normal: 0.2,
		output_normal: 0.6,
		output_reasoning: None,
	},
	ModelPricing {
		name: "meta-llama/llama-guard-4-12b",
		input_cached: None,
		input_normal: 0.2,
		output_normal: 0.2,
		output_reasoning: None,
	},
	ModelPricing {
		name: "deepseek-r1-distill-llama-70b",
		input_cached: None,
		input_normal: 0.75,
		output_normal: 0.99,
		output_reasoning: None,
	},
	ModelPricing {
		name: "qwen/qwen3-32b",
		input_cached: None,
		input_normal: 0.29,
		output_normal: 0.59,
		output_reasoning: None,
	},
	ModelPricing {
		name: "mistral-saba-24b-32k",
		input_cached: None,
		input_normal: 0.79,
		output_normal: 0.79,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3.3-70b-versatile",
		input_cached: None,
		input_normal: 0.59,
		output_normal: 0.79,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-3.1-8b-instant",
		input_cached: None,
		input_normal: 0.05,
		output_normal: 0.08,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama3-70b-8k",
		input_cached: None,
		input_normal: 0.59,
		output_normal: 0.79,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama3-8b-8k",
		input_cached: None,
		input_normal: 0.05,
		output_normal: 0.08,
		output_reasoning: None,
	},
	ModelPricing {
		name: "gemma2-9b-it",
		input_cached: None,
		input_normal: 0.2,
		output_normal: 0.2,
		output_reasoning: None,
	},
	ModelPricing {
		name: "llama-guard-3-8b-8k",
		input_cached: None,
		input_normal: 0.2,
		output_normal: 0.2,
		output_reasoning: None,
	},
];

pub const GROQ: ProviderPricing = ProviderPricing {
	name: "groq",
	models: GROQ_MODELS,
};
