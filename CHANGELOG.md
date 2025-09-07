# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2024-12-20

### Added
- **Nullable Field Support**: Fields with `nullable: true` now generate `Option<T>` type correctly
- **Smart Option Generation**: Fields become `Option<T>` when either not required OR nullable
- **Enhanced Type Safety**: Better representation of optional vs nullable fields in generated Rust code

## [0.2.0] - 2024-08-26

### Added
- **UUID Format Support**: Fields with `type: string, format: uuid` now generate `Uuid` type instead of `String`
- **DateTime Format Support**: Fields with `type: string, format: date-time` now generate `DateTime<Utc>` type  
- **Schema Composition Support**: Full support for OpenAPI composition patterns:
  - `allOf`: Combines multiple schemas into a single struct with merged fields
  - `oneOf`: Generates tagged union enum for exclusive choice between schemas
  - `anyOf`: Generates tagged union enum for inclusive choice between schemas
- **Improved Code Generation**:
  - Smart naming for union variants (uses schema names for `$ref`, auto-generates for inline schemas)
  - PascalCase naming for request/response models (follows Rust conventions)
  - Proper `$ref` reference resolution in complex schema hierarchies
- **Library API**: Export of main functions (`parse_openapi`, `generate_models`) for use as a library

## [0.1.0] - Initial Release

### Added
- Basic OpenAPI 3.0 specification parsing
- Simple struct generation for OpenAPI schemas
- Support for basic Rust types (String, i64, bool, etc.)
- Serde integration with proper field renaming
- Command-line interface with input/output options
- Support for required and optional fields
- Array type generation with `Vec<T>`
- Basic error handling and validation
