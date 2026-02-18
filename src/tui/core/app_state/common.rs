use crate::model::Id;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigTab {
	ApiKeys,
	ModelAliases,
	Help,
}

impl ConfigTab {
	pub fn next(self) -> Self {
		match self {
			ConfigTab::ApiKeys => ConfigTab::ModelAliases,
			ConfigTab::ModelAliases => ConfigTab::Help,
			ConfigTab::Help => ConfigTab::ApiKeys,
		}
	}

	#[allow(unused)]
	pub fn prev(self) -> Self {
		match self {
			ConfigTab::ApiKeys => ConfigTab::Help,
			ConfigTab::ModelAliases => ConfigTab::ApiKeys,
			ConfigTab::Help => ConfigTab::ModelAliases,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStage {
	Normal,
	Installing,
	Installed,
	PromptInstall(Id),
	Config(ConfigTab),
}
