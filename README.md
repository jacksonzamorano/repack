# Repack

**Powerful model code generation for multiple target languages**

Repack is a schema-first code generation tool that allows you to define your data models once and generate consistent, type-safe code across multiple programming languages and platforms. Define your objects, enums, and relationships in a simple schema language, then generate database schemas, API models, and data structures for Rust, TypeScript, Go, PostgreSQL, and more.

## Project Overview

Repack solves the problem of maintaining consistent data models across different parts of your application stack. Instead of manually writing and synchronizing model definitions across your database, backend API, frontend client, and other services, you define your schema once in a `.repack` file and automatically generate all the necessary code.

**Key Benefits:**
- **Single Source of Truth**: Define your data models once, use everywhere
- **Type Safety**: Generated code is fully typed for each target language
- **Relationship Management**: Automatic handling of foreign keys, joins, and references
- **Extensible**: Custom blueprints for any target language or framework
- **Database Integration**: Generate SQL schemas with indexes, constraints, and relationships

**What Repack Generates:**
- Database schemas (PostgreSQL, MySQL, etc.)
- API models (Rust structs, TypeScript interfaces, Go structs)
- Serialization/deserialization code
- Database query helpers
- Documentation (Markdown, etc.)

## Quick Start

### 1. Create Your First Schema

Create a file called `example.repack`:

```repack
// Define output targets
output postgres @database;
output rust @models;
output typescript @frontend/types;

// Define an enum
enum UserType #model {
    Admin
    User
    Guest
}

// Define a record (database table)
record User @users #model {
    id uuid db:pk
    created_date datetime db:default("NOW()")
    name string
    email string db:unique
    user_type UserType
    is_active boolean
}

// Define a struct (in-memory only)
struct UserList #model {
    users User[]
    total_count int32
}
```

### 2. Generate Code

```bash
# Build all configured outputs
repack example.repack

# Clean generated files
repack example.repack --clean
```

This will generate:
- `database/model.sql` - PostgreSQL schema
- `models/model.rs` - Rust structs  
- `frontend/types/User.ts`, `frontend/types/UserType.ts`, etc. - TypeScript interfaces

### 3. Create a Custom Blueprint

Create `custom.blueprint` for a simple documentation generator:

```blueprint
[meta id]docs[/meta]
[meta name]Documentation Generator[/meta]

[define string]Text[/define]
[define int32]Integer[/define]
[define boolean]True/False[/define]
[define uuid]Unique ID[/define]

[file]models.md[/file]
# Data Models

[each object]
## [name]
[if record]**Database Table**: `[table_name]`[/if]
[if struct]**In-Memory Structure**[/if]

[each field]
- **[name]**: [type][if optional] (optional)[/if][if array] (array)[/if]
[func db.pk]  - Primary Key[/func]
[func db.unique]  - Must be unique[/func]
[/each]

[/each]
```

Then use it in your schema:

```repack
blueprint "custom.blueprint"
output docs @documentation;

// ... your model definitions
```

## Detailed Reference

### Schema Syntax

#### Basic Structure

```repack
// Comments start with //
// Import external schema files
import "other_schema.repack"

// Load external blueprint
blueprint "path/to/custom.blueprint"

// Configure output targets
output <blueprint_id> @<output_path> #<category> #<category> {
    option_key option_value
}

// Define snippets (reusable field groups)
snippet <name> {
    field_name field_type
}

// Define enums
enum <name> #<category> {
    OptionOne
    OptionTwo
}

// Define objects
<object_type> <name> @<table_name> : <parent> #<category> {
    // Fields and functions
}
```

#### Object Types

Repack supports three types of objects, each with different capabilities:

| Object Type | Purpose | Database Table | Inheritance | Custom Types | Arrays |
|-------------|---------|----------------|-------------|--------------|--------|
| **`record`** | Database entities | ✅ Required (`@table_name`) | ❌ No | ✅ Enums only | ❌ No |
| **`struct`** | In-memory data | ❌ Not allowed | ❌ No | ✅ All types | ✅ Yes |
| **`synthetic`** | Computed views | ✅ Inherited from parent | ✅ Yes | ✅ Enums only | ❌ No |

**Examples:**

