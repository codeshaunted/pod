use clap::Args;

use super::CommandExecute;

#[derive(Debug, Args)]
pub struct GenArgs {}

impl CommandExecute for GenArgs {
    fn execute(&self) -> Result<(), String> {
        Err("gen".to_string())
    }
}
