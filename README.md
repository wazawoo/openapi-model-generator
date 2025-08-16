# OpenAPI Model Generator

[![CI](https://github.com/denislituev/openapi-model-generator/workflows/CI/badge.svg)](https://github.com/denislituev/openapi-model-generator/actions/workflows/ci.yml)
[![PR Check](https://github.com/denislituev/openapi-model-generator/workflows/PR%20Check/badge.svg)](https://github.com/denislituev/openapi-model-generator/actions/workflows/pr-check.yml)

A tool for generating Rust models from OpenAPI specifications. This utility automatically creates Rust structures based on schemas from OpenAPI (Swagger) documentation.

## Features

- OpenAPI 3.0 specification support
- YAML and JSON format support
- Automatic generation of Rust structures with Serde attributes
- Support for nested types and arrays
- Support for required and optional fields
- Support for various data types (String, Number, Integer, Boolean, DateTime, UUID)

## Installation

```bash
cargo install --path .
```

## Usage

```bash
omg -i path/to/openapi.yaml -o ./generated
```

### Parameters

- `-i, --input` - Path to the OpenAPI specification file (YAML or JSON)
- `-o, --output` - Path to the output directory (default: ./generated)

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

### Code Quality

This project uses GitHub Actions for continuous integration with the following checks:

- **Code formatting**: `cargo fmt --all -- --check`
- **Linting**: `cargo clippy --all-targets --all-features -- -D warnings`
- **Compilation**: `cargo check --all-targets --all-features`
- **Security audit**: `cargo audit`
- **Build verification**: `cargo build --release`

#### Local Development

Before submitting a PR, run these commands locally:

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --all-targets --all-features -- -D warnings

# Check compilation
cargo check --all-targets --all-features

# Run tests
cargo test --all-features
```

#### Clippy Configuration

Linting rules are configured in `clippy.toml`. The CI enforces zero warnings policy.

## License

MIT
