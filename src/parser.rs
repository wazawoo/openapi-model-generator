use crate::{
    models::{
        CompositionModel, EnumModel, Field, Model, ModelType, RequestModel, ResponseModel,
        UnionModel, UnionType, UnionVariant,
    },
    Result,
};
use indexmap::IndexMap;
use openapiv3::{
    OpenAPI, ReferenceOr, Schema, SchemaKind, StringFormat, Type, VariantOrUnknownOrEmpty,
};
use std::collections::HashSet;

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
        .split(&['-', '_'][..])
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

    let mut added_models = HashSet::new();

    // Parse components/schemas
    if let Some(components) = &openapi.components {
        for (name, schema) in &components.schemas {
            let model_types = parse_schema_to_model_type(name, schema, &components.schemas)?;
            for model_type in model_types {
                if added_models.insert(model_type.name().to_string()) {
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
                process_operation(op, &mut requests, &mut responses, &components.schemas)?;
            }
            if let Some(op) = &path_item.post {
                process_operation(op, &mut requests, &mut responses, &components.schemas)?;
            }
            if let Some(op) = &path_item.put {
                process_operation(op, &mut requests, &mut responses, &components.schemas)?;
            }
            if let Some(op) = &path_item.delete {
                process_operation(op, &mut requests, &mut responses, &components.schemas)?;
            }
            if let Some(op) = &path_item.patch {
                process_operation(op, &mut requests, &mut responses, &components.schemas)?;
            }
        }
    }

    Ok((models, requests, responses))
}

fn process_operation(
    operation: &openapiv3::Operation,
    requests: &mut Vec<RequestModel>,
    responses: &mut Vec<ResponseModel>,
    all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
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
                    schema: extract_type_and_format(schema, all_schemas)?.0,
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
                        schema: extract_type_and_format(schema, all_schemas)?.0,
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
) -> Result<Vec<ModelType>> {
    match schema {
        ReferenceOr::Reference { .. } => Ok(Vec::new()),
        ReferenceOr::Item(schema) => {
            match &schema.schema_kind {
                // regular objects
                SchemaKind::Type(Type::Object(obj)) => {
                    let mut fields = Vec::new();
                    let mut inline_models = Vec::new();
                    for (field_name, field_schema) in &obj.properties {
                        let (field_info, inline_model) = match field_schema {
                            ReferenceOr::Item(boxed_schema) => extract_field_info(
                                field_name,
                                &ReferenceOr::Item((**boxed_schema).clone()),
                                all_schemas,
                            )?,
                            ReferenceOr::Reference { reference } => extract_field_info(
                                field_name,
                                &ReferenceOr::Reference {
                                    reference: reference.clone(),
                                },
                                all_schemas,
                            )?,
                        };
                        if let Some(inline_model) = inline_model {
                            inline_models.push(inline_model);
                        }
                        let is_required = obj.required.contains(field_name);
                        fields.push(Field {
                            name: field_name.clone(),
                            field_type: field_info.field_type,
                            format: field_info.format,
                            is_required,
                            is_nullable: field_info.is_nullable,
                        });
                    }
                    let mut models = inline_models;
                    if !fields.is_empty() {
                        models.push(ModelType::Struct(Model {
                            name: to_pascal_case(name),
                            fields,
                        }));
                    }
                    Ok(models)
                }

                // allOf
                SchemaKind::AllOf { all_of } => {
                    let (all_fields, inline_models) =
                        resolve_all_of_fields(name, all_of, all_schemas)?;
                    let mut models = inline_models;

                    if !all_fields.is_empty() {
                        models.push(ModelType::Composition(CompositionModel {
                            name: to_pascal_case(name),
                            all_fields,
                        }));
                    }

                    Ok(models)
                }

                // oneOf
                SchemaKind::OneOf { one_of } => {
                    let (variants, inline_models) =
                        resolve_union_variants(name, one_of, all_schemas)?;
                    let mut models = inline_models;

                    models.push(ModelType::Union(UnionModel {
                        name: to_pascal_case(name),
                        variants,
                        union_type: UnionType::OneOf,
                    }));

                    Ok(models)
                }

                // anyOf
                SchemaKind::AnyOf { any_of } => {
                    let (variants, inline_models) =
                        resolve_union_variants(name, any_of, all_schemas)?;
                    let mut models = inline_models;

                    models.push(ModelType::Union(UnionModel {
                        name: to_pascal_case(name),
                        variants,
                        union_type: UnionType::AnyOf,
                    }));

                    Ok(models)
                }

                // enum strings
                SchemaKind::Type(Type::String(string_type)) => {
                    if !string_type.enumeration.is_empty() {
                        let variants: Vec<String> = string_type
                            .enumeration
                            .iter()
                            .filter_map(|value| {
                                value.clone()
                            })
                            .collect();

                        if !variants.is_empty() {
                            let models = vec![ModelType::Enum(EnumModel {
                                name: to_pascal_case(name),
                                variants,
                                description: schema.schema_data.description.clone(),
                            })];

                            return Ok(models);
                        }
                    }
                    Ok(Vec::new())
                }

                _ => Ok(Vec::new()),
            }
        }
    }
}

