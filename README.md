# Repack

**Schema-first code generation for multiple target languages**

Repack is a robust code generation tool that lets you define your data models once in a simple schema language, then generate consistent, type-safe code across multiple programming languages and formats.

## Overview

Write your data structures once, generate everywhere:

```repack
enum UserType #model {
    Admin
    User
    Guest
}

struct User @users #model {
    id uuid
    name string
    email string?
    user_type UserType
    tags string[]
}

output rust @./generated/rust #model;
output typescript @./generated/ts #model;
output postgres @./generated/sql #model;
```

This generates Rust structs, TypeScript interfaces, and PostgreSQL schemas from a single definition.

## Installation

Build from source:

```bash
git clone https://github.com/jacksonzamorano/repack
cd repack
cargo build --release
```

## Quick Start

1. **Create a schema file** (`example.repack`):

```repack
enum Status #model {
    Active
    Inactive
}

struct User @users #model {
    id uuid
    created_date datetime
    name string
    email string
    status Status
}

output rust @./generated #model;
```

2. **Generate code**:

```bash
repack build example.repack
```

3. **View generated Rust code** (`generated/model.rs`):

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    Active,
    Inactive,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub created_date: DateTime<Utc>,
    pub name: String,
    pub email: String,
    pub status: Status,
}
```

## Core Concepts

### Data Types

Repack supports 8 built-in types:

- `string` - UTF-8 text
- `int32` - 32-bit signed integer  
- `int64` - 64-bit signed integer
- `float64` - 64-bit floating point
- `boolean` - True/false value
- `datetime` - Timestamp with timezone
- `uuid` - UUID v4 identifier
- `bytes` - Byte array

### Field Modifiers

- `field_name type` - Required field
- `field_name type?` - Optional/nullable field  
- `field_name type[]` - Array of values

### Object Types

**Struct**: Basic data structures
```repack
struct ApiResponse #api {
    users User[]
    total_count int32
}
```

**Enum**: Fixed set of values
```repack
enum Priority #model {
    Low
    Medium  
    High
}
```

**Snippet**: Reusable field collections
```repack
snippet timestamps {
    created_at datetime
    updated_at datetime?
}

struct Post #model {
    !timestamps  // Include snippet
    title string
    content string
}
```

### Categories and Filtering

Use categories (`#tag`) to control what gets generated:

```repack
output rust @./backend #model #internal;
output typescript @./frontend #model #api;

enum Status #model #api {    // Generated for both
    Active
    Inactive  
}

struct InternalData #internal {  // Only in Rust output
    secret_key string
}

struct ApiResponse #api {        // Only in TypeScript output
    data Status[]
}
```

### Table Names and References

```repack
struct User @users #model {      // @users = table name
    id uuid
    email string
    role_id uuid ref(Role.id)    // Foreign key reference
}

struct Role @roles #model {
    id uuid
    name string
}
```

## Commands

### Build Commands

```bash
# Generate all code files
repack build schema.repack

# Clean generated files  
repack clean schema.repack

# Generate documentation
repack document schema.repack

# Generate configuration files
repack configure environment schema.repack
```

### Output Configuration

```repack
# Basic output
output rust @./src/models #model;

# Multiple outputs
output rust @./backend #model;
output typescript @./frontend #api;
output postgres @./database #model;
output markdown @./docs #model;
```

## Built-in Blueprints

Repack includes blueprints for:

- **rust** - Rust structs with derive traits
- **typescript** - TypeScript interfaces  
- **postgres** - PostgreSQL DDL with tables/indexes
- **go** - Go structs with JSON tags
- **markdown** - Documentation generation

Each blueprint maps repack types to language-specific equivalents:

| Repack Type | Rust | TypeScript | PostgreSQL |
|-------------|------|------------|------------|
| `string` | `String` | `string` | `TEXT` |
| `int32` | `i32` | `number` | `INTEGER` |
| `int64` | `i64` | `number` | `BIGINT` |
| `boolean` | `bool` | `boolean` | `BOOLEAN` |
| `datetime` | `DateTime<Utc>` | `Date` | `TIMESTAMPTZ` |
| `uuid` | `Uuid` | `string` | `UUID` |

## Real-World Example

```repack
snippet audit_fields {
    created_at datetime
    updated_at datetime?
    created_by uuid
}

enum OrderStatus #model #api {
    Pending
    Confirmed
    Shipped
    Delivered
    Cancelled
}

struct Customer @customers #model {
    !audit_fields
    id uuid
    email string
    first_name string
    last_name string
}

struct Order @orders #model #api {
    !audit_fields
    id uuid
    customer_id uuid ref(Customer.id)
    status OrderStatus
    total_amount float64
    items OrderItem[]
}

struct OrderItem #model {
    product_name string
    quantity int32
    unit_price float64
}

struct OrderSummary #api {
    order_id uuid
    customer_name string
    status OrderStatus
    total_amount float64
    item_count int32
}

output rust @./src/models #model;
output typescript @./frontend/types #api;
output postgres @./database #model;
```

This generates:
- Complete Rust structs with proper types
- TypeScript interfaces for frontend APIs
- PostgreSQL tables with foreign keys
- Consistent field names and types across all outputs

## Advanced Features

### Functions and Methods

```repack
struct User @users #model {
    id uuid
    name string
    
    database: find_by_email(email: string) {
        query "SELECT * FROM users WHERE email = $1"
    }
}
```

### Inheritance (Planned)

```repack
struct BaseEntity {
    id uuid
    created_at datetime
}

struct User: BaseEntity @users #model {
    name string
    email string
}
```

## Project Structure

```
src/
├── main.rs              # CLI entry point and command handling
├── syntax/              # Schema parsing and type system
│   ├── parser.rs        # Tokenization and file parsing
│   ├── types.rs         # Core and custom type definitions  
│   ├── repack_struct.rs # Object/struct parsing
│   └── repack_enum.rs   # Enum parsing
└── blueprint/           # Code generation system
    ├── renderer.rs      # Template rendering engine
    ├── store.rs         # Blueprint loading and management
    └── core/            # Built-in language blueprints
        ├── rust.blueprint
        ├── typescript.blueprint
        └── postgres.blueprint
```

## Contributing

Repack is open source (GPL-3.0). Contributions welcome:

1. **Bug Reports**: File issues with reproduction steps
2. **Feature Requests**: Propose new language blueprints or syntax features
3. **Blueprint Development**: Create blueprints for new target languages

## License

Licensed under GPL-3.0. See [LICENSE.txt](LICENSE.txt) for details.

---

**Eliminate boilerplate. Define once, generate everywhere.**