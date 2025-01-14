use std::{fs, path::Path, process::Command};

use clap::Args;
use goblin::pe::PE;

use crate::util;

use super::CommandExecute;

#[derive(Debug, Args)]
pub struct GenArgs {}

impl CommandExecute for GenArgs {
    fn execute(&self) -> Result<(), String> {
        let config = util::get_config()?;

        let file = fs::read(&config.executable)
            .map_err(|err| format!("failed to open executable ({})", err))?;

        let pe = PE::parse(&file).map_err(|err| format!("failed to parse executable ({})", err))?;

        let build_dir = Path::new("build");

        for sec in pe.sections.iter() {
            let sec_name = sec
                .name()
                .map_err(|err| format!("failed to get section name ({})", err))?;
            let asm_path = build_dir.join(sec_name.to_owned() + ".asm");
            let asm_path_str = asm_path.to_str().unwrap();

            let asm_command = Command::new(&config.assembler_path)
                .arg(format!("/Fo{}", build_dir.join(sec_name.to_owned() + ".obj").to_str().unwrap()))
                .arg("/c")
                .arg(&asm_path)
                .output()
                .map_err(|err| format!("failed to execute assemble command ({})", err))?;

            if asm_command.status.success() {
                println!("assembled `{}`", asm_path_str);
            } else {
                return Err(format!("assembly of `{}` failed ({})", asm_path_str, String::from_utf8_lossy(&asm_command.stdout)));
            }
        }

        Ok(())
    }
}
