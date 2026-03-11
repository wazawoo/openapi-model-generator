use std::sync::OnceLock;

use indexmap::IndexMap;
use openapiv3::{Operation, ReferenceOr};

use crate::{
    Result, models::{
        CompositionModel, EnumModel, Model, ModelType, RequestModel, ResponseModel, RouteModel, TypeAliasModel, UnionModel, UnionType
    }
};

bitflags::bitflags! {
    struct RequiredUses: u8 {
        const UUID = 0b00000001;
        const DATETIME = 0b00000010;
        const DATE = 0b00000100;
    }
}

static HDR: OnceLock<String> = OnceLock::new();

fn create_header() -> String {
    HDR.get_or_init(|| {
        format!(
            r#"
//!
//! Generated from an OAS specification by {}(v{})
//!

"#,
            option_env!("CARGO_PKG_NAME").unwrap_or("openapi-model-generator"),
            option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
        )
    })
    .clone()
}

const RUST_RESERVED_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try",
    "typeof", "unsized", "virtual", "yield",
];

const EMPTY_RESPONSE_NAME: &str = "UnknownResponse";
const EMPTY_REQUEST_NAME: &str = "UnknownRequest";

fn is_reserved_word(string_to_check: &str) -> bool {
    RUST_RESERVED_KEYWORDS.contains(&string_to_check.to_lowercase().as_str())
}

fn generate_description_docs(
    description: &Option<String>,
    fallback_str: &str,
    indent: &str,
) -> String {
    let mut output = String::new();
    if let Some(desc) = description {
        for line in desc.lines() {
            output.push_str(&format!("{}/// {}\n", indent, line.trim()));
        }
    } else if !fallback_str.is_empty() {
        output.push_str(&format!("{}/// {}\n", indent, fallback_str));
    }

    output
}

fn to_snake_case(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    let mut snake = String::new();

    for (i, c) in cleaned.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i != 0 {
                snake.push('_');
            }
            snake.push(c.to_ascii_lowercase());
        } else {
            snake.push(c);
        }
    }
    snake = snake.replace("__", "_");

    if snake == "self" {
        snake.push('_');
    }

    if snake
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        snake = format!("_{snake}");
    }

    snake
}

/// Checks if custom attributes contain a derive attribute
fn has_custom_derive(custom_attrs: &Option<Vec<String>>) -> bool {
    if let Some(attrs) = custom_attrs {
        attrs
            .iter()
            .any(|attr| attr.trim().starts_with("#[derive("))
    } else {
        false
    }
}

/// Checks if custom attributes contain a serde attribute
fn has_custom_serde(custom_attrs: &Option<Vec<String>>) -> bool {
    if let Some(attrs) = custom_attrs {
        attrs.iter().any(|attr| attr.trim().starts_with("#[serde("))
    } else {
        false
    }
}

/// Generates custom attributes from x-rust-attrs
fn generate_custom_attrs(custom_attrs: &Option<Vec<String>>) -> String {
    if let Some(attrs) = custom_attrs {
        attrs
            .iter()
            .map(|attr| format!("{attr}\n"))
            .collect::<String>()
    } else {
        String::new()
    }
}

