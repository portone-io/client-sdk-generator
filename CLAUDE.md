# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**browser-sdk-generator** is a tool that automatically generates PortOne SDK code for various environments based on YAML schema files. The project is a hybrid Rust/JavaScript monorepo that generates type-safe client SDKs for the PortOne payment service.

## Common Development Commands

### Building the Project

```bash
# Build Rust project (release mode)
cargo build --release

# Build Rust project (development mode)
cargo build

# Run the SDK generator CLI
cargo client-sdk-generator    # release mode
cargo client-sdk-generator-dev # development mode
```

### Testing

```bash
# Run all tests in the workspace
cargo test --workspace

# Run tests for specific packages
cargo test -p client_sdk_schema
cargo test -p client_sdk_ts_codegen
cargo test -p client_sdk_dart_codegen

# Run a specific test by name
cargo test test_function_name
```

### Code Quality

```bash
# Rust formatting
cargo format
# or
cargo fmt --all --verbose

# Rust linting
cargo lint
# or
cargo clippy --workspace --all-targets -- --deny warnings

# JavaScript/TypeScript formatting and linting (using Biome)
pnpm ci
# or
biome ci
```

### Schema and SDK Generation

```bash
# Regenerate schema.json
cargo run -p client_sdk_schema --bin generate_schema

# Generate TypeScript SDK
pnpm portone-client-sdk-generator generate --schema ./client-sdk.yml --generator typescript ./output

# Generate Dart SDK
pnpm portone-client-sdk-generator generate --schema ./client-sdk.yml --generator dart ./output
```

## Architecture Overview

### Project Structure

The project consists of several interconnected Rust crates and npm packages:

1. **Core Crates** (in `crates/`):
   - `client_sdk_generator`: Main CLI tool that orchestrates SDK generation
   - `client_sdk_schema`: Schema definitions and YAML parsing logic
   - `client_sdk_ts_codegen`: TypeScript code generator using Biome AST
   - `client_sdk_dart_codegen`: Dart code generator
   - `client_sdk_utils`: Shared utilities

2. **npm Packages** (in `packages/@portone/`):
   - Platform-specific binary packages for distribution
   - Main npm package that wraps the Rust binary

### Key Architectural Patterns

1. **Schema-Driven Generation**: The entire SDK generation process is driven by YAML schema files that define:
   - API methods and their parameters
   - Resource hierarchies with nested subresources
   - PG-specific features through `pg_specific` fields

2. **Resource Index Transformation**: The generator transforms nested resource structures into flat indices for efficient code generation:
   - Resources can have subresources and parameters
   - Each resource is uniquely identified by its path
   - Methods are associated with resources via ID references

3. **AST-Based Code Generation**: 
   - TypeScript generation uses Biome's AST for proper code formatting
   - Ensures generated code follows consistent style rules
   - Supports complex type generation including unions, intersections, and generics

4. **Platform-Specific Binary Distribution**: 
   - Rust binary is compiled for multiple platforms
   - Distributed via npm with platform-specific packages
   - Automatic platform detection during installation

### Important Implementation Details

- **YAML Parsing**: Uses `serde_yaml_ng` for robust YAML schema parsing
- **Type Safety**: Leverages Rust's type system to ensure generated SDKs are type-safe
- **Error Handling**: Comprehensive error types for schema validation and code generation
- **Extensibility**: New language generators can be added by implementing the generator interface

## Development Tips

1. When modifying schema structures, always regenerate `schema.json` using the generate_schema binary
2. Test generated code by running the generator with sample schemas
3. Use Biome for consistent JavaScript/TypeScript formatting in the generated code
4. Cargo workspace commands affect all crates - use `-p` flag to target specific packages
5. The project uses pnpm workspaces for JavaScript package management