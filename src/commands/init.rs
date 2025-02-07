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
                                units: vec![Unit {
                                    kind: "copy".to_string(),
                                    file: None,
                                    addr_virtual: pe.image_base + section.virtual_address as usize,
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