```repack
// Record: Maps to database table
record User @users #model {
    id uuid db:pk
    name string
}

// Struct: In-memory data structure
struct UserResponse #api {
    user User
    permissions string[]
}

// Synthetic: Extends a record with computed fields
record ContactInfo @contacts #model {
    user_id ref(User.id)
    email string
}

synthetic FullUser: ContactInfo #view {
    *  // Include all fields from ContactInfo
    name from(user_id.name)  // Add computed field via join
}
```

### Field Types

#### Core Types

| Type | Description | Example |
|------|-------------|---------|
| `string` | UTF-8 text | `name string` |
| `int32` | 32-bit integer | `count int32` |
| `int64` | 64-bit integer | `big_number int64` |
| `float64` | 64-bit float | `price float64` |
| `boolean` | True/false | `is_active boolean` |
| `datetime` | Timestamp | `created_at datetime` |
| `uuid` | UUID v4 | `id uuid` |

#### Field Modifiers

```repack
field_name type           // Required field
field_name type?          // Optional field  
field_name type[]         // Array of values (structs only)
field_name type[]?        // Optional array
```

#### Field References

```repack
// Direct type reference
user_id UserType

// Reference field from another object
user_name ref(User.name)

// Join via foreign key relationship
user_name from(user_id.name)

// Explicit join reference
user_name with(user_join.name)
```

### Functions

Functions provide additional metadata and behavior for fields and objects. They follow the pattern `namespace:function_name(arguments)`.

#### Database Functions (`db:` namespace)

**Field-Level Functions:**

| Function | Description | Example |
|----------|-------------|---------|
| `db:pk` | Primary key | `id uuid db:pk` |
| `db:unique` | Unique constraint | `email string db:unique` |
| `db:default("value")` | Default value | `created_at datetime db:default("NOW()")` |
| `db:generated` | Generated column | `full_name string db:generated` |
| `db:identity` | Auto-increment | `id int32 db:identity` |

**Object-Level Functions:**

| Function | Description | Example |
|----------|-------------|---------|
| `db:index("field")` | Single field index | `db:index("email")` |
| `db:index("f1", "f2")` | Composite index | `db:index("user_id", "created_at")` |

**Example Usage:**

```repack
record User @users #model {
    id uuid db:pk
    email string db:unique
    created_at datetime db:default("NOW()")
    updated_at datetime
    
    // Object-level functions
    db:index("email")
    db:index("created_at", "updated_at")
}
```

### Inheritance and Reuse

#### Snippets (Field Groups)

```repack
snippet timestamps {
    created_at datetime db:default("NOW()")
    updated_at datetime?
}

snippet base {
    id uuid db:pk
    !timestamps  // Include snippet
}

record User @users {
    !base  // Include base snippet
    name string
}
```

#### Object Inheritance (Synthetic Objects)

```repack
record User @users #model {
    id uuid db:pk
    name string
}

synthetic ExtendedUser: User #view {
    *                    // Include all fields from User
    - password          // Exclude specific fields
    + email             // Include only specific fields
    display_name string // Add new fields
}
```

### Categories and Filtering

Use categories to control which objects are generated for each output:

```repack
// Only generate objects tagged with #api
output typescript @frontend #api;

// Generate objects tagged with #model or #view  
output rust @backend #model #view;

enum Status #api #model {
    Active
    Inactive
}

record User @users #model {
    // Will be generated for rust output only
}

struct UserResponse #api {
    // Will be generated for typescript output only
}
```

### Output Configuration

```repack
output <blueprint_id> @<path> #<categories> <exclusions> {
    option_name option_value
}
```

**Example:**

```repack
output postgres @database/schema #model !User !internal_table;
output rust @src/models #model #api {
    derive_debug true
    serde_support true
}
```

### Example Files

See the `test/` directory for complete examples:

- **[`test/test.repack`](test/test.repack)** - Complete schema example with all features
- **[`test/markdown.blueprint`](test/markdown.blueprint)** - Custom blueprint for documentation
- **[`test/rust/model.rs`](test/rust/model.rs)** - Generated Rust code
- **[`test/postgres/model.sql`](test/postgres/model.sql)** - Generated SQL schema
- **[`test/typescript/`](test/typescript/)** - Generated TypeScript interfaces

### Built-in Blueprints

Repack includes built-in blueprints for common targets:

- **`rust`** - Rust structs with serde support
- **`typescript`** - TypeScript interfaces
- **`postgres`** - PostgreSQL DDL schemas
- **`go`** - Go structs with JSON tags