fn extract_type_and_format(
    schema: &ReferenceOr<Schema>,
    all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<(String, String)> {
    match schema {
        ReferenceOr::Reference { reference } => {
            let type_name = reference.split('/').next_back().unwrap_or("Unknown");

            if let Some(ReferenceOr::Item(schema)) = all_schemas.get(type_name) {
                if matches!(schema.schema_kind, SchemaKind::OneOf { .. }) {
                    return Ok((to_pascal_case(type_name), "oneOf".to_string()));
                }
            }
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
                    let (inner_type, format) = match items_ref {
                        ReferenceOr::Item(boxed_schema) => extract_type_and_format(
                            &ReferenceOr::Item((**boxed_schema).clone()),
                            all_schemas,
                        )?,
                        ReferenceOr::Reference { reference } => {
                            let type_name = reference.split('/').next_back().unwrap_or("Unknown");

                            if let Some(ReferenceOr::Item(schema)) = all_schemas.get(type_name) {
                                if matches!(schema.schema_kind, SchemaKind::OneOf { .. }) {
                                    (to_pascal_case(type_name), "oneOf".to_string())
                                } else {
                                    extract_type_and_format(
                                        &ReferenceOr::Reference {
                                            reference: reference.clone(),
                                        },
                                        all_schemas,
                                    )?
                                }
                            } else {
                                extract_type_and_format(
                                    &ReferenceOr::Reference {
                                        reference: reference.clone(),
                                    },
                                    all_schemas,
                                )?
                            }
                        }
                    };
                    Ok((format!("Vec<{inner_type}>"), format))
                } else {
                    Ok(("Vec<serde_json::Value>".to_string(), "array".to_string()))
                }
            }
            SchemaKind::Type(Type::Object(_obj)) => {
                Ok(("serde_json::Value".to_string(), "object".to_string()))
            }
            _ => Ok(("serde_json::Value".to_string(), "unknown".to_string())),
        },
    }
}

