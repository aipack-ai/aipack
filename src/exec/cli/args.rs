use crate::exec::ExecActionEvent;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CliArgs {
	/// Subcommands
	#[command(subcommand)]
	pub cmd: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
	/// Initialize the workspace `.aipack/` and the base `~/.aipack-base/` aipack directories
	Init(InitArgs),

	#[command(name = "init-base", about = "Init the ~/.aipack-base only with force update")]
	InitBase,

	#[command(
		about = "Executes the AIPack agent using `aip run demo@craft/code`, or an agent file `aip run path/to/agent.aip`.\n\n\
    Example usage:\n
\
    ```sh\n\
    # Run a direct agent file from the local directory\n\
    aip run some/agent.aip\n\
		\n\
    # Gives two inputs\n\
    aip run some/agent.aip -i \"input 1\" -i \"input 2\"\n\
		\n\
    # Use -f to give file or globs (each matched file will be a input)\n\
    aip run some/agent.aip -f \"src/**/*.js\"\n\
		\n\
    # Run the demo@craft/code AIP agent\n\
    aip run demo@craft/code\n\
    \n\
    # Run the demo@proof main.aip agent and provide a single file as input\n\
    aip run demo@proof -f ./README.md\n\
    \n\
    ```"
	)]
	Run(RunArgs),

	/// Create a new agent from a built-in template
	/// Disabled for now
	//New(NewArgs),

	/// List the available aipacks `aip run list` or `aip run list demo@`
	List(ListArgs),

	/// Pack a directory into a .aipack file
	Pack(PackArgs),

	/// Install an `.aipack` file, archive URL, repository pack, or Git repository pack
	Install(InstallArgs),

	/// Unpack a repo pack into the workspace custom pack area
	Unpack(UnpackArgs),

	/// Check available API keys in the environment
	#[command(name = "check-keys", about = "Check available API keys in the environment")]
	CheckKeys(CheckKeysArgs),

	/// Create a .gitignore file from a template
	#[command(name = "create-gitignore", about = "Create a .gitignore file from a template")]
	CreateGitignore(CreateGitignoreArgs),

	/// Self management commands (e.g., setup, update)
	#[command(name = "self", about = "Manage the aip CLI itself")]
	Xelf(XelfArgs),
}

/// Custom function
impl CliCommand {
	/// Returns true if this CliCommand should be in interative mode.
	///
	/// For now, for all Run, the interactive is on by default, regardless if it watch.
	pub fn is_interactive(&self) -> bool {
		match self {
			CliCommand::Run(run_args) => !run_args.single_shot,
			CliCommand::Init(_) => false,
			CliCommand::InitBase => false,
			//CliCommand::New(_) => true,
			CliCommand::List(_) => false,
			CliCommand::Pack(_) => false,
			CliCommand::Install(_) => false,
			CliCommand::Unpack(_) => false,
			CliCommand::CheckKeys(_) => false,       // Non-interactive
			CliCommand::CreateGitignore(_) => false, // Non-interactive
			CliCommand::Xelf(_) => false,            // Non-interactive
		}
	}

	pub fn is_tui(&self) -> bool {
		match self {
			CliCommand::Run(run_args) => run_args.is_tui(),
			CliCommand::Init(_) => false,
			CliCommand::InitBase => false,
			//CliCommand::New(_) => false,
			CliCommand::List(_) => false,
			CliCommand::Pack(_) => false,
			CliCommand::Install(_) => false,
			CliCommand::Unpack(_) => false,
			CliCommand::CheckKeys(_) => false,       // Non-interactive
			CliCommand::CreateGitignore(_) => false, // Non-interactive
			CliCommand::Xelf(_) => false,            // Non-interactive
		}
	}
}

// region:    --- Sub Command Args

/// Arguments for the `run` subcommand
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct RunArgs {
	#[clap(help = "The name of the agent, which can be:\n\
- A AIP pack reference:\n\
  `aip run demo@proof`\n\
- Or a direct file:\n\
  `aip run path/to/agent.aip`")]
	pub cmd_agent_name: String,

	/// Optional input, allowing multiple input
	/// NOTE: CANNOT be combined with -f/--on-files
	#[arg(short = 'i', long = "input")]
	pub on_inputs: Option<Vec<String>>,

	/// Optional file parameter, allowing multiple files
	/// NOTE: CANNOT be combined with -i/--input
	#[arg(short = 'f', long = "on-files")]
	pub on_files: Option<Vec<String>>,

	/// Optional watch flag
	#[arg(short = 'w', long = "watch")]
	pub watch: bool,

	/// Verbose mode
	#[arg(short = 'v', long = "verbose")]
	pub verbose: bool,

	/// Attempt to open the agent file (for now use VSCode code command)
	#[arg(short = 'o', long = "open")]
	pub open: bool,

	/// Dry mode, takes either 'req' or 'res'
	#[arg(long = "dry", value_parser = ["req", "res"])]
	pub dry_mode: Option<String>,

	/// Single Shot execution (e.g., non-interactive).
	/// (Was the `--ni` or `--non-interactive` in v0.6.x)
	#[arg(short = 's', long = "single-shot", alias = "ni")]
	pub single_shot: bool,

	/// Not used in v0.8.x as tui is now the default
	#[arg(long = "xp-tui")]
	xp_tui: bool,

	/// The Old Terminal
	#[arg(long = "old-term")]
	old_term: bool,
}

