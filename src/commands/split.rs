use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use clap::Args;
use goblin::pe::PE;

use crate::util;

use super::CommandExecute;

#[derive(Debug, Args)]
pub struct SplitArgs {}

impl CommandExecute for SplitArgs {
    fn execute(&self) -> Result<(), String> {
        let config = util::get_config()?;

        let file = fs::read(&config.executable)
            .map_err(|err| format!("failed to open executable ({})", err))?;

        let pe = PE::parse(&file).map_err(|err| format!("failed to parse executable ({})", err))?;

        let build_dir = Path::new("build");
        if !build_dir.exists() {
            fs::create_dir_all(build_dir)
                .map_err(|err| format!("failed to create build directory ({})", err))?;
        }

        // generate donee exe
        let mut donee_file_data = file.clone();
        for sec in pe.sections.iter() {
            let data_start = sec.pointer_to_raw_data as usize;
            let data_end = data_start + sec.size_of_raw_data as usize;

            donee_file_data[data_start..data_end].fill(0);
        }

        let exe_name = match Path::new(&config.executable).file_name() {
            Some(name_os_str) => match name_os_str.to_str() {
                Some(name_str) => name_str,
                None => return Err("failed to parse executable path's executable name".to_string()),
            },
            None => return Err("executable path is missing final executable name".to_string()),
        };

        let donee_file_path = build_dir.join(format!("{}.donee", exe_name));
        let mut donee_file = File::create(&donee_file_path).map_err(|err| {
            format!(
                "failed to create donee executable file at `{}`, ({})",
                donee_file_path.display(),
                err
            )
        })?;
        donee_file.write_all(&donee_file_data).map_err(|err| {
            format!(
                "failed to write to donee executable file at `{}` ({})",
                donee_file_path.display(),
                err
            )
        })?;

        println!("generated donee executable at `{}`", donee_file_path.display());

        let mut link_script = String::new();
        link_script += "ENTRY(_start)\n\nSECTIONS {\n";

        link_script += &format!("\t_start = 0x{:X};\n\n", pe.image_base + pe.entry);

        for sec in pe.sections.iter() {
            let sec_name = sec
                .name()
                .map_err(|err| format!("failed to get section name ({})", err))?;
            if let Some(cfg_sec) = config.sections.iter().find(|i_sec| i_sec.name == sec_name) {
                link_script += &format!(
                    "\t{} 0x{:X} : {{\n",
                    sec_name,
                    pe.image_base + sec.virtual_address as usize
                );

                let mut last_unit_end: usize = pe.image_base + sec.virtual_address as usize;
                let mut unit_i = 0;
                for unit in cfg_sec.units.iter() {
                    if unit.addr_virtual != last_unit_end {
                        return Err(format!("in section `{}`, unit `{}` does not begin at the end of the last unit (or start of section)", sec_name, unit_i));
                    }

                    match unit.kind.as_str() {
                        "copy" => {
                            let mut asm = String::new();
                            asm += ".386\n.MODEL flat\nPOD SEGMENT BYTE\n";

                            let data_start = sec.pointer_to_raw_data as usize + unit.addr_virtual
                                - sec.virtual_address as usize
                                - pe.image_base;
                            let data_end = data_start + unit.raw_size;
                            let data = &file[data_start..data_end]; // some issue here TODO FIX AVERY

                            // 49 is the max bytes MASM supports in one DB call for some reason
                            for chunk in data.chunks(49) {
                                asm += "DB ";

                                for byte in chunk.iter() {
                                    asm += &byte.to_string();
                                    asm += ", "
                                }

                                asm.pop();
                                asm.pop();
                                asm += "\n"
                            }

                            asm += "POD ENDS\nEND\n";

                            let asm_path =
                                build_dir.join(format!("{}_copy_{}.asm", sec_name, unit_i));
                            let mut asm_file = File::create(&asm_path).map_err(|err| {
                                format!(
                                    "failed to create section `{}`, unit `{}` copy asm file ({})",
                                    sec_name,
                                    unit_i,
                                    err.to_string()
                                )
                            })?;

                            asm_file.write_all(asm.as_bytes()).map_err(|err| {
                                format!(
                                    "failed to write section `{}`, unit `{}`, copy asm file ({})",
                                    sec_name, unit_i, err
                                )
                            })?;

                            println!(
                                "wrote section `{}`, unit `{}` copy asm data to `{}`",
                                sec_name,
                                unit_i,
                                asm_path.display()
                            );

                            link_script +=
                                &format!("\t\tbuild/{}_copy_{}.obj(POD)\n", sec_name, unit_i,);
                        }
                        "asm" => {
                            if let Some(asm_path) = &unit.file {
                                println!(
                                    "added `{}`, unit `{}` asm file `{}` data to linker script",
                                    sec_name, unit_i, asm_path
                                );

                                link_script +=
                                    &format!("\t\tbuild/{}_asm_{}.obj(POD)\n", sec_name, unit_i);
                            } else {
                                return Err(format!(
                                    "asm unit for section `{}`, unit `{}` is missing file path",
                                    sec_name, unit_i
                                ));
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
                    last_unit_end += unit.raw_size;
                }

                if last_unit_end
                    != pe.image_base + (sec.virtual_address + sec.size_of_raw_data) as usize
                {
                    return Err(format!(
                        "sizes of units for section `{}` is not the same as the section size",
                        sec_name
                    ));
                }

                link_script += "\t}\n\n";
            } else {
                return Err(format!(
                    "section `{}` is missing unit configuration",
                    sec_name
                ));
            }
        }

        link_script.pop();
        link_script += "}\n";

        let link_path = build_dir.join("link.ld");
        let mut link_file = File::create(&link_path)
            .map_err(|err| format!("failed to create link script file ({})", err))?;

        link_file
            .write_all(link_script.as_bytes())
            .map_err(|err| format!("failed to write link script file ({})", err))?;

        println!("wrote link.ld file to `{}`", link_path.to_str().unwrap());

        Ok(())
    }
}
