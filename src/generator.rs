use crate::{models::{Model, RequestModel, ResponseModel}, Result};

const RUST_RESERVED_KEYWORDS: &[&str] = &[
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while",

    "abstract", "become", "box", "do", "final", "macro", "override", "priv", "try",
    "typeof", "unsized", "virtual", "yield",
];

const EMPTY_RESPONSE_NAME: &str = "UnknownResponse";
const EMPTY_REQUEST_NAME: &str = "UnknownRequest";

fn is_reserved_word(string_to_check: &str) -> bool {
    RUST_RESERVED_KEYWORDS.contains(&string_to_check)
}

pub fn generate_models(
    models: &[Model],
    requests: &[RequestModel],
    responses: &[ResponseModel],
) -> Result<String> {
    let mut output = String::new();

    output.push_str("use serde::{Serialize, Deserialize};\n");
    output.push_str("use uuid::Uuid;\n");
    output.push_str("use chrono::{DateTime, NaiveDate, Utc};\n\n");

    for model in models {
        output.push_str(&generate_model(model)?);
        output.push('\n');
    }

    for request in requests {
        output.push_str(&generate_request_model(request)?);
        output.push('\n');
    }

    for response in responses {
        output.push_str(&generate_response_model(response)?);
        output.push('\n');
    }

    Ok(output)
}

fn generate_model(model: &Model) -> Result<String> {
    let mut output = String::new();

    if !model.name.is_empty() {
        output.push_str(&format!("/// {}\n", model.name));
    }
    
    output.push_str(&format!("#[derive(Debug, Serialize, Deserialize)]\n"));
    output.push_str(&format!("pub struct {} {{\n", model.name));

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

        let mut lowercased_name = field.name.to_lowercase();
        if is_reserved_word(&lowercased_name) {
            lowercased_name = format!("r#{}", lowercased_name)
        }

        output.push_str(&format!(
            "    #[serde(rename = \"{}\")]\n",
            field.name.to_lowercase()
        ));

        if field.is_required {
            output.push_str(&format!(
                "    pub {}: {},\n",
                lowercased_name,
                field_type
            ));
        } else {
            output.push_str(&format!(
                "    pub {}: Option<{}>,\n",
                lowercased_name,
                field_type
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
    output.push_str(&"#[derive(Debug, Serialize)]\n".to_string());
    output.push_str(&format!("pub struct {} {{\n", request.name));
    output.push_str(&"    pub content_type: String,\n".to_string());
    output.push_str(&format!("    pub body: {},\n", request.schema));
    output.push_str("}\n");
    Ok(output)
}

fn generate_response_model(response: &ResponseModel) -> Result<String> {
    let mut output = String::new();
    
    // Return if name is empty
    if response.name.is_empty() || response.name == EMPTY_RESPONSE_NAME {
        return Ok(String::new());
    }

    output.push_str(&format!("/// {}\n", response.name));
    output.push_str(&"#[derive(Debug, Deserialize)]\n".to_string());
    output.push_str(&format!("pub struct {} {{\n", response.name));
    output.push_str(&"    pub status_code: String,\n".to_string());
    output.push_str(&"    pub content_type: String,\n".to_string());
    output.push_str(&format!("    pub body: {},\n", response.schema));
    if let Some(desc) = &response.description {
        output.push_str(&format!("    /// {}\n", desc));
    }
    output.push_str("}\n");
    Ok(output)
}


pub fn generate_rust_code(models: &[Model]) -> Result<String> {
    let mut code = String::new();

    code.push_str("use serde::{Serialize, Deserialize};\n");
    code.push_str("use uuid::Uuid;\n");
    code.push_str("use chrono::{DateTime, NaiveDate, Utc};\n\n");

    for model in models {
        code.push_str(&format!("/// {}\n", model.name));
        code.push_str(&"#[derive(Debug, Serialize, Deserialize)]\n".to_string());
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

            let mut lowercased_name = field.name.to_lowercase();
            if is_reserved_word(&lowercased_name) {
                lowercased_name = format!("r#{}", lowercased_name)
            }

            code.push_str(&format!(
                "    #[serde(rename = \"{}\")]\n",
                field.name.to_lowercase()
            ));
            
            if field.is_required {
                code.push_str(&format!(
                    "    pub {}: {},\n",
                    lowercased_name,
                    field_type
                ));
            } else {
                code.push_str(&format!(
                    "    pub {}: Option<{}>,\n",
                    lowercased_name,
                    field_type
                ));
            }
        }

        code.push_str("}\n\n");
    }

    Ok(code)
}

pub fn generate_lib() -> Result<String> {
    let mut code = String::new();
    code.push_str("pub mod models;\n");

    Ok(code)
}