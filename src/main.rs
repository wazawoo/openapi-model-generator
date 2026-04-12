use clap::Parser;
use indexmap::IndexMap;
use openapi_model_generator::{cli::Args, generator, parser, Error, Result};
use openapiv3::OpenAPI;
use std::fs;
use std::io;
use std::path::PathBuf;

pub fn validate_input_file(path: &PathBuf) -> Result<()> {
    println!("Checking input file: {path:?}");

    if !path.exists() {
        return Err(Error::from(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Input path {path:?} does not exist"),
        )));
    }

    if !path.is_file() {
        return Err(Error::from(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Input path {path:?} is not a file"),
        )));
    }

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

    let content = fs::read_to_string(&args.input)?;

    let openapi: OpenAPI = if args.input.extension().is_some_and(|ext| ext == "yaml") {
        serde_yaml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };

    let (models, requests, responses, routes) = parser::parse_openapi(&openapi, args.get_path_params_from_path)?;

    // this one is string vec:
    //GETRefTaxonFormsSpeciesCodeResponseArrayObject200

    // this one is a type alias...
    let models_to_skip: Vec<String> = vec![
        "GETRefTaxonFormsSpeciesCodeResponseArrayObject200".to_string(),
        "GETDataObsRegionCodeRecentResponseArrayObject200".to_string(),
        "GETDataObsRegionCodeRecentSpeciesCodeResponseArrayObject200".to_string(),
        "GETDataObsGeoRecentResponseArrayObject200".to_string(),
        "GETRefRegionListRegionTypeParentRegionCodeResponseArrayObject200".to_string(),
        "GETDataObsGeoRecentSpeciesCodeResponseArrayObject200".to_string(),
        "GETDataNearestGeoRecentSpeciesCodeResponseArrayObject200".to_string(),
        "GETDataObsGeoRecentNotableResponseArrayObject200".to_string(),
        "GETDataObsRegionCodeHistoricYMDResponseArrayObject200".to_string(),
        "Parent".to_string(),
    ];

    let mut type_name_replacements: IndexMap<String, String> = IndexMap::new();
    type_name_replacements.insert("Vec<GETDataObsRegionCodeRecentNotableResponseArrayObject200>".to_string(), "Vec<Observation>".to_string());
    type_name_replacements.insert("GETDataObsRegionCodeRecentNotableResponseArrayObject200".to_string(), "Observation".to_string());
    
    type_name_replacements.insert("Vec<GETDataObsRegionCodeRecentSpeciesCodeResponseArrayObject200>".to_string(), "Vec<Observation>".to_string());
    type_name_replacements.insert("GETDataObsRegionCodeRecentSpeciesCodeResponseArrayObject200".to_string(), "Observation".to_string());
    
    type_name_replacements.insert("Vec<GETDataObsGeoRecentResponseArrayObject200>".to_string(), "Vec<Observation>".to_string());
    type_name_replacements.insert("GETDataObsGeoRecentResponseArrayObject200".to_string(), "Observation".to_string());
    
    type_name_replacements.insert("Vec<GETDataObsRegionCodeRecentResponseArrayObject200>".to_string(), "Vec<Observation>".to_string());
    type_name_replacements.insert("GETDataObsRegionCodeRecentResponseArrayObject200".to_string(), "Observation".to_string());
    type_name_replacements.insert("GETProductChecklistViewSubIdResponse200".to_string(), "Checklist".to_string());
    
    type_name_replacements.insert("Vec<GETRefAdjacentRegionCodeResponseArrayObject200>".to_string(), "Vec<RegionCode>".to_string());
    type_name_replacements.insert("GETRefAdjacentRegionCodeResponseArrayObject200".to_string(), "RegionCode".to_string());
    
    type_name_replacements.insert("Vec<GETRefRegionListRegionTypeParentRegionCodeResponseArrayObject200>".to_string(), "Vec<RegionCode>".to_string());
    
    type_name_replacements.insert("GETRefHotspotInfoLocIdResponse200".to_string(), "Hotspot".to_string());
    type_name_replacements.insert("Vec<GETRefTaxaLocalesEbirdResponseArrayObject200>".to_string(), "Vec<TaxaLocale>".to_string());
    type_name_replacements.insert("Vec<GETRefTaxonomyVersionsResponseArrayObject200>".to_string(), "Vec<TaxonomyVersion>".to_string());
    type_name_replacements.insert("Vec<GETRefSppgroupSpeciesGroupingResponseArrayObject200>".to_string(), "Vec<TaxonomicGroup>".to_string());
    type_name_replacements.insert("GETRefRegionInfoRegionCodeResponse200".to_string(), "RegionInfo".to_string());
    type_name_replacements.insert("Vec<GETRefTaxonFormsSpeciesCodeResponseArrayObject200>".to_string(), "Vec<String>".to_string());
    type_name_replacements.insert("Vec<GETDataObsGeoRecentSpeciesCodeResponseArrayObject200>".to_string(), "Vec<Observation>".to_string());
    type_name_replacements.insert("Vec<GETDataNearestGeoRecentSpeciesCodeResponseArrayObject200>".to_string(), "Vec<Observation>".to_string());
    type_name_replacements.insert("Vec<GETDataObsGeoRecentNotableResponseArrayObject200>".to_string(), "Vec<Observation>".to_string());
    type_name_replacements.insert("Vec<GETDataObsRegionCodeHistoricYMDResponseArrayObject200>".to_string(), "Vec<Observation>".to_string());

    type_name_replacements.insert("GETRefTaxaLocalesEbirdResponseArrayObject200".to_string(), "TaxaLocale".to_string());
    type_name_replacements.insert("GETRefTaxonomyVersionsResponseArrayObject200".to_string(), "TaxonomyVersion".to_string());
    type_name_replacements.insert("GETRefSppgroupSpeciesGroupingResponseArrayObject200".to_string(), "TaxonomicGroup".to_string());
    type_name_replacements.insert("GETRefTaxonFormsSpeciesCodeResponseArrayObject200".to_string(), "String".to_string());
    type_name_replacements.insert("GETDataObsGeoRecentSpeciesCodeResponseArrayObject200".to_string(), "Observation".to_string());
    type_name_replacements.insert("GETDataNearestGeoRecentSpeciesCodeResponseArrayObject200".to_string(), "Observation".to_string());
    type_name_replacements.insert("GETDataObsGeoRecentNotableResponseArrayObject200".to_string(), "Observation".to_string());
    type_name_replacements.insert("Parent".to_string(), "Box<RegionInfo>".to_string());

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