pub fn generate_models(
    models: &[ModelType],
    requests: &[RequestModel],
    responses: &[ResponseModel],
    models_to_skip: &[String],
    type_name_replacements: &IndexMap<String, String>
) -> Result<String> {
    // First, generate all model code to determine which imports are needed
    let mut models_code = String::new();
    let mut required_uses = RequiredUses::empty();

    for model_type in models {
        match model_type {
            ModelType::Struct(model) => {
                if models_to_skip.contains(&model.name) {
                    continue;
                }
                models_code.push_str(&generate_model(model, &mut required_uses, type_name_replacements)?);
            }
            ModelType::Union(union) => {
                models_code.push_str(&generate_union(union)?);
            }
            ModelType::Composition(comp) => {
                models_code.push_str(&generate_composition(comp, &mut required_uses)?);
            }
            ModelType::Enum(enum_model) => {
                models_code.push_str(&generate_enum(enum_model)?);
            }
            ModelType::TypeAlias(type_alias) => {
                if models_to_skip.contains(&type_alias.name) {
                    continue;
                }
                models_code.push_str(&generate_type_alias(type_alias)?);
            }
        }
    }

    for request in requests {
        models_code.push_str(&generate_request_model(request)?);
    }

    for response in responses {
        models_code.push_str(&generate_response_model(response, &type_name_replacements)?);
    }

    // Determine which imports are actually needed
    let needs_uuid = required_uses.contains(RequiredUses::UUID);
    let needs_datetime = required_uses.contains(RequiredUses::DATETIME);
    let needs_date = required_uses.contains(RequiredUses::DATE);

    // Build final output with only necessary imports
    let mut output = create_header();
    output.push_str("use serde::{Serialize, Deserialize};\n");

    if needs_uuid {
        output.push_str("use uuid::Uuid;\n");
    }

    if needs_datetime || needs_date {
        output.push_str("use chrono::{");
        let mut chrono_imports = Vec::new();
        if needs_datetime {
            chrono_imports.push("DateTime");
        }
        if needs_date {
            chrono_imports.push("NaiveDate");
        }
        if needs_datetime {
            chrono_imports.push("Utc");
        }
        output.push_str(&chrono_imports.join(", "));
        output.push_str("};\n");
    }

    output.push('\n');
    output.push_str(&models_code);

    Ok(output)
}

fn generate_model(model: &Model, required_uses: &mut RequiredUses, type_name_replacements: &IndexMap<String, String>) -> Result<String> {
    let mut output = String::new();

    let model_type = if let Some(replacement) = type_name_replacements.get(&model.name) {
        replacement
    } else {
        &model.name
    };

    output.push_str(&generate_description_docs(
        &model.description,
        model_type,
        "",
    ));

    output.push_str(&generate_custom_attrs(&model.custom_attrs));

    // Only add default derive if custom_attrs doesn't already contain a derive directive
    if !has_custom_derive(&model.custom_attrs) {
        output.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    }

    output.push_str(&format!("pub struct {} {{\n", model_type));

    for field in &model.fields {
        let _field_type = match field.field_type.as_str() {
            "String" => "String",
            "f64" => "f64",
            "i64" => "i64",
            "bool" => "bool",
            "DateTime" => {
                *required_uses |= RequiredUses::DATETIME;
                "DateTime<Utc>"
            }
            "Date" => {
                *required_uses |= RequiredUses::DATE;
                "NaiveDate"
            }
            "Uuid" => {
                *required_uses |= RequiredUses::UUID;
                "Uuid"
            }
            _ => &field.field_type,
        };

        let field_type = if let Some(replacement) = type_name_replacements.get(_field_type) {
            replacement
        } else {
            _field_type
        };

        let mut lowercased_name = to_snake_case(field.name.as_str());
        if is_reserved_word(&lowercased_name) {
            lowercased_name = format!("r#{lowercased_name}")
        }

        // Add field description if present
        output.push_str(&generate_description_docs(&field.description, "", "    "));

        // Only add serde rename if the Rust field name differs from the original field name
        if lowercased_name != field.name {
            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.name));
        }

        if field.should_flatten() {
            output.push_str("    #[serde(flatten)]\n");
        }

        // If field references an array, wrap it in Vec<>
        if field.is_array_ref {
            if field.is_required && !field.is_nullable {
                output.push_str(&format!("    pub {lowercased_name}: Vec<{field_type}>,\n",));
            } else {
                output.push_str(&format!(
                    "    pub {lowercased_name}: Option<Vec<{field_type}>>,\n",
                ));
            }
        } else if field.is_required && !field.is_nullable {
            output.push_str(&format!("    pub {lowercased_name}: {field_type},\n",));
        } else {
            output.push_str(&format!(
                "    pub {lowercased_name}: Option<{field_type}>,\n",
            ));
        }
    }

    output.push_str("}\n\n");
    Ok(output)
}

