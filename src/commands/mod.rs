use clap::{Parser, Subcommand};

pub mod gen;
pub mod init;
pub mod link;
pub mod patch_exe;
pub mod split;
pub trait CommandExecute {
    fn execute(&self) -> Result<(), String>;
}

#[derive(Debug, Parser)]
#[command(name = "pod")]
#[command(about = "a PE binary splitting and re-linking utility", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    Init(init::InitArgs),
    Split(split::SplitArgs),
    Gen(gen::GenArgs),
    Link(link::LinkArgs),
    PatchExe(patch_exe::PatchExeArgs),
}
