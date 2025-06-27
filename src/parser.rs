use crate::{models::{Model, Field, RequestModel, ResponseModel}, Result};
use openapiv3::{OpenAPI, Schema, ReferenceOr, SchemaKind, Type, VariantOrUnknownOrEmpty, StringFormat};

pub fn parse_openapi(openapi: &OpenAPI) -> Result<(Vec<Model>, Vec<RequestModel>, Vec<ResponseModel>)> {
    let mut models = Vec::new();
    let mut requests = Vec::new();
    let mut responses = Vec::new();

    // Parse components/schemas
    if let Some(components) = &openapi.components {
        for (name, schema) in &components.schemas {
            if let Some(model) = parse_schema(name, schema)? {
                models.push(model);
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
    if let Some(request_body_ref) = &operation.request_body {
        if let ReferenceOr::Item(request_body) = request_body_ref {
            for (content_type, media_type) in &request_body.content {
                if let Some(schema) = &media_type.schema {
                    let request = RequestModel {
                        name: format!("{}Request", operation.operation_id.as_deref().unwrap_or("Unknown")),
                        content_type: content_type.clone(),
                        schema: extract_type(schema)?,
                        is_required: request_body.required,
                    };
                    requests.push(request);
                }
            }
        }
    }

    // Parse responses
    for (status, response_ref) in operation.responses.responses.iter() {
        if let ReferenceOr::Item(response) = response_ref {
            for (content_type, media_type) in &response.content {
                if let Some(schema) = &media_type.schema {
                    let response = ResponseModel {
                        name: format!("{}Response", operation.operation_id.as_deref().unwrap_or("Unknown")),
                        status_code: status.to_string(),
                        content_type: content_type.clone(),
                        schema: extract_type(schema)?,
                        description: Some(response.description.clone()),
                    };
                    responses.push(response);
                }
            }
        }
    }

    Ok(())
}

fn parse_schema(name: &str, schema: &ReferenceOr<Schema>) -> Result<Option<Model>> {
    match schema {
        ReferenceOr::Reference { .. } => Ok(None),
        ReferenceOr::Item(schema) => {
            if let SchemaKind::Type(Type::Object(obj)) = &schema.schema_kind {
                let mut fields = Vec::new();
                for (field_name, field_schema) in &obj.properties {
                    let field_type = match field_schema {
                        ReferenceOr::Item(boxed_schema) => {
                            extract_type(&ReferenceOr::Item((**boxed_schema).clone()))?
                        },
                        ReferenceOr::Reference { reference } => {
                            extract_type(&ReferenceOr::Reference { reference: reference.clone() })?
                        }
                    };
                    let is_required = obj.required.contains(field_name);
                    fields.push(Field {
                        name: field_name.clone(),
                        field_type,
                        is_required,
                    });
                }
                Ok(Some(Model {
                    name: name.to_string(),
                    fields,
                }))
            } else {
                Ok(None)
            }
        }
    }
}

fn extract_type(schema: &ReferenceOr<Schema>) -> Result<String> {
    match schema {
        ReferenceOr::Reference { reference } => {
            let type_name = reference.split('/').last().unwrap_or("Unknown");
            Ok(type_name.to_string())
        }
        ReferenceOr::Item(schema) => {
            match &schema.schema_kind {
                SchemaKind::Type(Type::String(string_type)) => {
                    match &string_type.format {
                        VariantOrUnknownOrEmpty::Item(fmt) => match fmt {
                            StringFormat::DateTime => Ok("DateTime<Utc>".to_string()),
                            StringFormat::Date => Ok("NaiveDate".to_string()),
                            _ => Ok("String".to_string()),
                        },
                        _ => Ok("String".to_string()),
                    }
                }
                SchemaKind::Type(Type::Integer(_)) => Ok("i64".to_string()),
                SchemaKind::Type(Type::Number(_)) => Ok("f64".to_string()),
                SchemaKind::Type(Type::Boolean {}) => Ok("bool".to_string()),
                SchemaKind::Type(Type::Array(arr)) => {
                    if let Some(items) = &arr.items {
                        let items_ref: &ReferenceOr<Box<Schema>> = &*items;
                        let inner_type = match items_ref {
                            ReferenceOr::Item(boxed_schema) => extract_type(&ReferenceOr::Item((**boxed_schema).clone()))?,
                            ReferenceOr::Reference { reference } => extract_type(&ReferenceOr::Reference { reference: reference.clone() })?,
                        };
                        Ok(format!("Vec<{}>", inner_type))
                    } else {
                        Ok("Vec<serde_json::Value>".to_string())
                    }
                }
                SchemaKind::Type(Type::Object(obj)) => {
                    if obj.properties.is_empty() {
                        Ok("()".to_string())
                    } else {
                        Ok("serde_json::Value".to_string())
                    }
                }
                _ => Ok("serde_json::Value".to_string()),
            }
        }
    }
} 
