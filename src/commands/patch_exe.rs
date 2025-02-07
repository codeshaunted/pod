use std::{fs, option, path::Path, process::Command};

use clap::Args;
use goblin::pe::{
    section_table::{self, SectionTable},
    PE,
};

use crate::util;

use super::CommandExecute;

#[derive(Debug, Args)]
pub struct PatchExeArgs {}

impl CommandExecute for PatchExeArgs {
    fn execute(&self) -> Result<(), String> {
        let config = util::get_config()?;

        let original_file = fs::read(&config.executable)
            .map_err(|err| format!("failed to open original executable ({})", err))?;

        let original_pe = PE::parse(&original_file)
            .map_err(|err| format!("failed to parse original executable ({})", err))?;

        let build_dir = Path::new("build");
        let binding = build_dir.join(
            Path::new(&config.executable)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        let linked_file_path = binding.to_str().unwrap();

        let linked_file = fs::read(linked_file_path)
            .map_err(|err| format!("failed to open linked executable ({})", err))?;

        let mut patched_file = linked_file.clone();

        let linked_pe = PE::parse(&linked_file)
            .map_err(|err| format!("failed to parse linked executable ({})", err))?;

        let sig_ptr_off = 0x3c;
        let sig_ptr = u32::from_le_bytes(
            linked_file[sig_ptr_off..sig_ptr_off + 4]
                .try_into()
                .map_err(|err| {
                    format!(
                        "failed to read the PE signature pointer in linked executable ({})",
                        err
                    )
                })?,
        );

        let mut off = sig_ptr as usize + 0x18 + linked_pe.header.coff_header.size_of_optional_header as usize;

        for _ in 0..linked_pe.sections.len() {
            let i_sec = SectionTable::parse(&linked_file, &mut off, 0).map_err(|err| format!("failed to parse section table in linked executable ({})", err))?;

            if let Some(original_sec) = original_pe.sections.iter().find(|sec| sec.name == i_sec.name) {
                patched_file[off - 32..off - 28].copy_from_slice(&original_sec.virtual_size.to_le_bytes());
                println!("patched virtual size for section {}", i_sec.name().unwrap());
            }
        }

        off = sig_ptr as usize + 0x18 + linked_pe.header.coff_header.size_of_optional_header as usize - 0x80;
        
        if let Some(optional_header) = original_pe.header.optional_header {
            if let Some(export_table) = optional_header.data_directories.get_export_table() {
                patched_file[off..off + 4].copy_from_slice(&export_table.virtual_address.to_le_bytes());
                patched_file[off + 4..off + 8].copy_from_slice(&export_table.size.to_le_bytes());
            }
            off += 8;
            
            if let Some(import_table) = optional_header.data_directories.get_import_table() {
                patched_file[off..off + 4].copy_from_slice(&import_table.virtual_address.to_le_bytes());
                patched_file[off + 4..off + 8].copy_from_slice(&import_table.size.to_le_bytes());
            }
            off += 8;

            if let Some(resource_table) = optional_header.data_directories.get_resource_table() {
                patched_file[off..off + 4].copy_from_slice(&resource_table.virtual_address.to_le_bytes());
                patched_file[off + 4..off + 8].copy_from_slice(&resource_table.size.to_le_bytes());
            }
            off += 8;

            // todo: do the rest of these
        }

        fs::write(linked_file_path, patched_file).map_err(|err| format!("failed to write patched linked executable to disk ({})", err))?;

        println!("successfully wrote patched linked executable to `{}`", linked_file_path);

        Ok(())
    }
}