See [`src/blueprint/core/`](src/blueprint/core/) for the blueprint source code.

### Advanced Features

#### Joins and Relationships

```repack
record User @users {
    id uuid db:pk
    name string
}

record Post @posts {
    id uuid db:pk
    user_id ref(User.id)
    title string
    
    // Explicit join definition
    ^ author_posts self.user_id = User.id
}

synthetic PostWithAuthor: Post {
    *
    author_name with(author_posts.name)
}
```

#### Complex Schemas

```repack
import "common/*.repack"  // Import all .repack files from directory
blueprint "blueprints/custom.blueprint"

output postgres @database #core #audit;
output rust @models #core {
    package_name my_models
}

snippet auditable {
    created_by uuid
    created_at datetime db:default("NOW()")
    updated_by uuid?
    updated_at datetime?
}

enum Priority #core {
    Low
    Medium  
    High
    Critical
}

record Project @projects #core {
    !auditable
    name string db:unique
    description string?
    priority Priority
    is_active boolean db:default("true")
    
    db:index("name")
    db:index("priority", "is_active")
}
```

## Blueprint Development

Blueprints are template files that define how to generate code for specific target languages or frameworks. They use a powerful template syntax with variables, loops, conditionals, and type mappings to produce consistent, type-safe output.

### Blueprint File Structure

Every blueprint must follow this basic structure:

```blueprint
[meta id]unique_identifier[/meta]
[meta name]Human Readable Name[/meta]

[define core_type]target_type[/define]
[link dependency]import_statement[/link]

[file]output_filename[/file]
template content here...
```

### Required Meta Tags

Every blueprint **must** start with these meta tags:

| Meta Tag | Description | Required | Example |
|----------|-------------|----------|---------|
| `[meta id]` | Unique identifier used in `output` statements | ✅ Yes | `[meta id]rust[/meta]` |
| `[meta name]` | Human-readable name for documentation | ✅ Yes | `[meta name]Rust Structs[/meta]` |

**Example:**
```blueprint
[meta id]rust[/meta]
[meta name]Rust[/meta]
```

### Type Definitions

Map Repack's core types to target language types using `[define]`:

```blueprint
[define repack_type]target_language_type[/define]
```

**Available Core Types:**

| Repack Type | Description | Example Mappings |
|-------------|-------------|------------------|
| `string` | UTF-8 text | `String` (Rust), `string` (Go/TS), `TEXT` (SQL) |
| `int32` | 32-bit integer | `i32` (Rust), `int32` (Go), `number` (TS), `INTEGER` (SQL) |
| `int64` | 64-bit integer | `i64` (Rust), `int64` (Go), `bigint` (TS), `BIGINT` (SQL) |
| `float64` | 64-bit float | `f64` (Rust), `float64` (Go), `number` (TS), `DOUBLE` (SQL) |
| `boolean` | True/false | `bool` (Rust), `bool` (Go), `boolean` (TS), `BOOLEAN` (SQL) |
| `datetime` | Timestamp | `DateTime<Utc>` (Rust), `time.Time` (Go), `Date` (TS), `TIMESTAMPTZ` (SQL) |
| `uuid` | UUID v4 | `Uuid` (Rust), `uuid.UUID` (Go), `string` (TS), `UUID` (SQL) |

**Example Type Definitions:**
```blueprint
// Rust types
[define string]String[/define]
[define int32]i32[/define]
[define boolean]bool[/define]
[define datetime]DateTime<Utc>[/define]
[define uuid]Uuid[/define]

// TypeScript types  
[define string]string[/define]
[define int32]number[/define]
[define boolean]boolean[/define]
[define datetime]Date[/define]
[define uuid]string[/define]

// PostgreSQL types
[define string]TEXT[/define]
[define int32]INTEGER[/define]
[define boolean]BOOLEAN[/define]
[define datetime]TIMESTAMPTZ[/define]
[define uuid]UUID[/define]
```

### Import Links

Define import statements for types that require external dependencies:

```blueprint
[link dependency_key]import_statement[/link]
```

**Special Variables:**
- `$` - Replaced with custom type name (e.g., `custom::$` becomes `custom::UserType`)