/// Extracts field information including type, format, and nullable flag from OpenAPI schema
fn extract_field_info(
    field_name: &str,
    schema: &ReferenceOr<Schema>,
    all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<(FieldInfo, Option<ModelType>)> {
    let (mut field_type, format) = extract_type_and_format(schema, all_schemas)?;

    let (is_nullable, en) = match schema {
        ReferenceOr::Reference { .. } => (false, None),

        ReferenceOr::Item(schema) => {
            let is_nullable = schema.schema_data.nullable;

            let maybe_enum = match &schema.schema_kind {
                SchemaKind::Type(Type::String(s)) if !s.enumeration.is_empty() => {
                    let variants: Vec<String> =
                        s.enumeration.iter().filter_map(|v| v.clone()).collect();
                    field_type = to_pascal_case(field_name);
                    Some(ModelType::Enum(EnumModel {
                        name: to_pascal_case(field_name),
                        variants,
                        description: schema.schema_data.description.clone(),
                    }))
                }
                SchemaKind::Type(Type::Object(_)) => {
                    field_type = "serde_json::Value".to_string();
                    None
                }
                _ => None,
            };
            (is_nullable, maybe_enum)
        }
    };

    Ok((
        FieldInfo {
            field_type,
            format,
            is_nullable,
        },
        en,
    ))
}

fn resolve_all_of_fields(
    _name: &str,
    all_of: &[ReferenceOr<Schema>],
    all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<(Vec<Field>, Vec<ModelType>)> {
    let mut all_fields = Vec::new();
    let mut models = Vec::new();

    for schema_ref in all_of {
        match schema_ref {
            ReferenceOr::Reference { reference } => {
                if let Some(schema_name) = reference.strip_prefix("#/components/schemas/") {
                    if let Some(referenced_schema) = all_schemas.get(schema_name) {
                        let (fields, inline_models) =
                            extract_fields_from_schema(referenced_schema, all_schemas)?;
                        all_fields.extend(fields);
                        models.extend(inline_models);
                    }
                }
            }
            ReferenceOr::Item(_schema) => {
                let (fields, inline_models) = extract_fields_from_schema(schema_ref, all_schemas)?;
                all_fields.extend(fields);
                models.extend(inline_models);
            }
        }
    }
    Ok((all_fields, models))
}

fn resolve_union_variants(
    name: &str,
    schemas: &[ReferenceOr<Schema>],
    all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<(Vec<UnionVariant>, Vec<ModelType>)> {
    use std::collections::BTreeSet;

    let mut variants = Vec::new();
    let mut models = Vec::new();
    let mut enum_values: BTreeSet<String> = BTreeSet::new();
    let mut is_all_simple_enum = true;

    for schema_ref in schemas {
        let resolved = match schema_ref {
            ReferenceOr::Reference { reference } => reference
                .strip_prefix("#/components/schemas/")
                .and_then(|n| all_schemas.get(n)),
            ReferenceOr::Item(_) => Some(schema_ref),
        };

        let Some(resolved_schema) = resolved else {
            is_all_simple_enum = false;
            continue;
        };

        match resolved_schema {
            ReferenceOr::Item(schema) => match &schema.schema_kind {
                SchemaKind::Type(Type::String(s)) if !s.enumeration.is_empty() => {
                    enum_values.extend(s.enumeration.iter().filter_map(|v| v.as_ref().cloned()));
                }
                SchemaKind::Type(Type::Integer(n)) if !n.enumeration.is_empty() => {
                    enum_values.extend(
                        n.enumeration
                            .iter()
                            .filter_map(|v| v.map(|num| format!("Value{num}"))),
                    );
                }

                _ => is_all_simple_enum = false,
            },
            ReferenceOr::Reference { reference } => {
                if let Some(n) = reference.strip_prefix("#/components/schemas/") {
                    if let Some(ReferenceOr::Item(inner)) = all_schemas.get(n) {
                        if let SchemaKind::Type(Type::String(s)) = &inner.schema_kind {
                            let values: Vec<String> = s
                                .enumeration
                                .iter()
                                .filter_map(|v| v.as_ref().cloned())
                                .collect();
                            enum_values.extend(values);
                        } else {
                            is_all_simple_enum = false;
                        }
                    }
                }
            }
        }
    }
    if is_all_simple_enum && !enum_values.is_empty() {
        let enum_name = to_pascal_case(name);
        let enum_model = ModelType::Enum(EnumModel {
            name: enum_name.clone(),
            variants: enum_values.iter().map(|v| to_pascal_case(v)).collect(),
            description: None,
        });

        return Ok((vec![], vec![enum_model]));
    }

    // fallback for usual union-schemas
    for (index, schema_ref) in schemas.iter().enumerate() {
        match schema_ref {
            ReferenceOr::Reference { reference } => {
                if let Some(schema_name) = reference.strip_prefix("#/components/schemas/") {
                    if let Some(referenced_schema) = all_schemas.get(schema_name) {
                        if let ReferenceOr::Item(schema) = referenced_schema {
                            if matches!(schema.schema_kind, SchemaKind::OneOf { .. }) {
                                variants.push(UnionVariant {
                                    name: to_pascal_case(schema_name),
                                    fields: vec![],
                                });
                            } else {
                                let (fields, inline_models) =
                                    extract_fields_from_schema(referenced_schema, all_schemas)?;
                                variants.push(UnionVariant {
                                    name: to_pascal_case(schema_name),
                                    fields,
                                });
                                models.extend(inline_models);
                            }
                        }
                    }
                }
            }
            ReferenceOr::Item(_) => {
                let (fields, inline_models) = extract_fields_from_schema(schema_ref, all_schemas)?;
                let variant_name = format!("Variant{index}");
                variants.push(UnionVariant {
                    name: variant_name,
                    fields,
                });
                models.extend(inline_models);
            }
        }
    }

    Ok((variants, models))
}

fn extract_fields_from_schema(
    schema_ref: &ReferenceOr<Schema>,
    _all_schemas: &IndexMap<String, ReferenceOr<Schema>>,
) -> Result<(Vec<Field>, Vec<ModelType>)> {
    let mut fields = Vec::new();
    let mut inline_models = Vec::new();

    match schema_ref {
        ReferenceOr::Reference { .. } => Ok((fields, inline_models)),
        ReferenceOr::Item(schema) => {
            match &schema.schema_kind {
                SchemaKind::Type(Type::Object(obj)) => {
                    for (field_name, field_schema) in &obj.properties {
                        let (field_info, inline_model) = match field_schema {
                            ReferenceOr::Item(boxed_schema) => extract_field_info(
                                field_name,
                                &ReferenceOr::Item((**boxed_schema).clone()),
                                _all_schemas,
                            )?,
                            ReferenceOr::Reference { reference } => extract_field_info(
                                field_name,
                                &ReferenceOr::Reference {
                                    reference: reference.clone(),
                                },
                                _all_schemas,
                            )?,
                        };

                        let is_nullable = field_info.is_nullable
                            || field_name == "value"
                            || field_name == "default_value";

                        let field_type = field_info.field_type.clone();

                        let is_required = obj.required.contains(field_name);
                        fields.push(Field {
                            name: field_name.clone(),
                            field_type,
                            format: field_info.format,
                            is_required,
                            is_nullable,
                        });
                        if let Some(inline_model) = inline_model {
                            match &inline_model {
                                ModelType::Struct(m) if m.fields.is_empty() => {}
                                _ => inline_models.push(inline_model),
                            }
                        }
                    }
                }
                SchemaKind::Type(Type::String(s)) if !s.enumeration.is_empty() => {
                    let name = schema
                        .schema_data
                        .title
                        .clone()
                        .unwrap_or_else(|| "AnonymousStringEnum".to_string());

                    let enum_model = ModelType::Enum(EnumModel {
                        name,
                        variants: s
                            .enumeration
                            .iter()
                            .filter_map(|v| v.as_ref().map(|s| to_pascal_case(s)))
                            .collect(),
                        description: schema.schema_data.description.clone(),
                    });

                    inline_models.push(enum_model);
                }
                SchemaKind::Type(Type::Integer(n)) if !n.enumeration.is_empty() => {
                    let name = schema
                        .schema_data
                        .title
                        .clone()
                        .unwrap_or_else(|| "AnonymousIntEnum".to_string());

                    let enum_model = ModelType::Enum(EnumModel {
                        name,
                        variants: n
                            .enumeration
                            .iter()
                            .filter_map(|v| v.map(|num| format!("Value{num}")))
                            .collect(),
                        description: schema.schema_data.description.clone(),
                    });

                    inline_models.push(enum_model);
                }

                _ => {}
            }

            Ok((fields, inline_models))
        }
    }
}