fn generate_request_model(request: &RequestModel) -> Result<String> {
    let mut output = String::new();
    tracing::info!("Generating request model");
    tracing::info!("{:#?}", request);

    if request.name.is_empty() || request.name == EMPTY_REQUEST_NAME {
        return Ok(String::new());
    }

    output.push_str(&format!("/// {}\n", request.name));
    output.push_str("#[derive(Debug, Clone, Serialize)]\n");
    output.push_str(&format!("pub struct {} {{\n", request.name));
    output.push_str(&format!("    pub body: {},\n", request.schema));
    output.push_str("}\n");
    Ok(output)
}

fn generate_response_model(response: &ResponseModel, type_name_replacements: &IndexMap<String, String>) -> Result<String> {
    if response.name.is_empty() || response.name == EMPTY_RESPONSE_NAME {
        return Ok(String::new());
    }

    let type_name = format!("{}{}", response.name, response.status_code);

    let response_type = if let Some(replacement) = type_name_replacements.get(&response.schema) {
        replacement
    } else {
        &response.schema
    };

    let mut output = String::new();

    output.push_str(&generate_description_docs(
        &response.description,
        &type_name,
        "",
    ));

    output.push_str("#[derive(Debug, Clone, Deserialize)]\n");
    output.push_str(&format!("pub struct {type_name} {{\n"));
    output.push_str(&format!("    pub body: {},\n", response_type));
    output.push_str("}\n");

    Ok(output)
}

fn generate_union(union: &UnionModel) -> Result<String> {
    let mut output = String::new();

    output.push_str(&format!(
        "/// {} ({})\n",
        union.name,
        match union.union_type {
            UnionType::OneOf => "oneOf",
            UnionType::AnyOf => "anyOf",
        }
    ));
    output.push_str(&generate_custom_attrs(&union.custom_attrs));

    // Only add default derive if custom_attrs doesn't already contain a derive
    if !has_custom_derive(&union.custom_attrs) {
        output.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    }

    // Only add default serde(untagged) if custom_attrs doesn't already contain a serde attribute
    if !has_custom_serde(&union.custom_attrs) {
        output.push_str("#[serde(untagged)]\n");
    }

    output.push_str(&format!("pub enum {} {{\n", union.name));

    for variant in &union.variants {
        match &variant.primitive_type {
            Some(t) => output.push_str(&format!("    {}({}),\n", variant.name, t)),
            None => output.push_str(&format!("    {}({}),\n", variant.name, variant.name)),
        }
    }

    output.push_str("}\n");
    Ok(output)
}

fn generate_composition(
    comp: &CompositionModel,
    required_uses: &mut RequiredUses,
) -> Result<String> {
    let mut output = String::new();

    output.push_str(&format!("/// {} (allOf composition)\n", comp.name));
    output.push_str(&generate_custom_attrs(&comp.custom_attrs));

    // Only add default derive if custom_attrs doesn't already contain a derive
    if !has_custom_derive(&comp.custom_attrs) {
        output.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    }

    output.push_str(&format!("pub struct {} {{\n", comp.name));

    for field in &comp.all_fields {
        let field_type = match field.field_type.as_str() {
            "String" => "String",
            "f64" => "f64",
            "i64" => "i64",
            "bool" => "bool",
            "DateTime" => {
                *required_uses |= RequiredUses::DATETIME;
                "DateTime<Utc>"
            }
            "Date" => {
                *required_uses |= RequiredUses::DATE;
                "NaiveDate"
            }
            "Uuid" => {
                *required_uses |= RequiredUses::UUID;
                "Uuid"
            }
            _ => &field.field_type,
        };

        let mut lowercased_name = to_snake_case(field.name.as_str());
        if is_reserved_word(&lowercased_name) {
            lowercased_name = format!("r#{lowercased_name}");
        }

        // Only add serde rename if the Rust field name differs from the original field name
        if lowercased_name != field.name {
            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.name));
        }

        // If field references an array, wrap it in Vec<>
        if field.is_array_ref {
            if field.is_required && !field.is_nullable {
                output.push_str(&format!("    pub {lowercased_name}: Vec<{field_type}>,\n",));
            } else {
                output.push_str(&format!(
                    "    pub {lowercased_name}: Option<Vec<{field_type}>>,\n",
                ));
            }
        } else if field.is_required && !field.is_nullable {
            output.push_str(&format!("    pub {lowercased_name}: {field_type},\n",));
        } else {
            output.push_str(&format!(
                "    pub {lowercased_name}: Option<{field_type}>,\n",
            ));
        }
    }

    output.push_str("}\n");
    Ok(output)
}