**Examples:**
```blueprint
// Rust imports
[link uuid]use uuid::Uuid;[/link]
[link datetime]use chrono::{DateTime, Utc};[/link]
[link custom]use crate::types::$;[/link]

// Go imports  
[link uuid]import "github.com/google/uuid"[/link]
[link datetime]import "time"[/link]

// TypeScript imports
[link custom]import { $ } from './types';[/link]
```

### File Output

Specify output files and their content. Files inherit the current iteration context:

```blueprint
[file]filename[/file]
content goes here...
```

**Dynamic Filenames:**
Files can use any variable from the current context:
```blueprint
[file]models.rs[/file]           // Single file for all models
[file][name].ts[/file]           // Separate file per object (uses current object name)
[file]schema/[name].sql[/file]   // Files in subdirectories
[file][enum_name]_types.rs[/file] // In enum context, uses enum name
```

**Context Inheritance:**
Files inherit all variables and flags from their surrounding context:
```blueprint
[each object]
[file][name]_model.rs[/file]     // Has access to object variables: [name], [table_name], etc.
pub struct [name] {
[each field]
    [name]: [type],              // Has access to field variables: [name], [type], etc.
[/each]
}
[/each]
```

### Template Syntax

#### Iteration Constructs

**Standard Iteration:**
```blueprint
[each object]
  // Content repeated for each object (record/struct/synthetic)
[/each]

[each enum]
  // Content repeated for each enum
[/each]

[each field] 
  // Content repeated for each field in current object
[/each]

[each case]
  // Content repeated for each enum value
[/each]

[each join]
  // Content repeated for each join relationship
[/each]
```

**Reverse Iteration:**
```blueprint
[eachr object]
  // Process objects in reverse order
[/eachr]
```

#### Conditional Logic

**Basic Conditionals:**
```blueprint
[if condition]content when true[/if]
[ifn condition]content when false[/ifn]
```

**Available Flags:**

| Context | Flag | Description | Example |
|---------|------|-------------|---------|
| Object | `record` | Object is a database record | `[if record]CREATE TABLE...[/if]` |
| Object | `struct` | Object is in-memory struct | `[if struct]pub struct...[/if]` |
| Object | `syn` | Object is synthetic view | `[if syn]-- View definition[/if]` |
| Object | `has_joins` | Object has join relationships | `[if has_joins]-- With joins[/if]` |
| Field | `optional` | Field is optional (nullable) | `[if optional]Option<[/if]` |
| Field | `array` | Field is array type | `[if array]Vec<[/if]` |
| Field | `custom` | Field uses custom enum type | `[if custom]// Enum field[/if]` |
| Field | `local` | Field is local (not a reference) | `[if local]// Local field[/if]` |
| Iteration | `sep` | Not the last item in loop | `[if sep],[/if]` |
| Function | `has_args` | Function has arguments | `[if has_args]([0])[/if]` |

#### Variable Access

**Object Context Variables:**
```blueprint
[each object]
  [name]         // Object name: "User"
  [table_name]   // Database table: "users" 
[/each]
```

**Field Context Variables:**
```blueprint
[each field]
  [name]            // Field name: "user_id"
  [type]            // Resolved type: "Uuid" 
  [object_name]     // Parent object: "User"
  [ref_table]       // Referenced table: "users"
  [ref_field]       // Referenced field: "id"
[/each]
```

**Enum Context Variables:**
```blueprint
[each enum]
  [name]         // Enum name: "UserType"
[/each]

[each case]
  [name]         // Case name: "Admin"
  [value]        // Case value: "Admin" 
  [enum_name]    // Parent enum: "UserType"
[/each]
```

**Join Context Variables:**
```blueprint
[each join]
  [name]          // Join name: "user_posts"
  [local_field]   // Local field: "user_id"
  [ref_field]     // Referenced field: "id"
  [ref_table]     // Referenced table: "users"
  [ref_entity]    // Referenced entity: "User"
  [condition]     // Join condition: "="
[/each]
```

**Variable Transformations:**
All variables support case transformations:
```blueprint
[variable]              // Original: "user_name"
[variable.camelcase]    // camelCase: "userName" 
[variable.titlecase]    // TitleCase: "UserName"
[variable.snakecase]    // snake_case: "user_name"
[variable.uppercase]    // UPPERCASE: "USER_NAME"
[variable.lowercase]    // lowercase: "user_name"
```

### Function Context