impl RunArgs {
	pub fn is_tui(&self) -> bool {
		// self.xp_tui // for 0.7.x
		!self.old_term // for 0.8.x
	}
}
/// Arguments for the `pack` subcommand
#[derive(Parser, Debug)]
pub struct PackArgs {
	/// The directory to pack into a .aipack file
	pub dir_path: String,

	/// Optional destination directory for the .aipack file
	/// If not provided, the .aipack file will be created in the current directory
	#[arg(short = 'o', long = "output")]
	pub output_dir: Option<String>,
}

/// Arguments for the `install` subcommand
#[derive(Parser, Debug)]
pub struct InstallArgs {
	/// Force installation even if the pack is already installed and up to date
	#[arg(long = "force")]
	pub force: bool,

	/// The installation source, such as a local `.aipack` path, a repository
	/// pack reference, an HTTP or HTTPS archive link, or a Git URL.
	/// Git URLs support an explicit `#path/to/pack_dir` selector, for example
	/// `git://host/path/repository.git#path/to/pack_dir` or
	/// `git+ssh://git@github.com/owner/repository.git#path/to/pack_dir`.
	/// Without a selector, `pack.toml` must be at the repository root. With a
	/// selector, the selected directory must contain `pack.toml`.
	/// A path after `.git` without `#` is not inferred as a pack path.
	pub aipack_ref: String,
}

/// Arguments for the `unpack` subcommand
#[derive(Parser, Debug)]
pub struct UnpackArgs {
	/// The pack identity to unpack (e.g., `namespace@pack_name`)
	pub pack_ref: String,

	/// Force unpack even if the destination already exists (trashes existing directory first)
	#[arg(long = "force")]
	pub force: bool,
}

/// Arguments for the `list` subcommand
#[derive(Parser, Debug)]
pub struct ListArgs {
	/// A complete or partial aipack reference
	/// (optional)
	/// e.g., `pro@coder` or `jc@` or `@coder`
	pub pack_ref: Option<String>,

	/// Open the .aipack file, and the target file if exists.
	/// Note: For now assume vscode `code ...` is installed
	#[arg(short = 'o', long = "open")]
	pub open: bool,
}

/// DISABLED FOR NOW
/// Arguments for the `new` subcommand
#[derive(Parser, Debug)]
pub struct NewArgs {
	pub agent_path: Option<String>,

	/// Open the .aipack file, and the target file if exists.
	/// Note: For now assume vscode `code ...` is installed
	#[arg(short = 'o', long = "open")]
	pub open: bool,
}

/// Arguments for the `init` subcommand
#[derive(Parser, Debug)]
pub struct InitArgs {
	/// The optional path of were to init the .aipack (relative to current directory)
	/// If not given, aipack will find the closest .aipack/ or create one at current directory
	pub path: Option<String>,
}

/// Arguments for the `check-keys` subcommand
#[derive(Parser, Debug)]
pub struct CheckKeysArgs {}

/// Arguments for the `create-gitignore` subcommand
#[derive(Parser, Debug)]
pub struct CreateGitignoreArgs {
	/// Force overwrite if .gitignore already exists
	#[arg(long = "force")]
	pub force: bool,
}

/// Arguments for the `self` subcommand
#[derive(Parser, Debug)]
pub struct XelfArgs {
	#[command(subcommand)]
	pub cmd: XelfCommand,
}

/// Subcommands for the `self` command
#[derive(Subcommand, Debug)]
pub enum XelfCommand {
	/// Perform initial setup for the aip CLI environment
	Setup(XelfSetupArgs),
	Update(XelfUpdateArgs),
}

/// Arguments for the `self setup` subcommand
#[derive(Parser, Debug)]
pub struct XelfSetupArgs {}

/// Arguments for the `self Update` subcommand
#[derive(Parser, Debug)]
pub struct XelfUpdateArgs {
	/// The version (needs to be valid, can start with 'v')
	#[arg(short = 'v', long = "version")]
	pub version: Option<String>,
}

// endregion: --- Sub Command Args

// region:    --- From CliCommand to ExecCommand

impl From<CliCommand> for ExecActionEvent {
	fn from(cli_cmd: CliCommand) -> Self {
		match cli_cmd {
			CliCommand::Init(init_args) => ExecActionEvent::CmdInit(init_args),
			CliCommand::InitBase => ExecActionEvent::CmdInitBase,
			CliCommand::Run(run_args) => ExecActionEvent::Run(run_args),
			// CliCommand::New(new_args) => ExecActionEvent::CmdNew(new_args),
			// CliCommand::New(new_args) => ExecCommand::NewCommandAgent(new_args),
			CliCommand::List(list_args) => ExecActionEvent::CmdList(list_args),
			CliCommand::Pack(pack_args) => ExecActionEvent::CmdPack(pack_args),
			CliCommand::Install(install_args) => ExecActionEvent::CmdInstall(install_args),
			CliCommand::Unpack(unpack_args) => ExecActionEvent::CmdUnpack(unpack_args),
			CliCommand::CheckKeys(args) => ExecActionEvent::CmdCheckKeys(args),
			CliCommand::CreateGitignore(args) => ExecActionEvent::CmdCreateGitignore(args),
			CliCommand::Xelf(xelf_args) => {
				// Map Xelf subcommands to specific ExecActionEvent variants
				match xelf_args.cmd {
					XelfCommand::Setup(args) => ExecActionEvent::CmdXelfSetup(args),
					XelfCommand::Update(args) => ExecActionEvent::CmdXelfUpdate(args),
				}
			}
		}
	}
}

// endregion: --- From CliCommand to ExecCommand
