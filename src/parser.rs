use crate::{
    models::{
        CompositionModel, Field, Model, ModelType, RequestModel, ResponseModel, UnionModel,
        UnionType, UnionVariant,
    },
    Result,
};
use indexmap::IndexMap;
use openapiv3::{
    OpenAPI, ReferenceOr, Schema, SchemaKind, StringFormat, Type, VariantOrUnknownOrEmpty,
};

/// Information about a field extracted from OpenAPI schema
#[derive(Debug)]
struct FieldInfo {
    field_type: String,
    format: String,
    is_nullable: bool,
}

/// Converts camelCase to PascalCase
/// Example: "createRole" -> "CreateRole", "listRoles" -> "ListRoles", "listRoles-Input" -> "ListRolesInput"
fn to_pascal_case(input: &str) -> String {
    input
        .split(&['-', '_'][..]) // split on '-' or '_'
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

pub fn parse_openapi(
    openapi: &OpenAPI,
) -> Result<(Vec<ModelType>, Vec<RequestModel>, Vec<ResponseModel>)> {
    let mut models = Vec::new();
    let mut requests = Vec::new();
    let mut responses = Vec::new();

    // Parse components/schemas
    if let Some(components) = &openapi.components {
        for (name, schema) in &components.schemas {
            if let Some(model_type) = parse_schema_to_model_type(name, schema, &components.schemas)?
            {
                models.push(model_type);
            }
        }
    }

    // Parse paths
    for (_path, path_item) in openapi.paths.iter() {
        let path_item = match path_item {
            ReferenceOr::Item(item) => item,
            ReferenceOr::Reference { .. } => continue,
        };

        if let Some(op) = &path_item.get {
            process_operation(op, &mut requests, &mut responses)?;
        }
        if let Some(op) = &path_item.post {
            process_operation(op, &mut requests, &mut responses)?;
        }
        if let Some(op) = &path_item.put {
            process_operation(op, &mut requests, &mut responses)?;
        }
        if let Some(op) = &path_item.delete {
            process_operation(op, &mut requests, &mut responses)?;
        }
        if let Some(op) = &path_item.patch {
            process_operation(op, &mut requests, &mut responses)?;
        }
    }

    Ok((models, requests, responses))
}

fn process_operation(
    operation: &openapiv3::Operation,
    requests: &mut Vec<RequestModel>,
    responses: &mut Vec<ResponseModel>,
) -> Result<()> {
    // Parse request body
    if let Some(ReferenceOr::Item(request_body)) = &operation.request_body {
        for (content_type, media_type) in &request_body.content {
            if let Some(schema) = &media_type.schema {
                let request = RequestModel {
                    name: format!(
                        "{}Request",
                        to_pascal_case(operation.operation_id.as_deref().unwrap_or("Unknown"))
                    ),
                    content_type: content_type.clone(),
                    schema: extract_type_and_format(schema)?.0,
                    is_required: request_body.required,
                };
                requests.push(request);
            }
        }
    }

    // Parse responses
    for (status, response_ref) in operation.responses.responses.iter() {
        if let ReferenceOr::Item(response) = response_ref {
            for (content_type, media_type) in &response.content {
                if let Some(schema) = &media_type.schema {
                    let response = ResponseModel {
                        name: format!(
                            "{}Response",
                            to_pascal_case(operation.operation_id.as_deref().unwrap_or("Unknown"))
                        ),
                        status_code: status.to_string(),
                        content_type: content_type.clone(),
                        schema: extract_type_and_format(schema)?.0,
                        description: Some(response.description.clone()),
                    };
                    responses.push(response);
                }
            }
        }
    }

    Ok(())
}

fn parse_schema_to_model_type(
    name: &str,
    schema: &ReferenceOr<Schema>,
    all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<Option<ModelType>> {
    match schema {
        ReferenceOr::Reference { .. } => Ok(None),
        ReferenceOr::Item(schema) => {
            match &schema.schema_kind {
                // regular objects
                SchemaKind::Type(Type::Object(obj)) => {
                    let mut fields = Vec::new();
                    for (field_name, field_schema) in &obj.properties {
                        let field_info = match field_schema {
                            ReferenceOr::Item(boxed_schema) => {
                                extract_field_info(&ReferenceOr::Item((**boxed_schema).clone()))?
                            }
                            ReferenceOr::Reference { reference } => {
                                extract_field_info(&ReferenceOr::Reference {
                                    reference: reference.clone(),
                                })?
                            }
                        };

                        let is_required = obj.required.contains(field_name);
                        fields.push(Field {
                            name: field_name.clone(),
                            field_type: field_info.field_type,
                            format: field_info.format,
                            is_required,
                            is_nullable: field_info.is_nullable,
                        });
                    }
                    Ok(Some(ModelType::Struct(Model {
                        name: to_pascal_case(name),
                        fields,
                    })))
                }

                // allOf
                SchemaKind::AllOf { all_of } => {
                    let all_fields = resolve_all_of_fields(name, all_of, all_schemas)?;
                    Ok(Some(ModelType::Composition(CompositionModel {
                        name: to_pascal_case(name),
                        all_fields,
                    })))
                }

                // oneOf
                SchemaKind::OneOf { one_of } => {
                    let variants = resolve_union_variants(one_of, all_schemas)?;
                    Ok(Some(ModelType::Union(UnionModel {
                        name: to_pascal_case(name),
                        variants,
                        union_type: UnionType::OneOf,
                    })))
                }

                // anyOf
                SchemaKind::AnyOf { any_of } => {
                    let variants = resolve_union_variants(any_of, all_schemas)?;
                    Ok(Some(ModelType::Union(UnionModel {
                        name: to_pascal_case(name),
                        variants,
                        union_type: UnionType::AnyOf,
                    })))
                }

                _ => Ok(None),
            }
        }
    }
}

fn extract_type_and_format(schema: &ReferenceOr<Schema>) -> Result<(String, String)> {
    match schema {
        ReferenceOr::Reference { reference } => {
            let type_name = reference.split('/').next_back().unwrap_or("Unknown");
            Ok((to_pascal_case(type_name), "reference".to_string()))
        }
        ReferenceOr::Item(schema) => match &schema.schema_kind {
            SchemaKind::Type(Type::String(string_type)) => match &string_type.format {
                VariantOrUnknownOrEmpty::Item(fmt) => match fmt {
                    StringFormat::DateTime => {
                        Ok(("DateTime<Utc>".to_string(), "date-time".to_string()))
                    }
                    StringFormat::Date => Ok(("NaiveDate".to_string(), "date".to_string())),
                    _ => Ok(("String".to_string(), format!("{fmt:?}"))),
                },
                VariantOrUnknownOrEmpty::Unknown(unknown_format) => {
                    if unknown_format.to_lowercase() == "uuid" {
                        Ok(("Uuid".to_string(), "uuid".to_string()))
                    } else {
                        Ok(("String".to_string(), unknown_format.clone()))
                    }
                }
                _ => Ok(("String".to_string(), "string".to_string())),
            },
            SchemaKind::Type(Type::Integer(_)) => Ok(("i64".to_string(), "integer".to_string())),
            SchemaKind::Type(Type::Number(_)) => Ok(("f64".to_string(), "number".to_string())),
            SchemaKind::Type(Type::Boolean(_)) => Ok(("bool".to_string(), "boolean".to_string())),
            SchemaKind::Type(Type::Array(arr)) => {
                if let Some(items) = &arr.items {
                    let items_ref: &ReferenceOr<Box<Schema>> = items;
                    let (inner_type, _) = match items_ref {
                        ReferenceOr::Item(boxed_schema) => {
                            extract_type_and_format(&ReferenceOr::Item((**boxed_schema).clone()))?
                        }
                        ReferenceOr::Reference { reference } => {
                            extract_type_and_format(&ReferenceOr::Reference {
                                reference: reference.clone(),
                            })?
                        }
                    };
                    Ok((format!("Vec<{inner_type}>"), "array".to_string()))
                } else {
                    Ok(("Vec<serde_json::Value>".to_string(), "array".to_string()))
                }
            }
            SchemaKind::Type(Type::Object(obj)) => {
                if obj.properties.is_empty() {
                    Ok(("()".to_string(), "object".to_string()))
                } else {
                    Ok(("serde_json::Value".to_string(), "object".to_string()))
                }
            }
            _ => Ok(("serde_json::Value".to_string(), "unknown".to_string())),
        },
    }
}

/// Extracts field information including type, format, and nullable flag from OpenAPI schema
fn extract_field_info(schema: &ReferenceOr<Schema>) -> Result<FieldInfo> {
    let (field_type, format) = extract_type_and_format(schema)?;

    let is_nullable = match schema {
        ReferenceOr::Reference { .. } => false,
        ReferenceOr::Item(schema) => schema.schema_data.nullable,
    };

    Ok(FieldInfo {
        field_type,
        format,
        is_nullable,
    })
}

fn resolve_all_of_fields(
    _name: &str,
    all_of: &[ReferenceOr<Schema>],
    all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<Vec<Field>> {
    let mut all_fields = Vec::new();

    for schema_ref in all_of {
        match schema_ref {
            ReferenceOr::Reference { reference } => {
                if let Some(schema_name) = reference.strip_prefix("#/components/schemas/") {
                    if let Some(referenced_schema) = all_schemas.get(schema_name) {
                        let fields = extract_fields_from_schema(referenced_schema, all_schemas)?;
                        all_fields.extend(fields);
                    }
                }
            }
            ReferenceOr::Item(_schema) => {
                let fields = extract_fields_from_schema(schema_ref, all_schemas)?;
                all_fields.extend(fields);
            }
        }
    }

    Ok(all_fields)
}

fn resolve_union_variants(
    schemas: &[ReferenceOr<Schema>],
    all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<Vec<UnionVariant>> {
    let mut variants = Vec::new();

    for (index, schema_ref) in schemas.iter().enumerate() {
        match schema_ref {
            ReferenceOr::Reference { reference } => {
                if let Some(schema_name) = reference.strip_prefix("#/components/schemas/") {
                    if let Some(referenced_schema) = all_schemas.get(schema_name) {
                        let fields = extract_fields_from_schema(referenced_schema, all_schemas)?;
                        variants.push(UnionVariant {
                            name: to_pascal_case(schema_name),
                            fields,
                        });
                    }
                }
            }
            ReferenceOr::Item(_schema) => {
                let fields = extract_fields_from_schema(schema_ref, all_schemas)?;
                let variant_name = format!("Variant{index}");
                variants.push(UnionVariant {
                    name: variant_name,
                    fields,
                });
            }
        }
    }

    Ok(variants)
}

fn extract_fields_from_schema(
    schema_ref: &ReferenceOr<Schema>,
    _all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<Vec<Field>> {
    let mut fields = Vec::new();

    match schema_ref {
        ReferenceOr::Reference { .. } => Ok(fields),
        ReferenceOr::Item(schema) => {
            if let SchemaKind::Type(Type::Object(obj)) = &schema.schema_kind {
                for (field_name, field_schema) in &obj.properties {
                    let field_info = match field_schema {
                        ReferenceOr::Item(boxed_schema) => {
                            extract_field_info(&ReferenceOr::Item((**boxed_schema).clone()))?
                        }
                        ReferenceOr::Reference { reference } => {
                            extract_field_info(&ReferenceOr::Reference {
                                reference: reference.clone(),
                            })?
                        }
                    };

                    let is_required = obj.required.contains(field_name);
                    fields.push(Field {
                        name: field_name.clone(),
                        field_type: field_info.field_type,
                        format: field_info.format,
                        is_required,
                        is_nullable: field_info.is_nullable,
                    });
                }
            }
            Ok(fields)
        }
    }
}
