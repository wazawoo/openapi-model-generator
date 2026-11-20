use clap::Parser;
use indexmap::IndexMap;
use openapi_model_generator::{cli::Args, generator, parser, Error, Result};
use openapiv3::OpenAPI;
use std::fs;
use std::io;
use std::path::PathBuf;

fn ensure_file_exists(path: &PathBuf, arg_name: &str) -> Result<()> {
    if !path.exists() {
        return Err(Error::from(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{arg_name} path {path:?} does not exist"),
        )));
    }

    if !path.is_file() {
        return Err(Error::from(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{arg_name} path {path:?} is not a file"),
        )));
    }

    Ok(())
}

pub fn validate_input_file(path: &PathBuf) -> Result<()> {
    println!("Checking input file: {path:?}");
    ensure_file_exists(path, "Input")?;

    fs::File::open(path).map(|_| {
        println!("Input file is valid and readable.");
    })?;

    Ok(())
}

pub fn create_output_dir(path: &PathBuf) -> Result<()> {
    println!("Checking output directory: {path:?}");

    if path.exists() {
        if path.is_dir() {
            println!("Output directory already exists.");
            Ok(())
        } else {
            Err(Error::from(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("Path {path:?} exists but is not a directory"),
            )))
        }
    } else {
        println!("Creating directory: {path:?}");
        fs::create_dir_all(path)?;
        println!("Directory created.");
        Ok(())
    }
}

pub fn get_models_to_skip(csv_path: &PathBuf) -> Result<Vec<String>> {
    ensure_file_exists(csv_path, "Skip models csv")?;

    let mut models_to_skip: Vec<String> = vec![];
    let expected_column_count = 1;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(false)
        .from_path(csv_path)?;
    for (i, result) in rdr.records().enumerate() {
        match result {
            Ok(record) => {
                println!("{:?}", record);
                let row_size = record.len();
                if row_size != expected_column_count {
                    return Err(Error::CsvOther(format!("Row {i} had {row_size} columns (expected {expected_column_count})")));
                }

                if let Some(model) = record.get(0) {
                    models_to_skip.push(model.into());
                }
            },
            Err(e) => {
                return Err(Error::Csv(e))
            },
        }
    }
    Ok(models_to_skip)
}

pub fn get_model_name_replacements(csv_path: &PathBuf) -> Result<IndexMap<String, String>> {
    ensure_file_exists(csv_path, "Model name replacements csv")?;

    let mut model_name_replacements: IndexMap<String, String> = IndexMap::new();
    let expected_column_count = 2;

    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(false)
        .from_path(csv_path)?;
    for (i, result) in rdr.records().enumerate() {
        match result {
            Ok(record) => {
                println!("{:?}", record);
                let row_size = record.len();
                if row_size != expected_column_count {
                    return Err(Error::CsvOther(format!("Row {i} had {row_size} columns (expected {expected_column_count})")));
                }

                if let Some(model) = record.get(0) && let Some(replacement) = record.get(1) {
                    model_name_replacements.insert(model.into(), replacement.into());
                }
            },
            Err(e) => {
                return Err(Error::Csv(e))
            },
        }
    }
    Ok(model_name_replacements)
}

fn main() -> Result<()> {
    let args = Args::parse();

    if let Err(e) = validate_input_file(&args.input) {
        eprintln!("Failed to validate input file: {e}");
        std::process::exit(1);
    }

    if let Err(e) = create_output_dir(&args.output) {
        eprintln!("Failed to create output directory: {e}");
        std::process::exit(1);
    }

    let mut models_to_skip: Vec<String> = vec![];
    if let Some(skip_path) = args.skip_models_csv && let Ok(to_skip) = get_models_to_skip(&skip_path) {
        models_to_skip = to_skip;
    }

    let mut type_name_replacements: IndexMap<String, String> = IndexMap::new();
    if let Some(replace_path) = args.replace_models_csv && let Ok(replacements) = get_model_name_replacements(&replace_path) {
        type_name_replacements = replacements;
    }

    let content = fs::read_to_string(&args.input)?;

    let openapi: OpenAPI = if args.input.extension().is_some_and(|ext| ext == "yaml") {
        serde_yaml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };

    let (models, requests, responses, routes) = parser::parse_openapi(&openapi)?;

    let rust_code = generator::generate_models(&models, &requests, &responses, &models_to_skip, &type_name_replacements)?;
    let output_models_path = args.output.join("models.rs");
    fs::write(&output_models_path, rust_code.trim())?;

    let rust_routes = generator::generate_routes(&routes, &type_name_replacements)?;
    let output_routes_path = args.output.join("routes.rs");
    fs::write(&output_routes_path, rust_routes.trim())?;

    let rust_lib = generator::generate_lib()?;
    let output_lib_path = args.output.join("mod.rs");
    fs::write(&output_lib_path, rust_lib.trim())?;

    // let rust_lib = generator::g()?;
    let tests = generator::generate_tests(&models, &requests, &responses, &routes, &models_to_skip, &type_name_replacements)?;
    let output_lib_path = args.output.join("tests.rs");
    fs::write(&output_lib_path, tests.trim())?;

    let readme = generator::generate_readme(&models, &requests, &responses, &routes, &models_to_skip, &type_name_replacements)?;
    let output_lib_path = args.output.join("README.md");
    fs::write(&output_lib_path, readme.trim())?;

    println!("Models generated successfully to {output_models_path:?}");

    Ok(())
}
