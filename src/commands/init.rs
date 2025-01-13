use std::{
    fs::{self, File},
    io::Write,
};

use clap::Args;
use goblin::{
    pe::{optional_header::StandardFields32, PE},
    Object,
};

use crate::{
    config::{Config, Section},
    util::{self},
};

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
                                    .map_err(|err| {
                                        format!("failed to get section name ({})", err.to_string())
                                    })?
                                    .to_string(),
                                //addr_file: section.pointer_to_raw_data,
                                //size_file: section.size_of_raw_data,
                                //addr_virtual: section.virtual_address,
                                //size_virtual: section.virtual_size,
                                //flags: section.characteristics,
                                units: None,
                            })
                        })
                        .collect::<Result<Vec<Section>, String>>()?;

                    let config = Config {
                        executable: self.executable.clone(),
                        hash: hash,
                        //base_addr_virtual: pe.header.optional_header.unwrap().windows_fields.image_base,
                        //entry: StandardFields32::from(pe.header.optional_header.unwrap().standard_fields).address_of_entry_point,
                        //subsystem: pe.header.optional_header.unwrap().windows_fields.subsystem,
                        sections: sections,
                    };

                    let toml_string = toml::to_string_pretty(&config).unwrap();

                    let mut cfg_file = File::create("pod.toml").unwrap();
                    cfg_file.write_all(toml_string.as_bytes()).unwrap();

                    Ok(())
                }
                Err(err) => Err(format!("executable parsing failed ({})", err.to_string())),
            },
            Err(err) => Err(format!("failed to open executable ({})", err.to_string())),
        }
    }
}
