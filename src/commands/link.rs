use std::{fs, path::Path, process::Command};

use clap::Args;
use goblin::pe::PE;

use crate::util;

use super::CommandExecute;

#[derive(Debug, Args)]
pub struct LinkArgs {}

impl CommandExecute for LinkArgs {
    fn execute(&self) -> Result<(), String> {
        let config = util::get_config()?;

        let file = fs::read(&config.executable)
            .map_err(|err| format!("failed to open executable ({})", err))?;

        let pe = PE::parse(&file).map_err(|err| format!("failed to parse executable ({})", err))?;

        let build_dir = Path::new("build");
        let binding = build_dir.join(
            Path::new(&config.executable)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        let output_file = binding.to_str().unwrap();

        let link_command = Command::new(config.linker_path)
            .arg("-mi386pe")
            .arg(format!("-o{}", output_file))
            .arg("-n")
            .arg("-Tbuild/link.ld")
            .arg("--subsystem=windows")
            .arg("--strip-debug")
            .arg("--disable-dynamicbase")
            .arg("--disable-nxcompat")
            .arg("--strip-all")
            .arg("--major-image-version=0")
            .arg(format!("--image-base=0x{:X}", pe.image_base))
            .output()
            .map_err(|err| format!("failed to execute link command ({})", err))?;

        if link_command.status.success() {
            println!("linked object files into `{}`", output_file);
        } else {
            return Err(format!(
                "linkage of failed ({})",
                String::from_utf8_lossy(&link_command.stderr)
            ));
        }

        Ok(())
    }
}
