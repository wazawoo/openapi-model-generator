# OpenAPI Model Generator

A Rust library and CLI tool for generating Rust models from OpenAPI specifications. This utility automatically creates Rust structures based on schemas from OpenAPI (Swagger) documentation.

## Features

- **OpenAPI 3.0 specification support** with full schema parsing
- **YAML and JSON format support** for input specifications
- **Automatic generation of Rust structures** with Serde attributes
- **Schema Composition Support**: Complete implementation of OpenAPI composition patterns:
  - `allOf` - Combines multiple schemas into a single struct
  - `oneOf` / `anyOf` - Generates tagged union enums with proper serde configuration
- **Advanced Type Support**: 
  - Enum Support - Automatically generates Rust enums from OpenAPI string schemas with enumeration constraints.
  - UUID fields (`format: uuid` → `Uuid` type)
  - DateTime fields (`format: date-time` → `DateTime<Utc>` type)
  - Nested types and arrays with proper generic handling
- **Smart Code Generation**:
  - Required vs optional field detection (`Option<T>` for nullable fields)
  - PascalCase naming for generated request/response models
  - Reference resolution across schema definitions
- **Clean Code Output**: Properly formatted Rust code with comprehensive serde annotations

## Installation

### As a CLI tool

```bash
cargo install openapi-model-generator
```

### As a library dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
openapi-model-generator = "0.3.1"
```

## Usage

### Command Line Interface

```bash
omg -i path/to/openapi.yaml -o ./generated
```

### Parameters

- `-i, --input` - Path to the OpenAPI specification file (YAML or JSON)
- `-o, --output` - Path to the output directory (default: ./generated)

### Library Usage

```rust
use openapi_model_generator::{parse_openapi, generate_models};
use std::fs;

// Parse OpenAPI specification
let openapi_spec = fs::read_to_string("openapi.yaml")?;
let openapi: openapiv3::OpenAPI = serde_yaml::from_str(&openapi_spec)?;

// Generate models
let (models, requests, responses) = parse_openapi(&openapi)?;
let generated_code = generate_models(&models, &requests, &responses)?;

// Write to file
fs::write("models.rs", generated_code)?;
```

## Example

Source OpenAPI schema:
```yaml
components:
  schemas:
    User:
      type: object
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
        email:
          type: string
        age:
          type: integer
        is_active:
          type: boolean
      required:
        - id
        - name
        - email
```

Generated Rust code:
```rust
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "id")]
    pub id: Uuid,
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "email")]
    pub email: String,
    #[serde(rename = "age")]
    pub age: Option<i64>,
    #[serde(rename = "is_active")]
    pub is_active: Option<bool>,
}
```

## Development

### Dependencies

- Rust 1.70 or higher
- Cargo

### Building

```bash
cargo build
```

### Running Tests

```bash
cargo test
```

## Examples

The generator supports complex OpenAPI patterns including schema composition:

- **UUID and DateTime handling** - Automatic type conversion for formatted strings
- **Schema composition with allOf** - Inheritance and field merging 
- **Tagged unions with oneOf/anyOf** - Automatic enum generation with proper serde tags

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for detailed information about releases and changes.

## License

MIT
