use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Struct(Model),
    Union(UnionModel),             // oneOf/anyOf -> enum
    Composition(CompositionModel), // allOf
    Enum(EnumModel),               // enum values -> enum
    TypeAlias(TypeAliasModel),     // x-rust-type -> type alias
}

impl ModelType {
    pub fn name(&self) -> &str {
        match self {
            ModelType::Struct(m) => &m.name,
            ModelType::Enum(e) => &e.name,
            ModelType::Union(u) => &u.name,
            ModelType::Composition(c) => &c.name,
            ModelType::TypeAlias(t) => &t.name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub fields: Vec<Field>,
    pub custom_attrs: Option<Vec<String>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub format: String,
    pub is_required: bool,
    pub is_nullable: bool,
    pub is_array_ref: bool,
    pub description: Option<String>,
}

impl Field {
    /// Returns true if this field should be flattened (for additionalProperties)
    pub fn should_flatten(&self) -> bool {
        self.name == "additional_properties"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnionModel {
    pub name: String,
    pub variants: Vec<UnionVariant>,
    pub union_type: UnionType,
    pub custom_attrs: Option<Vec<String>>,
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
    pub primitive_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionModel {
    pub name: String,
    pub all_fields: Vec<Field>,
    pub custom_attrs: Option<Vec<String>>,
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
pub struct RouteModel {
    /// `/blah/wow/{ohMy}/{lookAtMe}`
    pub path: String,
    pub backup_name: String,
    /// `GET`, etc
    pub method: String,
    /// path with params removed: `/blah/wow/{}/{}`
    pub format_path: String,
    /// query param `qParam` to rust name `q_param`
    pub query_params: IndexMap<String, String>,
    /// path param `pParam` to rust name `p_param`
    pub path_params: IndexMap<String, String>,
    /// header name `ADDL-HEADER` to rust name `addl_header`
    pub additional_headers: IndexMap<String, String>,
    pub response_schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumModel {
    pub name: String,
    pub variants: Vec<String>,
    pub description: Option<String>,
    pub custom_attrs: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeAliasModel {
    pub name: String,
    pub target_type: String,
    pub description: Option<String>,
    pub custom_attrs: Option<Vec<String>>,
}