fn generate_enum(enum_model: &EnumModel) -> Result<String> {
    let mut output = String::new();

    output.push_str(&generate_description_docs(
        &enum_model.description,
        &enum_model.name,
        "",
    ));

    output.push_str(&generate_custom_attrs(&enum_model.custom_attrs));

    // Only add default derive if custom_attrs doesn't already contain a derive
    if !has_custom_derive(&enum_model.custom_attrs) {
        output.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    }

    output.push_str(&format!("pub enum {} {{\n", enum_model.name));

    for (i, variant) in enum_model.variants.iter().enumerate() {
        let original = variant.clone();

        let mut rust_name = crate::parser::to_pascal_case(variant);

        let serde_rename = if is_reserved_word(&rust_name) {
            rust_name.push_str("Value");
            Some(original)
        } else if rust_name != original {
            Some(original)
        } else {
            None
        };

        if let Some(rename) = serde_rename {
            output.push_str(&format!("    #[serde(rename = \"{rename}\")]\n"));
        }

        if i + 1 == enum_model.variants.len() {
            output.push_str(&format!("    {rust_name}\n"));
        } else {
            output.push_str(&format!("    {rust_name},\n"));
        }
    }

    output.push_str("}\n");
    Ok(output)
}

fn generate_type_alias(type_alias: &TypeAliasModel) -> Result<String> {
    let mut output = String::new();

    output.push_str(&generate_description_docs(
        &type_alias.description,
        &type_alias.name,
        "",
    ));

    output.push_str(&generate_custom_attrs(&type_alias.custom_attrs));
    output.push_str(&format!(
        "pub type {} = {};\n\n",
        type_alias.name, type_alias.target_type
    ));

    Ok(output)
}

pub fn generate_routes(
    routes: &[RouteModel],
    type_name_replacements: &IndexMap<String, String>,
) -> Result<String> {
    let mut routes_output = "".to_string();

    for route in routes {
        let route_output = generate_route_model(
            &route,
            type_name_replacements.clone()
        )?;
        routes_output.push_str(&route_output);
    }

    // write fully clears the file, so i can only call it once
    let mut file_contents = "".to_string();

    file_contents.push_str("use reqwest::Method;\n");
    file_contents.push_str("use reqwest::header::HeaderMap;\n");
    file_contents.push_str("use reqwest::header::HeaderName;\n");
    file_contents.push_str("use reqwest::header::HeaderValue;\n\n");
    file_contents.push_str("use crate::models::*;\n");
    file_contents.push_str("use crate::bird_client::BirdRequest;\n\n");

    file_contents.push_str(&routes_output);
    Ok(file_contents)
}

pub fn create_route_model(
    path: String,
    backup_name: String,
    method: String,
    response_schema: String,
    op: &Operation,
) -> Result<RouteModel> {
    // let mut q_params_string = "".to_string();
    // let mut h_params_string = "".to_string();
    // let mut p_params_string = "".to_string();
    // let mut c_params_string = "".to_string();
    
    let mut query_params: IndexMap<String, String> = IndexMap::new();
    let mut additional_headers: IndexMap<String, String> = IndexMap::new();

    for param in op.parameters.clone() {
        if let ReferenceOr::Item(_param) = param {
            match _param {
                openapiv3::Parameter::Query { parameter_data, allow_reserved: _, style: _, allow_empty_value: _} => {
                    // dbg!(param);
                    let rust_name = to_snake_case(&parameter_data.name);
                    query_params.insert(
                        parameter_data.name, 
                        rust_name
                    );
                },
                openapiv3::Parameter::Header { parameter_data, style: _ } => {
                   let rust_name = parameter_data.name
                        .replace('-',"_")
                        .to_lowercase();
                    additional_headers.insert(
                        parameter_data.name,
                        rust_name
                    );
                },
                openapiv3::Parameter::Path { parameter_data: _, style: _ } => {
                    // let name = parameter_data.name;
                    // will want these in a moment... once include path params in schema
                    // p_params_string.push_str(&format!("P:{}, ", name).to_string());
                },
                openapiv3::Parameter::Cookie { parameter_data: _, style: _ } => {
                    // let name = parameter_data.name;
                    // c_params_string.push_str(&format!("C:{}, ", name).to_string());
                },
            }
        }
    }

    // maybe unique to my dataset, but grabbing path params from path string (they arent in op.params above)
    let mut format_path = "".to_string();
    let mut path_params: IndexMap<String, String> = IndexMap::new();
    let mut in_param = false;
    let mut current_param = "".to_string();
    for ch in path.chars() {
        match ch {
            '{' => {
                in_param = true;
                current_param = "".to_string();
                format_path.push(ch);
            },
            '}' => {
                in_param = false;
                let rust_name = to_snake_case(&current_param);
                path_params.insert(current_param.clone(), rust_name);
                format_path.push(ch);
            },
            _ => {
                if in_param {
                    current_param.push(ch);
                } else {
                    format_path.push(ch);
                }
            }
        }
    }

    dbg!(&response_schema);

    Ok(
        RouteModel { 
            path: path.to_string(), 
            backup_name,
            method,
            format_path,
            query_params,
            path_params,
            additional_headers,
            response_schema
        }
    )
}

