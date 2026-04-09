use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub input: PathBuf,

    #[arg(short, long, default_value = "./generated")]
    pub output: PathBuf,

    /// Get the path params from the path itself "blah/blah/{path_param1}/" rather than from the path parameters in the spec
    #[arg(long, default_value_t = false)]
    pub get_path_params_from_path: bool,
}
