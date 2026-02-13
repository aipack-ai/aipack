mod data_anthropic;
mod data_deepseek;
mod data_fireworks;
mod data_gemini;
mod data_groq;
mod data_openai;
mod data_together;
mod data_xai;
mod data_zai;

use crate::run::pricing::ProviderPricing;

pub const PROVIDERS: &[ProviderPricing] = &[
	data_openai::OPENAI,
	data_groq::GROQ,
	data_gemini::GEMINI,
	data_deepseek::DEEPSEEK,
	data_anthropic::ANTHROPIC,
	data_xai::XAI,
	data_fireworks::FIREWORKS,
	data_together::TOGETHER,
	data_zai::ZAI,
];
