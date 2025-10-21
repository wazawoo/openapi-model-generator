use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Struct(Model),
    Union(UnionModel),             // oneOf/anyOf -> enum
    Composition(CompositionModel), // allOf
    Enum(EnumModel),               // enum values -> enum
}

impl ModelType {
    pub fn name(&self) -> &str {
        match self {
            ModelType::Struct(m) => &m.name,
            ModelType::Enum(e) => &e.name,
            ModelType::Union(u) => &u.name,
            ModelType::Composition(c) => &c.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub format: String,
    pub is_required: bool,
    pub is_nullable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionModel {
    pub name: String,
    pub variants: Vec<UnionVariant>,
    pub union_type: UnionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnionType {
    OneOf,
    AnyOf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionVariant {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionModel {
    pub name: String,
    pub all_fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestModel {
    pub name: String,
    pub content_type: String,
    pub schema: String,
    pub is_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseModel {
    pub name: String,
    pub status_code: String,
    pub content_type: String,
    pub schema: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumModel {
    pub name: String,
    pub variants: Vec<String>,
    pub description: Option<String>,
}
