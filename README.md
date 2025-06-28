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

### Blueprint Development

To create custom blueprints, see the existing blueprints in [`src/blueprint/core/`](src/blueprint/core/) as examples. Blueprints use a template syntax with variables, loops, and conditionals to generate target code.

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