Handle field and object functions using the `[func]` construct:

```blueprint
[func namespace.function_name]
  content when function is present
[/func]
```

**Function Arguments:**
Access function arguments using numeric indices:
```blueprint
[func db.default]
  DEFAULT [0]     // First argument: db:default("NOW()") -> "NOW()"
[/func]

[func db.index]
  INDEX ([0][if 1], [1][/if])  // Multiple arguments: db:index("field1", "field2")
[/func]
```

See the [Functions](#functions) section above for the complete list of available functions and their syntax.

### Reference Context

Handle field references and relationships:

```blueprint
[ref]
  // Content when field references another object
  REFERENCES [foreign_table]([foreign_field])
[/ref]
```

### Special Constructs

**Line Breaks:**
```blueprint
[br]  // Insert line break/newline
```

**Import Processing:**
```blueprint
[imports]  // Process and insert all import statements
```

**Import Tags:**
Add imports conditionally when specific content is generated:
```blueprint
[import]use serde::{Serialize, Deserialize};[/import]
```
Import tags are collected and inserted at `[imports]` locations if the surrounding block is evaluated.

**Separators:**
```blueprint
[if sep],[/if]      // Comma separator (not on last item)
[if sep] | [/if]    // Pipe separator  
[if sep][br][/if]   // Line break separator
```

### Complete Blueprint Example

Here's a complete blueprint that generates Rust structs:

```blueprint
[meta id]custom_rust[/meta]
[meta name]Custom Rust Generator[/meta]

[define string]String[/define]
[define int32]i32[/define]
[define int64]i64[/define]
[define float64]f64[/define]
[define boolean]bool[/define]
[define datetime]DateTime<Utc>[/define]
[define uuid]Uuid[/define]

[link uuid]use uuid::Uuid;[/link]
[link datetime]use chrono::{DateTime, Utc};[/link]
[link custom]use crate::types::$;[/link]

[file]models.rs[/file]
//! Generated models - do not edit manually
[imports]

[each enum]
#[derive(Debug, Clone, PartialEq)]
pub enum [name] {
[each case]
    [name][if sep],[/if]
[/each]
}

[/each]
[each object]
[if struct]
#[derive(Debug, Clone)]
pub struct [name] {
[each field]
    pub [name]: [if optional]Option<[/if][if array]Vec<[/if][type][if array]>[/if][if optional]>[/if][if sep],[/if]
[/each]
}

[/if]
[if record]
#[derive(Debug, Clone)]
pub struct [name] {
[each field]
    pub [name]: [if optional]Option<[/if][type][if optional]>[/if][if sep],[/if]
[func db.pk]
    // Primary key field
[/func]
[func db.unique]
    // Unique constraint
[/func]
[/each]
}

[/if]
[/each]
```

### Advanced Blueprint Features

**Multiple Files:**
```blueprint
[each object]
[file][name].rs[/file]
use super::*;

pub struct [name] {
    // fields here...
}
[/each]
```

**Conditional File Generation:**
```blueprint  
[each object]
[if record]
[file]records/[name].sql[/file]
CREATE TABLE [table_name] (
[each field]
    [name] [type][func db.pk] PRIMARY KEY[/func][if sep],[/if]
[/each]
);
[/if]
[/each]
```

**Complex Conditionals:**
```blueprint
[if record]
[if has_joins]
-- Table with relationships
[/if]
CREATE TABLE [table_name] (
[each field]
    [name] [type][func db.pk] PRIMARY KEY[/func][if optional] NULL[/if][if sep],[/if]
[/each]
);
[/if]
```

For more examples, see the built-in blueprints in [`src/blueprint/core/`](src/blueprint/core/) including:
- **[`rust.blueprint`](src/blueprint/core/rust.blueprint)** - Rust structs with serde
- **[`typescript.blueprint`](src/blueprint/core/typescript.blueprint)** - TypeScript interfaces  
- **[`postgres.blueprint`](src/blueprint/core/postgres.blueprint)** - PostgreSQL DDL
- **[`go.blueprint`](src/blueprint/core/go.blueprint)** - Go structs with JSON tags
- **[`test/markdown.blueprint`](test/markdown.blueprint)** - Documentation generator

### Building from Source

```bash
# Clone the repository
git clone https://github.com/jacksonzamorano/repack
cd repack

# Build the project
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```