pub fn generate_route_model(
    route: &RouteModel,
    type_name_replacements: IndexMap<String, String>
) -> Result<String> {
    let mut route_output = String::new();
    // // ignoring p params and c params for now...
    // let mut p_params_string = "".to_string();
    // let mut c_params_string = "".to_string();

    // Query params
    let mut q_params_string = "".to_string();
    for (q_param, _rust_name) in &route.query_params {
        // this is just one substitution. rightmost brackets are escaped.
        // after substitution looks like: "?qParam={}"
        if q_params_string.is_empty() {
            q_params_string.push_str(&format!("?{}={{}}", q_param).to_string());
        } else {
            q_params_string.push_str(&format!("&{}={{}}", q_param).to_string());
        }
    }

    // string of args for "format!()"
    let mut format_path = format!("\"{}{}\"", route.format_path, q_params_string);
    for (_param, rust_name) in route.path_params.clone() {
        format_path.push_str(&format!(", self.{}", rust_name));
    }
    for (_param, rust_name) in route.query_params.clone() {
        format_path.push_str(&format!(", self.{}", rust_name));
    }
    
    let func_name = &route.backup_name;
    let tab = "    ";

    let response_type = if let Some(replacement) = type_name_replacements.get(&route.response_schema) {
        replacement
    } else {
        &route.response_schema
    };

    // request model
    route_output.push_str(&format!("pub struct {} {{\n", func_name));
    if !route.path_params.is_empty() {
        route_output.push_str(&format!("{}// path params: {:?} \n", tab, route.path_params.keys()));
        for (_, rust_name) in &route.path_params {
            route_output.push_str(&format!("{}pub {}: String,\n", tab, rust_name));
        }
    }
    if !q_params_string.is_empty() {
        route_output.push_str(&format!("{}// q params: {} \n", tab, q_params_string));
        for (_, rust_name) in &route.query_params {
            route_output.push_str(&format!("{}pub {}: String,\n", tab, rust_name));
        }
    }
    if !route.additional_headers.is_empty() {
        route_output.push_str(&format!("{}// headers: {:?} \n", tab, route.additional_headers.keys()));
        for (_, rust_name) in &route.additional_headers {
            route_output.push_str(&format!("{}pub {}: String,\n", tab, rust_name));
        }
    }
    route_output.push_str("}\n");

    // if !p_params_string.is_empty() {
    //     route_output.push_str(&format!("{}// p params: {} \n", tab, p_params_string));
    // }
    // if !c_params_string.is_empty() {
    //     route_output.push_str(&format!("{}// c params: {} \n", tab, c_params_string));
    // }

    // put params here. some url, some path. should have type too...
    route_output.push_str(&format!("impl BirdRequest for {} {{\n", func_name));
    route_output.push_str(&format!("{}type ResponseType = {};\n", tab, response_type));
    route_output.push_str(&format!("{}const METHOD: Method = Method::{};\n", tab, route.method));

    // if res type is raw string, we need to decode differently
    if response_type == "String" {
        route_output.push_str(&format!("{}const RETURNS_CSV: bool = true;\n", tab));
    }

    // path()
    route_output.push_str(&format!("{}fn path(&self) -> String {{\n", tab));
    route_output.push_str(&format!("{}{}format!({})\n", tab, tab, format_path));
    route_output.push_str(&format!("{}}}\n", tab));

    // headers (if any)
    route_output.push_str(&format!("{}fn additional_headers(&self) -> HeaderMap {{\n", tab));
    if route.additional_headers.is_empty() {
        route_output.push_str(&format!("{}{}HeaderMap::new()\n", tab, tab));
    } else {
        route_output.push_str(&format!("{}{}let mut map = HeaderMap::new();\n", tab, tab));
        for (header_param, rust_name) in &route.additional_headers {
            route_output.push_str(&format!("{}{}if let (Ok(name), Ok(value)) = (\n", tab, tab));
            route_output.push_str(&format!("{}{}{}HeaderName::from_bytes(\"{}\".as_bytes()),\n", tab, tab, tab, header_param));
            route_output.push_str(&format!("{}{}{}HeaderValue::from_str(&self.{})\n", tab, tab, tab, rust_name));
            route_output.push_str(&format!("{}{}) {{\n", tab, tab));
            route_output.push_str(&format!("{}{}{}map.insert(name, value);\n", tab, tab, tab));
            route_output.push_str(&format!("{}{}}}\n", tab, tab));
        }
        route_output.push_str(&format!("{}{}map\n", tab, tab));
    }
    route_output.push_str(&format!("{}}}\n", tab));

    route_output.push_str("}\n\n");
    Ok(route_output)
}

