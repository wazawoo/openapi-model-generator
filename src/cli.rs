use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// path to openapi spec (json)
    #[arg(short, long)]
    pub input: PathBuf,

    /// output folder path for generated files
    #[arg(short, long, default_value = "./generated")]
    pub output: PathBuf,

    /// path to single-column csv file containing COL1: model names to skip during generation, one per row.
    #[arg(short, long)]
    pub skip_models_csv: Option<PathBuf>,
    /// path to two-column csv file containing COL1: model names to replace, and COL2: their replacements, one per row.
    #[arg(short, long)]
    pub replace_models_csv: Option<PathBuf>,
}