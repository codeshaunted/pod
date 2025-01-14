use std::{
    fs::{self, File},
    io::Write,
};

use clap::Args;
use goblin::pe::PE;

use crate::config::{Config, Section, Unit};

use super::CommandExecute;

#[derive(Debug, Args)]
pub struct InitArgs {
    pub executable: String,
}

impl CommandExecute for InitArgs {
    fn execute(&self) -> Result<(), String> {
        match fs::read(&self.executable) {
            Ok(file) => match PE::parse(&file) {
                Ok(pe) => {
                    let hash = blake3::hash(&file).to_string();

                    // "use rust", they said
                    // error handling is easy, they said
                    let sections: Vec<Section> = pe
                        .sections
                        .iter()
                        .map(|section| {
                            Ok(Section {
                                name: section
                                    .name()
                                    .map_err(|err| format!("failed to get section name ({})", err))?
                                    .to_string(),
                                //addr_file: section.pointer_to_raw_data,
                                //size_file: section.size_of_raw_data,
                                //addr_virtual: section.virtual_address,
                                //size_virtual: section.virtual_size,
                                //flags: section.characteristics,
                                units: vec![Unit {
                                    kind: "copy".to_string(),
                                    addr_virtual: section.virtual_address as usize,
                                    raw_size: section.size_of_raw_data as usize,
                                }],
                            })
                        })
                        .collect::<Result<Vec<Section>, String>>()?;

                    let config = Config {
                        executable: self.executable.clone(),
                        hash: hash,
                        assembler_path: "ml".to_string(),
                        compiler_path: "cl".to_string(),
                        linker_path: "ld".to_string(),
                        //base_addr_virtual: pe.header.optional_header.unwrap().windows_fields.image_base,
                        //entry: StandardFields32::from(pe.header.optional_header.unwrap().standard_fields).address_of_entry_point,
                        //subsystem: pe.header.optional_header.unwrap().windows_fields.subsystem,
                        sections: sections,
                    };

                    let toml_string = toml::to_string_pretty(&config).unwrap();

                    let mut cfg_file = File::create("pod.toml").unwrap();
                    cfg_file.write_all(toml_string.as_bytes()).unwrap();

                    println!(
                        "initialized pod.toml for executable at `{}`",
                        self.executable
                    );
                    Ok(())
                }
                Err(err) => Err(format!("executable parsing failed ({})", err)),
            },
            Err(err) => Err(format!("failed to open executable ({})", err)),
        }
    }
}
