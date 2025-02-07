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
        let donor_file_path = format!("{}.donor", binding.to_str().unwrap());

        let link_command = Command::new(config.linker_path)
            .arg("-mi386pe")
            .arg(format!("-o{}", donor_file_path))
            .arg("-n")
            .arg("-Tbuild/link.ld")
            .arg("--subsystem=windows")
            .arg("--strip-debug")
            .arg("--disable-dynamicbase")
            .arg("--disable-nxcompat")
            .arg("--strip-all")
            .arg("--major-image-version=0")
            .arg("--file-alignment=1")
            .arg(format!("--image-base=0x{:X}", pe.image_base))
            .output()
            .map_err(|err| format!("failed to execute link command ({})", err))?;

        if link_command.status.success() {
            println!("linked object files into `{}`", donor_file_path);
        } else {
            return Err(format!(
                "linkage failed ({})",
                String::from_utf8_lossy(&link_command.stderr)
            ));
        }

        let donor_file = fs::read(&donor_file_path)
            .map_err(|err| format!("failed to open donor executable ({})", err))?;
        let donor_pe: PE<'_> = PE::parse(&donor_file)
            .map_err(|err| format!("failed to parse executable ({})", err))?;

        let donee_file_path = format!("{}.donee", binding.to_str().unwrap());
        let mut donee_file = fs::read(&donee_file_path)
            .map_err(|err| format!("failed to open donee executable ({})", err))?;
        for sec in pe.sections.iter() {
            let sec_name = sec
                .name()
                .map_err(|err| format!("failed to get donee section name ({})", err))?;
            if let Some(donor_sec) = donor_pe
                .sections
                .iter()
                .find(|i_sec| i_sec.name == sec.name)
            {
                if sec.size_of_raw_data != donor_sec.size_of_raw_data {
                    return Err(format!(
                        "donor section and donee `{}` section data sizes do not match: {} vs {}",
                        sec_name, donor_sec.size_of_raw_data, sec.size_of_raw_data
                    ));
                }
                let donee_data_start = sec.pointer_to_raw_data as usize;
                let donee_data_end = donee_data_start + sec.size_of_raw_data as usize;
                let donor_data_start = donor_sec.pointer_to_raw_data as usize;
                let donor_data_end = donor_data_start + donor_sec.size_of_raw_data as usize;

                let original_slice = &file[donee_data_start..donee_data_end];
                let donor_slice = &donor_file[donor_data_start..donor_data_end];

                for (i, (x, y)) in original_slice.iter().zip(donor_slice.iter()).enumerate() {
                    if x != y {
                        return Err(format!(
                            "donor and original `{}` section mismatch at index `{}`: {:02x} vs {:02x}",
                            sec_name, i, donor_slice[i], original_slice[i]
                        ));
                    }
                }
                /*if original_slice != donor_slice {
                    return Err(format!(
                        "donor and original `{}` section data does not match",
                        sec_name
                    ));
                }*/

                donee_file[donee_data_start..donee_data_end]
                    .copy_from_slice(&donor_file[donor_data_start..donor_data_end]);

                println!("donated `{}` section to donee executable", sec_name);
            } else {
                return Err(format!(
                    "donor executable is missing section `{}`",
                    &sec_name
                ));
            }
        }

        let final_file_path = binding.to_str().unwrap();
        fs::write(final_file_path, donee_file).map_err(|err| format!("failed to write final executable to disk ({})", err))?;
        
        println!("output final executable at `{}`", final_file_path);

        Ok(())
    }
}
