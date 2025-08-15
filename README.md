# Repack: Schema-Driven Code Generation

Welcome to Repack! 🚀 This powerful tool lets you define your data models once and generate code for multiple languages and platforms. Think of it as a universal translator for your data structures - write your schema in Repack's simple syntax, create blueprints for your target languages, and let Repack handle the rest.

## Quick Start

Here's a taste of what Repack can do:

**1. Define your schema (`.repack` file):**
```repack
enum UserType {
    Admin
    User  
    Guest
}

struct User @users {
    id uuid
    name string
    email string
    user_type UserType
    created_date datetime
}
```

**2. Choose your outputs:**
```repack
output typescript @src/types;
output postgres @database;
```

**3. Get generated code automatically!**

## Table of Contents

- [Core Concepts](#core-concepts)
- [Repack Language Reference](#repack-language-reference)
  - [Data Types](#data-types)
  - [Structs](#structs)
  - [Enums](#enums)
  - [Fields](#fields)
  - [Queries](#queries)
  - [Inheritance](#inheritance)
  - [Snippets](#snippets)
- [Blueprint Language Reference](#blueprint-language-reference)
  - [Template Syntax](#template-syntax)
  - [Variables](#variables)
  - [Control Flow](#control-flow)
  - [File Generation](#file-generation)
- [Complete Examples](#complete-examples)
- [Best Practices](#best-practices)

## Core Concepts

Repack consists of two complementary languages:

- **Repack Schema Language**: Define your data structures, relationships, and database queries
- **Blueprint Template Language**: Create templates that generate code from your schemas

The workflow is simple:
1. Write `.repack` files defining your data models
2. Use existing blueprints or create custom `.blueprint` templates  
3. Run Repack to generate code in any language you need

## Repack Language Reference

### Data Types

Repack provides a comprehensive type system with built-in primitives and custom types.

#### Primitive Types

| Type | Description | Example Usage |
|------|-------------|---------------|
| `string` | UTF-8 text | `name string` |
| `int32` | 32-bit signed integer | `count int32` |
| `int64` | 64-bit signed integer | `big_number int64` |
| `float64` | 64-bit floating point | `price float64` |
| `boolean` | True/false value | `is_active boolean` |
| `datetime` | Timestamp | `created_date datetime` |
| `uuid` | Universally unique identifier | `id uuid` |
| `bytes` | Byte array | `file_data bytes` |

#### Type Modifiers

| Modifier | Syntax | Description |
|----------|--------|-------------|
| Optional | `type?` | Field can be null/undefined |
| Array | `type[]` | Field is a collection |
| Optional Array | `type[]?` | Array itself can be null |

**Examples:**
```repack
struct Product {
    id uuid                    // Required UUID
    name string               // Required string
    description string?       // Optional string
    tags string[]            // Required array of strings
    images string[]?         // Optional array of strings
    price float64            // Required number
}
```

### Structs

Structs are the core building blocks representing entities in your system.

#### Basic Syntax

```repack
struct EntityName [@table_name] [: ParentStruct] [#category]* {
    // fields, queries, and functions
}
```

#### Struct Components

| Component | Purpose | Example |
|-----------|---------|---------|
| `@table_name` | Database table mapping | `@users` |
| `: ParentStruct` | Inheritance | `: BaseEntity` |
| `#category` | Organization/filtering | `#model` |

**Complete Example:**
```repack
struct User @users : BaseEntity #model #api {
    name string
    email string  
    is_admin boolean
    
    query GetByEmail(_email string) = "SELECT $fields FROM $locations WHERE $email = $_email" : one
}
```

### Enums

Enums define fixed sets of possible values.

#### Basic Syntax

```repack
enum EnumName [#category]* {
    ValueA ["custom_string"]
    ValueB ["another_string"]
    ValueC
}
```

#### Enum Features

| Feature | Description | Example |
|---------|-------------|---------|
| Implicit Values | Uses the case name as value | `Admin` → `"Admin"` |
| Custom Values | Override with custom string | `Admin "ADMIN_USER"` |
| Categories | Group enums for filtering | `#status` |

**Examples:**
```repack
// Simple enum
enum Status {
    Active
    Inactive
    Pending
}

// Enum with custom values
enum UserRole #auth {
    Admin "ADMIN_USER"
    Editor "EDITOR_USER"  
    Viewer "VIEWER_USER"
}
```

### Fields

Fields define the properties of your structs.

#### Field Types

| Field Type | Syntax | Description |
|------------|--------|-------------|
| Direct | `name Type` | Standard field |
| External Reference | `name Other.field` | Reference to another struct's field |
| Parent Reference | `name super.field` | Reference to inherited field |

#### Field Functions

Fields can have attached functions that modify behavior:

```repack
struct User {
    id uuid db:pk                                    // Database primary key
    email string db:unique                           // Database unique constraint  
    full_name string db:as("first_name || ' ' || last_name")  // Computed field
    created_date datetime db:default("NOW()")       // Default value
}
```

**Common Field Functions:**

| Namespace | Function | Purpose | Example |
|-----------|----------|---------|---------|
| `db` | `pk` | Primary key | `db:pk` |
| `db` | `unique` | Unique constraint | `db:unique` |
| `db` | `default(value)` | Default value | `db:default("NOW()")` |
| `db` | `as(expression)` | Computed field | `db:as("LOWER(email)")` |

### Queries

Repack supports three types of queries for database operations.

#### Query Types Comparison

| Type | Purpose | Syntax | Return Options |
|------|---------|--------|----------------|
| Manual | Custom SQL | `query Name(args) = "SQL" : return_type` | `:one`, `:many`, none |
| Auto Insert | Generated INSERT | `insert Name(field1, field2) : return_type` | `:one`, `:many`, none |
| Auto Update | Generated UPDATE | `update Name(args) = "SET clause" : return_type` | `:one`, `:many`, none |

#### Query Interpolation Variables

Repack provides powerful query interpolation:

| Variable | Description | Example Output |
|----------|-------------|----------------|
| `$fields` | All struct fields with aliases | `users.id AS id, users.name AS name` |
| `$locations` | Table name with joins | `users INNER JOIN tokens t ON ...` |
| `$table` | Primary table name | `users` |
| `$fieldName` | Qualified field reference | `users.email` |
| `$#fieldName` | Unqualified field reference | `email` |
| `$argName` | Parameter placeholder | `$1`, `$2`, etc. |

**Query Examples:**
```repack
struct User @users {
    id uuid
    name string
    email string
    
    // Manual query
    query GetByEmail(_email string) = "SELECT $fields FROM $locations WHERE $email = $_email" : one
    
    // Auto insert
    insert CreateUser(id, name, email) : one
    
    // Auto update  
    update UpdateEmail(_id uuid, _email string) = "SET $email = $_email WHERE $id = $_id" : one
}
```

### Inheritance

Repack supports single inheritance for sharing common fields.

```repack
struct BaseEntity {
    id uuid
    created_date datetime
    updated_date datetime
}

struct User : BaseEntity @users {
    name string
    email string
    
    // Reference parent field
    user_id super.id
}
```

**Inheritance Rules:**
- Child inherits parent's table name (if any)
- Use `super.field` to reference parent fields
- Only single inheritance is supported

### Snippets

Snippets provide reusable field collections.

#### Defining Snippets

```repack
snippet Timestamps {
    created_date datetime db:default("NOW()")
    updated_date datetime db:default("NOW()")
}

snippet Identifiable {
    id uuid db:pk
}
```

#### Using Snippets

```repack
struct User @users {
    !Identifiable    // Includes id field
    !Timestamps      // Includes created_date and updated_date
    
    name string
    email string
}
```

**Key Benefits:**
- Reduce duplication across structs
- Ensure consistency of common patterns
- Easy to maintain shared functionality

### Advanced Features

#### Joins

Define relationships between structs for complex queries:

```repack
struct UserWithTokens : User {
    join(t Token) = "INNER JOIN $name ON $super.id = $t.user_id"
    
    token_value t.token_value
    
    query GetUserTokens(_id uuid) = "SELECT $fields FROM $locations WHERE $user_id = $_id" : many
}
```

#### Categories and Filtering

Use categories to organize and filter your schema:

```repack
struct User #model #api {
    // This struct has both 'model' and 'api' categories
}

enum Status #enums {
    // This enum has the 'enums' category
}

// Generate only structs with 'api' category
output typescript @types #api;
```

## Blueprint Language Reference

Blueprints are templates that transform your Repack schemas into target language code.

### Template Syntax

Blueprints use bracketed directives: `[directive]content[/directive]`

#### Auto-Closing vs Block Directives

| Type | Syntax | Purpose |
|------|--------|---------|
| Auto-closing | `[variable]` | Insert single values |
| Block | `[directive]...[/directive]` | Control flow and iteration |

### Variables

Access schema data through variables:

#### Core Variables

| Variable | Context | Description |
|----------|---------|-------------|
| `[name]` | Any | Entity name |
| `[type]` | Field | Field type |
| `[table_name]` | Struct | Database table name |
| `[value]` | Enum case | Enum case value |
| `[query]` | Query | Rendered SQL |

#### Variable Modifiers

Transform variable output with dot notation:

| Modifier | Effect | Example |
|----------|--------|---------|
| `uppercase` | ALL CAPS | `[name.uppercase]` |
| `lowercase` | all lowercase | `[name.lowercase]` |
| `titlecase` | Title Case | `[name.titlecase]` |
| `camelcase` | camelCase | `[name.camelcase]` |

**Example:**
```blueprint
// Input: user_profile
[name.titlecase]     // → User Profile
[name.camelcase]     // → userProfile
[name.uppercase]     // → USER_PROFILE
```

### Control Flow

#### Iteration

| Directive | Purpose | Context Required |
|-----------|---------|------------------|
| `[each struct]` | Loop through structs | Global |
| `[each field]` | Loop through fields | Inside struct |
| `[each enum]` | Loop through enums | Global |
| `[each case]` | Loop through enum cases | Inside enum |
| `[each query]` | Loop through queries | Inside struct |

**Example:**
```blueprint
[each struct]
export interface [name] {
[each field]
    [name]: [type];
[/each]
}
[/each]
```

#### Conditionals

| Directive | Purpose | Usage |
|-----------|---------|-------|
| `[if flag]` | Execute if true | `[if optional]` |
| `[ifn flag]` | Execute if false | `[ifn returns_none]` |

**Available Flags:**

| Flag | Context | When True |
|------|---------|-----------|
| `optional` | Field | Field is optional |
| `array` | Field | Field is array |
| `returns_one` | Query | Query returns single result |
| `returns_many` | Query | Query returns multiple results |
| `returns_none` | Query | Query returns no results |
| `sep` | Iteration | Not the last item (for commas) |

### File Generation

#### File Directives

| Directive | Purpose | Example |
|-----------|---------|---------|
| `[file]name[/file]` | Set output file | `[file][name].ts[/file]` |
| `[imports]` | Import insertion point | Place where imports appear |

#### Import System

```blueprint
// Define import template
[link custom]import type { $ } from './$'[/link]

// Import will be auto-generated when custom types are used
[imports]  // Imports appear here

export interface User {
    profile: UserProfile;  // This triggers the import
}
```

### Type Definitions

Map Repack types to target language types:

```blueprint
[define string]string[/define]
[define int64]number[/define]  
[define boolean]boolean[/define]
[define uuid]string[/define]
```

### Complete Blueprint Examples

#### TypeScript Interface Generator

```blueprint
[meta id]typescript[/meta]
[meta name]TypeScript Interfaces[/meta]

[define string]string[/define]
[define int64]number[/define]
[define boolean]boolean[/define]
[link custom]import type { $ } from './$'[/link]

[each struct]
[file][name].ts[/file]
[imports]

export interface [name] {
[each field]
    [name][if optional]?[/if]: [type][if array][][\if];
[/each]
}
[/each]

[file]index.ts[/file]
[each struct]
export type { [name] } from './[name]';
[/each]
```

#### SQL Schema Generator

```blueprint
[meta id]postgres[/meta]
[meta name]PostgreSQL Schema[/meta]

[define string]TEXT[/define]
[define int64]BIGINT[/define]
[define boolean]BOOLEAN[/define]
[define uuid]UUID[/define]
[define datetime]TIMESTAMPTZ[/define]

[file]schema.sql[/file]
BEGIN;

[each enum]
CREATE TYPE [name] AS ENUM([each case]'[value]'[if sep], [/if][/each]);
[/each]

[each struct]
CREATE TABLE [table_name] (
[each field]
    [name] [type][ifn optional] NOT NULL[/ifn][if sep],[/if]
[/each]
);
[/each]

COMMIT;
```

## Complete Examples

### E-commerce System

**Schema Definition (`ecommerce.repack`):**

```repack
blueprint "typescript.blueprint"
blueprint "postgres.blueprint"

output typescript @src/types;
output postgres @database;

snippet BaseEntity {
    id uuid db:pk
    created_date datetime db:default("NOW()")
    updated_date datetime db:default("NOW()")
}

enum OrderStatus #enums {
    Pending
    Processing  
    Shipped
    Delivered
    Cancelled
}

enum UserRole #enums {
    Customer
    Admin
    Moderator
}

struct User @users #model {
    !BaseEntity
    
    email string db:unique
    first_name string
    last_name string
    role UserRole db:default("'Customer'")
    
    // Computed field
    full_name string db:as("first_name || ' ' || last_name")
    
    query GetByEmail(_email string) = "SELECT $fields FROM $locations WHERE $email = $_email" : one
    query GetAdmins() = "SELECT $fields FROM $locations WHERE $role = 'Admin'" : many
    
    insert CreateUser(id, email, first_name, last_name) : one
    update UpdateName(_id uuid, _first string, _last string) = "SET first_name = $_first, last_name = $_last WHERE $id = $_id" : one
}

struct Product @products #model {
    !BaseEntity
    
    name string
    description string?
    price float64
    inventory_count int32
    is_active boolean db:default("true")
    
    query GetActive() = "SELECT $fields FROM $locations WHERE $is_active = true" : many
    query SearchByName(_name string) = "SELECT $fields FROM $locations WHERE $name ILIKE '%' || $_name || '%'" : many
    
    insert CreateProduct(id, name, price, inventory_count) : one
}

struct Order @orders #model {
    !BaseEntity
    
    user_id uuid
    status OrderStatus db:default("'Pending'")
    total_amount float64
    
    // External field reference
    user_email User.email
    
    query GetByUser(_user_id uuid) = "SELECT $fields FROM $locations WHERE $user_id = $_user_id ORDER BY created_date DESC" : many
    query GetByStatus(_status OrderStatus) = "SELECT $fields FROM $locations WHERE $status = $_status" : many
    
    insert CreateOrder(id, user_id, total_amount) : one
    update UpdateStatus(_id uuid, _status OrderStatus) = "SET $status = $_status WHERE $id = $_id"
}

struct OrderItem @order_items #model {
    !BaseEntity
    
    order_id uuid
    product_id uuid  
    quantity int32
    unit_price float64
    
    // Computed total
    line_total float64 db:as("quantity * unit_price")
    
    query GetByOrder(_order_id uuid) = "SELECT $fields FROM $locations WHERE $order_id = $_order_id" : many
}

// Join example - Orders with user information
struct OrderWithUser : Order {
    join(u User) = "INNER JOIN $name ON $super.user_id = $u.id"
    
    user_name u.full_name
    user_email u.email
    
    query GetOrdersWithUsers() = "SELECT $fields FROM $locations ORDER BY created_date DESC" : many
}
```

This schema generates:

1. **TypeScript interfaces** with proper typing
2. **PostgreSQL schema** with tables, constraints, and indexes  
3. **Database functions** for common operations
4. **Type-safe query builders**

### Blog System

**Schema Definition (`blog.repack`):**

```repack
snippet Auditable {
    created_date datetime db:default("NOW()")  
    updated_date datetime db:default("NOW()")
    created_by uuid
    updated_by uuid?
}

enum PostStatus {
    Draft
    Published
    Archived
}

struct Author @authors {
    id uuid db:pk
    !Auditable
    
    name string
    email string db:unique
    bio string?
    website string?
    
    query GetByEmail(_email string) = "SELECT $fields FROM $locations WHERE $email = $_email" : one
}

struct Category @categories {
    id uuid db:pk
    !Auditable
    
    name string db:unique
    slug string db:unique
    description string?
    
    query GetBySlug(_slug string) = "SELECT $fields FROM $locations WHERE $slug = $_slug" : one
}

struct Post @posts {
    id uuid db:pk
    !Auditable
    
    title string
    slug string db:unique
    content string
    excerpt string?
    status PostStatus db:default("'Draft'")
    published_date datetime?
    
    author_id uuid
    category_id uuid
    
    // External references
    author_name Author.name
    category_name Category.name
    
    query GetPublished() = "SELECT $fields FROM $locations WHERE $status = 'Published' ORDER BY published_date DESC" : many
    query GetByAuthor(_author_id uuid) = "SELECT $fields FROM $locations WHERE $author_id = $_author_id" : many
    query GetByCategory(_category_id uuid) = "SELECT $fields FROM $locations WHERE $category_id = $_category_id" : many
    
    insert CreatePost(id, title, slug, content, author_id, category_id) : one
    update Publish(_id uuid) = "SET status = 'Published', published_date = NOW() WHERE $id = $_id"
}

struct Tag @tags {
    id uuid db:pk
    name string db:unique
    color string?
}

struct PostTag @post_tags {
    post_id uuid
    tag_id uuid
    
    query GetTagsByPost(_post_id uuid) = "SELECT $fields FROM $locations WHERE $post_id = $_post_id" : many
    query GetPostsByTag(_tag_id uuid) = "SELECT $fields FROM $locations WHERE $tag_id = $_tag_id" : many
}
```

## Best Practices

### Blueprint Development

1. **Start simple**: Begin with basic templates and add complexity gradually
2. **Handle edge cases**: Use conditionals for optional fields and special cases
3. **Maintain consistency**: Establish naming conventions in your templates
4. **Test thoroughly**: Verify generated code with various schema combinations

### Project Structure

```
project/
├── schemas/
│   ├── user.repack
│   ├── product.repack
│   └── order.repack
├── blueprints/
│   ├── typescript.blueprint
│   ├── postgres.blueprint
│   └── docs.blueprint  
├── generated/
│   ├── types/
│   ├── database/
│   └── docs/
└── repack.config
```

### Blueprint Debugging

1. **Use debug blueprint**: Create simple debug templates to inspect data
2. **Check variable scope**: Ensure variables are available in current context
3. **Validate conditionals**: Test flag conditions with debug output
4. **Verify imports**: Ensure import templates are correctly defined

Happy coding with Repack! 🎉