pub fn generate_rust_code(models: &[Model]) -> Result<String> {
    let mut code = create_header();

    code.push_str("use serde::{Serialize, Deserialize};\n");
    code.push_str("use uuid::Uuid;\n");
    code.push_str("use chrono::{DateTime, NaiveDate, Utc};\n\n");

    for model in models {
        code.push_str(&format!("/// {}\n", model.name));
        code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
        code.push_str(&format!("pub struct {} {{\n", model.name));

        for field in &model.fields {
            let field_type = match field.field_type.as_str() {
                "String" => "String",
                "f64" => "f64",
                "i64" => "i64",
                "bool" => "bool",
                "DateTime" => "DateTime<Utc>",
                "Date" => "NaiveDate",
                "Uuid" => "Uuid",
                _ => &field.field_type,
            };

            let mut lowercased_name = to_snake_case(field.name.as_str());
            if is_reserved_word(&lowercased_name) {
                lowercased_name = format!("r#{lowercased_name}")
            }

            // Only add serde rename if the Rust field name differs from the original field name
            if lowercased_name != field.name {
                code.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.name));
            }

            if field.is_required {
                code.push_str(&format!("    pub {lowercased_name}: {field_type},\n",));
            } else {
                code.push_str(&format!(
                    "    pub {lowercased_name}: Option<{field_type}>,\n",
                ));
            }
        }

        code.push_str("}\n\n");
    }

    Ok(code)
}

pub fn generate_lib() -> Result<String> {
    let mut code = create_header();
    code.push_str("pub mod models;\n");
    code.push_str("pub mod routes;\n");

    Ok(code)
}

pub fn generate_readme(
    models: &[ModelType],
    requests: &[RequestModel],
    responses: &[ResponseModel],
    models_to_skip: &[String],
    type_name_replacements: &IndexMap<String, String>
) -> Result<String> {
    let mut code = "```\n".to_string();
    code.push_str(&create_header());
    code.push_str("```\n");

    // beef goes here...

    code.push_str("# Replacements and Omissions\n");
    code.push_str("## Models to skip\n");
    for model in models_to_skip {
        code.push_str(&format!("- `{}`\n", model));
    }
    code.push_str("## Models type replacements\n");
    code.push_str("| old | -> | new |\n");
    code.push_str("| --- | --- | --- |\n");

    for (from, to) in type_name_replacements {
        code.push_str(&format!("| `{}` | -> | `{}` |\n", from, to));
    }

    Ok(code)
}
