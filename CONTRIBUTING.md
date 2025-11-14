# Contributing to OpenAPI Model Generator

Thank you for your interest in contributing to OpenAPI Model Generator! This document provides guidelines and information for contributors.

## Quick Start

### Prerequisites

- **Git** for version control
- **Rust 1.70+** for building and running the tool
- Basic understanding of OpenAPI 3.0 specification
- Familiarity with Serde serialization (helpful but not required)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/openapi-model-generator
cd openapi-model-generator

# Build the project
cargo build

# Run tests
cargo test

# Try the CLI tool with a test schema
cargo run -- --input test.yaml --output generated
```

### Running Code Coverage

To measure test coverage, ensure you have [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov) installed:

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --lib
```

### Repository Layout

```
openapi-model-generator/
├── README.md                 # Main project documentation
├── CONTRIBUTING.md           # This file
├── CHANGELOG.md              # Version history and changes
├── LICENSE                   # Apache-2.0 License
├── Cargo.toml                # Project configuration
├── clippy.toml               # Linter configuration
├── src/                      # Source code
│   ├── main.rs               # CLI entry point
│   ├── lib.rs                # Library entry point
│   ├── cli.rs                # CLI argument parsing
│   ├── parser.rs             # OpenAPI specification parser
│   ├── models.rs             # Internal model representations
│   ├── generator.rs          # Rust code generator
│   └── error.rs              # Error handling
└── tests/                    # Unit and integration tests
```

## Development Workflow

### 1. Create a Feature Branch or Fork the Repository

```bash
git checkout -b feature/your-feature-name
```

Use descriptive branch names:
- `feature/add-discriminator-support`
- `fix/nullable-reference-handling`
- `docs/update-custom-attrs-examples`
- `test/allof-required-fields`

### 2. Make Your Changes

Follow the code style and patterns described below.

### 3. Validate Your Changes

```bash
# Run all tests
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy

# Test on your own OpenAPI schema
cargo run -- --input your-spec.yaml --output generated
```

### 4. Commit Changes

Follow a structured commit message format:

```text
<type>(<scope>): <description>
```

- `<type>`: change category (see table below)
- `<scope>` (optional): the area touched (e.g., parser, generator, cli)
- `<description>`: concise, imperative summary

Accepted commit types:

| Type       | Meaning                                                     |
|------------|-------------------------------------------------------------|
| feat       | New feature                                                 |
| fix        | Bug fixes                                                   |
| docs       | Documentation updates                                       |
| test       | Adding or modifying tests                                   |
| style      | Formatting changes (rustfmt, whitespace, etc.)              |
| refactor   | Code changes that neither fix bugs nor add features         |
| perf       | Performance improvements                                    |
| chore      | Misc tasks (tooling, dependencies)                          |
| breaking   | Backward incompatible changes                               |

Best practices:

- Keep the title concise (ideally <50 chars)
- Use imperative mood (e.g., "Add support", not "Added support")
- Make commits atomic (one logical change per commit)
- Add details in the body when necessary (what/why, not how)
- For breaking changes, either use `breaking!:` or include a `BREAKING CHANGE:` footer

Examples:

```
feat(parser): Add support for x-rust-type extension
fix(generator): Resolve nullable field handling for references
docs: Update README with x-rust-attrs examples
test(parser): Add tests for allOf required fields merge
refactor(generator): Optimize import generation
```

## Code Style

### Formatting

- Follow Rust standard formatting: `cargo fmt`
- Use 4 spaces for indentation
- Keep lines under 100 characters when reasonable

### Linting

- Run clippy and address all warnings: `cargo clippy`
- Fix all clippy warnings before submitting a PR
- Project uses custom clippy configuration in `clippy.toml`

### Testing

- Add unit tests in the same file as the code (using `#[cfg(test)]` modules)
- Add integration tests with inline OpenAPI specs in test functions
- Ensure all tests pass before submitting a PR
- Write tests for new functionality
- Test edge cases (nullable fields, empty arrays, missing properties, etc.)
- Aim for high code coverage

### Test Examples

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_schema() {
        let spec = r#"
            openapi: "3.0.0"
            info:
              title: "Test"
              version: "1.0.0"
            paths: {}
            components:
              schemas:
                User:
                  type: object
                  properties:
                    id:
                      type: string
        "#;
        
        let result = parse_openapi(spec);
        assert!(result.is_ok());
    }
}
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes
4. Run tests: `cargo test`
5. Format code: `cargo fmt`
6. Run clippy: `cargo clippy`
7. Update `CHANGELOG.md` if adding a feature or fixing a bug
8. Commit with descriptive messages (see commit message guidelines above)
9. Push to your fork
10. Open a Pull Request with a clear description

## Adding New Features

### Parser Features

When adding new OpenAPI features to the parser (`src/parser.rs`):

1. Check the [OpenAPI 3.0 Specification](https://spec.openapis.org/oas/v3.0.3)
2. Add the parsing logic to handle the new feature
3. Update the `ModelType` enum in `src/models.rs` if needed
4. Add unit tests in `src/parser.rs` with inline test schemas
5. Update the `README.md` with the new feature

### Generator Features

When adding new code generation features (`src/generator.rs`):

1. Update the generation logic
2. Ensure proper Rust syntax and idioms
3. Add proper Serde attributes when needed
4. Test that generated code compiles
5. Add tests and examples
6. Update documentation

### Custom Extensions

When adding support for new `x-*` vendor extensions:

1. Document the extension format in `README.md`
2. Add parsing in `src/parser.rs`
3. Add generation in `src/generator.rs`
4. Add comprehensive tests with inline test schemas
5. Update `CHANGELOG.md`

## Common Tasks

### Testing Changes

```bash
# Run unit tests
cargo test --lib

# Run specific test
cargo test test_nullable_reference_field

# Run with output
cargo test -- --nocapture

# Test with your own OpenAPI spec
cargo run -- --input your-spec.yaml --output output_dir
```

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run -- --input your-spec.yaml --output generated

# Build with debug symbols
cargo build

# Use rust-gdb or rust-lldb for debugging
rust-gdb target/debug/omg
```

## OpenAPI Specification Resources

- [OpenAPI 3.0.3 Specification](https://spec.openapis.org/oas/v3.0.3)
- [OpenAPI Examples](https://github.com/OAI/OpenAPI-Specification/tree/main/examples)
- [Swagger Editor](https://editor.swagger.io/) - for validating schemas
- [openapiv3 crate docs](https://docs.rs/openapiv3/) - the library we use for parsing

## Questions?

Open an issue or discussion on GitHub.

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 License.
