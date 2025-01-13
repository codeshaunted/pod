use std::{
    fs::{self, File},
    io::{Split, Write},
    path::Path,
};

use clap::Args;
use goblin::pe::PE;

use crate::config::Config;

use super::CommandExecute;

#[derive(Debug, Args)]
pub struct SplitArgs {}

impl CommandExecute for SplitArgs {
    fn execute(&self) -> Result<(), String> {
        match fs::read_to_string("pod.toml") {
            Ok(toml_string) => {
                let config: Config = match toml::from_str(&toml_string) {
                    Ok(config) => config,
                    Err(err) => return Err(format!("invalid pod.toml file ({})", err.to_string())),
                };

                let file = match fs::read(config.executable) {
                    Ok(file) => file,
                    Err(err) => return Err(format!("failed to open executable ({})", err)),
                };

                let pe = match PE::parse(&file) {
                    Ok(pe) => pe,
                    Err(err) => return Err(format!("failed to parse executable ({})", err)),
                };

                let build_dir = Path::new("build");
                if !build_dir.exists() {
                    fs::create_dir_all(build_dir).map_err(|err| {
                        format!("failed to create build directory ({})", err.to_string())
                    })?;
                }

                let mut link_script = String::new();
                link_script += "ENTRY(_start)\n\nSECTIONS {\n";

                link_script += &format!("\t_start = 0x{:X};\n\n", pe.image_base + pe.entry);

                for sec in pe.sections.iter() {
                    let mut asm = String::new();
                    asm += ".MODEL flat\n.DATA\n";

                    let data_start = sec.pointer_to_raw_data as usize;
                    let data_end = data_start + sec.size_of_raw_data as usize;
                    let data = &file[data_start..data_end];

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

                    asm += "END\n";

                    let sec_name = sec.name().map_err(|err| {
                        format!("failed to get section name ({})", err.to_string())
                    })?;
                    let asm_path = build_dir.join(sec_name.to_owned() + ".asm");
                    let mut asm_file = File::create(&asm_path).map_err(|err| {
                        format!("failed to create section asm file ({})", err.to_string())
                    })?;

                    asm_file.write_all(asm.as_bytes()).map_err(|err| {
                        format!("failed to write section asm file ({})", err.to_string())
                    })?;

                    println!(
                        "wrote `{}` section data to `{}`",
                        sec_name,
                        asm_path.to_str().unwrap()
                    );

                    link_script += &format!(
                        "\t{} 0x{:X} : {{\n\t\t{}.obj(.data)\n\t}}\n\n",
                        sec_name,
                        pe.image_base + sec.virtual_address as usize,
                        sec_name
                    );
                }

                link_script.pop();
                link_script += "}\n";

                let link_path = build_dir.join("link.ld");
                let mut link_file = File::create(&link_path).map_err(|err| {
                    format!("failed to create link script file ({})", err.to_string())
                })?;

                link_file.write_all(link_script.as_bytes()).map_err(|err| {
                    format!("failed to write link script file ({})", err.to_string())
                })?;

                println!("wrote link.ld file to `{}`", link_path.to_str().unwrap());

                Ok(())
            }
            Err(err) => Err(format!("failed to open pod.toml ({})", err.to_string())),
        }
    }
}
