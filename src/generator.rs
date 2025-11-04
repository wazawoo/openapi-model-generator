use crate::{
    models::{
        CompositionModel, EnumModel, Model, ModelType, RequestModel, ResponseModel, UnionModel,
        UnionType,
    },
    Result,
};

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

pub fn generate_models(
    models: &[ModelType],
    requests: &[RequestModel],
    responses: &[ResponseModel],
) -> Result<String> {
    let mut output = String::new();

    output.push_str("use serde::{Serialize, Deserialize};\n");
    output.push_str("use uuid::Uuid;\n");
    output.push_str("use chrono::{DateTime, NaiveDate, Utc};\n\n");

    for model_type in models {
        match model_type {
            ModelType::Struct(model) => {
                output.push_str(&generate_model(model)?);
                output.push('\n');
            }
            ModelType::Union(union) => {
                output.push_str(&generate_union(union)?);
                output.push('\n');
            }
            ModelType::Composition(comp) => {
                output.push_str(&generate_composition(comp)?);
                output.push('\n');
            }
            ModelType::Enum(enum_model) => {
                output.push_str(&generate_enum(enum_model)?);
                output.push('\n');
            }
        }
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

    output.push_str("#[derive(Debug, Serialize, Deserialize)]\n");
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
            lowercased_name = format!("r#{lowercased_name}")
        }

        // Only add serde rename if the Rust field name differs from the original field name
        if lowercased_name != field.name {
            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.name));
        }

        if field.is_required && !field.is_nullable {
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
    output.push_str("#[derive(Debug, Serialize)]\n");
    output.push_str(&format!("pub struct {} {{\n", request.name));
    output.push_str("    pub content_type: String,\n");
    output.push_str(&format!("    pub body: {},\n", request.schema));
    output.push_str("}\n");
    Ok(output)
}

fn generate_response_model(response: &ResponseModel) -> Result<String> {
    if response.name.is_empty() || response.name == EMPTY_RESPONSE_NAME {
        return Ok(String::new());
    }

    let type_name = format!("{}{}", response.name, response.status_code);

    let mut output = String::new();

    if let Some(desc) = &response.description {
        for line in desc.lines() {
            output.push_str(&format!("/// {}\n", line.trim()));
        }
    } else {
        output.push_str(&format!("/// {type_name}\n"));
    }

    output.push_str("#[derive(Debug, Deserialize)]\n");
    output.push_str(&format!("pub struct {type_name} {{\n"));
    output.push_str(&format!("    pub body: {},\n", response.schema));
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
    output.push_str("#[derive(Debug, Serialize, Deserialize)]\n");
    output.push_str("#[serde(untagged)]\n");
    output.push_str(&format!("pub enum {} {{\n", union.name));

    for variant in &union.variants {
        output.push_str(&format!("    {}({}),\n", variant.name, variant.name));
    }

    output.push_str("}\n");
    Ok(output)
}

fn generate_composition(comp: &CompositionModel) -> Result<String> {
    let mut output = String::new();

    output.push_str(&format!("/// {} (allOf composition)\n", comp.name));
    output.push_str("#[derive(Debug, Serialize, Deserialize)]\n");
    output.push_str(&format!("pub struct {} {{\n", comp.name));

    for field in &comp.all_fields {
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
            lowercased_name = format!("r#{lowercased_name}");
        }

        // Only add serde rename if the Rust field name differs from the original field name
        if lowercased_name != field.name {
            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.name));
        }

        if field.is_required && !field.is_nullable {
            output.push_str(&format!("    pub {lowercased_name}: {field_type},\n"));
        } else {
            output.push_str(&format!(
                "    pub {lowercased_name}: Option<{field_type}>,\n"
            ));
        }
    }

    output.push_str("}\n");
    Ok(output)
}

fn generate_enum(enum_model: &EnumModel) -> Result<String> {
    let mut output = String::new();

    if let Some(description) = &enum_model.description {
        output.push_str(&format!("/// {description}\n"));
    } else {
        output.push_str(&format!("/// {}\n", enum_model.name));
    }

    output.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
    output.push_str(&format!("pub enum {} {{\n", enum_model.name));

    for (i, variant) in enum_model.variants.iter().enumerate() {
        let original = variant.clone();

        let mut chars = variant.chars();
        let first_char = chars.next().unwrap().to_ascii_uppercase();
        let rest: String = chars.collect();
        let mut rust_name = format!("{first_char}{rest}");

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

pub fn generate_rust_code(models: &[Model]) -> Result<String> {
    let mut code = String::new();

    code.push_str("use serde::{Serialize, Deserialize};\n");
    code.push_str("use uuid::Uuid;\n");
    code.push_str("use chrono::{DateTime, NaiveDate, Utc};\n\n");

    for model in models {
        code.push_str(&format!("/// {}\n", model.name));
        code.push_str("#[derive(Debug, Serialize, Deserialize)]\n");
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
    let mut code = String::new();
    code.push_str("pub mod models;\n");

    Ok(code)
}
