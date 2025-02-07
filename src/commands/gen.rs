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

            if let Some(cfg_sec) = config.sections.iter().find(|i_sec| i_sec.name == sec_name) {
                let mut unit_i = 0;
                for unit in cfg_sec.units.iter() {
                    match unit.kind.as_str() {
                        "copy" => {
                            let asm_path =
                                build_dir.join(format!("{}_copy_{}.asm", sec_name, unit_i));

                            let asm_command = Command::new(&config.assembler_path)
                                .arg(format!(
                                    "/Fo{}",
                                    build_dir.join(format!("{}_copy_{}.obj", sec_name, unit_i)).display()
                                ))
                                .arg("/c")
                                .arg(&asm_path)
                                .output()
                                .map_err(|err| {
                                    format!("failed to execute copy asm command for section `{}`, unit `{}` ({})", sec_name, unit_i, err)
                                })?;

                            if asm_command.status.success() {
                                println!("assembled copy unit for section `{}`, unit `{}`", sec_name, unit_i);
                            } else {
                                return Err(format!(
                                    "copy assembly of section `{}`, unit `{}` failed ({})",
                                    sec_name,
                                    unit_i,
                                    String::from_utf8_lossy(&asm_command.stdout)
                                ));
                            }
                        },
                        "asm" => {
                            if let Some(asm_path) = &unit.file {
                                let asm_command = Command::new(&config.assembler_path)
                                .arg(format!(
                                    "/Fo{}",
                                    build_dir.join(format!("{}_asm_{}.obj", sec_name, unit_i)).display()
                                ))
                                .arg("/c")
                                .arg(&asm_path)
                                .output()
                                .map_err(|err| {
                                    format!("failed to execute asm command for section `{}`, unit `{}` ({})", sec_name, unit_i, err)
                                })?;

                                if asm_command.status.success() {
                                    println!("assembled asm unit for section `{}`, unit `{}`", sec_name, unit_i);
                                } else {
                                    return Err(format!(
                                        "assembly of section `{}`, unit `{}` failed ({})",
                                        sec_name,
                                        unit_i,
                                        String::from_utf8_lossy(&asm_command.stdout)
                                    ));
                                }
                            } else {
                                return Err(format!("asm unit for section `{}`, unit `{}` is missing file path", sec_name, unit_i))
                            }
                        }
                        _ => {
                            return Err(format!(
                                "section `{}`, unit `{}` has invalid kind `{}`",
                                sec_name, unit_i, unit.kind
                            ))
                        }
                    }

                    unit_i += 1;
                }
            }
        }

        Ok(())
    }
}
