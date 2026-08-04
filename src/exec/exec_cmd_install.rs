use crate::Result;
use crate::dir_context::DirContext;
use crate::exec::cli::InstallArgs;
use crate::exec::packer::{InstallResponse, InstalledPack, install_pack};
use crate::hub::get_hub;
use size::Size;

/// Executes the install command which installs an aipack file
pub async fn exec_install(dir_context: DirContext, install_args: InstallArgs) -> Result<InstalledPack> {
	let hub = get_hub();
	let install_res = install_pack(&dir_context, &install_args.aipack_ref, install_args.force).await?;

	let (installed_pack, skipped) = match install_res {
		InstallResponse::Installed(pack) => {
			hub.publish(format!(
				"\n==== Installing aipack:\n\n{:>15} {}",
				"From:", install_args.aipack_ref
			))
			.await;
			(pack, false)
		}
		InstallResponse::UpToDate(pack) => {
			hub.publish(format!(
				"\n==== Pack Already Installed\n\n{:>15} {}\n{:>15} {}",
				"Local:", pack.pack_toml.version, "Source:", pack.pack_toml.version
			))
			.await;
			(pack, true)
		}
	};

	let is_git_source = install_args.aipack_ref.starts_with("git://")
		|| install_args.aipack_ref.starts_with("git+ssh://");

	// Format the zip size using the size crate
	let formatted_zip_size = Size::from_bytes(installed_pack.zip_size as u64).to_string();

	// Format the size using the size crate
	// let formatted_size = Size::from_bytes(installed_pack.size as u64).to_string();

	if skipped {
		hub.publish(format!(
			"{:>15} {}@{}\n{:>15} {}\n{:>15} {}",
			"Pack:",
			installed_pack.pack_toml.namespace,
			installed_pack.pack_toml.name,
			"Version:",
			installed_pack.pack_toml.version,
			"Installed At:",
			installed_pack.path
		))
		.await;
		hub.publish("\n==== DONE (Skipped)".to_string()).await;
	} else {
		if is_git_source {
			hub.publish(format!(
				"{:>15} {}@{}\n{:>15} {}\n{:>15} {}",
				"Pack:",
				installed_pack.pack_toml.namespace,
				installed_pack.pack_toml.name,
				"Version:",
				installed_pack.pack_toml.version,
				"Installed At:",
				installed_pack.path
			))
			.await;
		} else {
			hub.publish(format!(
				"{:>15} {formatted_zip_size}\n{:>15} {}@{}\n{:>15} {}\n{:>15} {}",
				".aipack Size:",
				"Pack:",
				installed_pack.pack_toml.namespace,
				installed_pack.pack_toml.name,
				"Version:",
				installed_pack.pack_toml.version,
				"Installed At:",
				installed_pack.path
			))
			.await;
		}
		hub.publish("\n==== DONE".to_string()).await;
	}

	Ok(installed_pack)
}
