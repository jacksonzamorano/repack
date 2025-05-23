# Repack

Repack is a code generation tool that allows you to define data models in a simple, declarative language and automatically generate code for multiple targets including TypeScript, Rust, PostgreSQL, and more.

## Overview

Repack uses `.repack` files to define data models with records (database tables) and structs (data transfer objects). From a single schema definition, you can generate:

- **PostgreSQL** database schemas with proper constraints and relationships
- **TypeScript** classes and interfaces
- **Rust** structs and data types
- **Documentation** and other outputs
- More coming soon!

## Getting Started

### Installation

Build the project using Cargo:

```bash
cargo build --release
```

### Basic Usage

Create a `.repack` file with your data model definitions. For samples, check out `test` directory.

## Repack Language Reference

### Basics

There are three top-level entities in Repack: Records, Structs, and Outputs.