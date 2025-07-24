# Repack

**Powerful schema-first code generation and deployment configuration for multiple target languages**

Repack is a comprehensive schema-first development tool that allows you to define your data models, relationships, and deployment configurations once, then generate consistent, type-safe code across multiple programming languages, platforms, and environments. Define your objects, enums, relationships, and deployment configurations in a simple schema language, then generate database schemas, API models, data structures, and deployment configurations for Rust, TypeScript, Go, PostgreSQL, and more.

## What Makes Repack Different

Unlike simple code generators, Repack provides a complete development ecosystem:

- **📋 Schema-First Development**: Define data models once, generate everywhere
- **⚙️ Configuration Management**: Manage deployment configurations and environment-specific settings
- **🔗 Advanced Relationships**: Sophisticated join system with explicit and implicit relationships
- **📚 Multi-Modal Generation**: Generate code, documentation, and configuration files
- **🎯 Type Safety**: Fully typed output for each target language
- **🔧 Extensible**: Custom blueprints for any target language or framework

## Quick Start Guide

### 1. Installation

```bash
# Build from source
git clone https://github.com/jacksonzamorano/repack
cd repack
cargo build --release
cargo install --path .
```

### 2. Create Your First Schema

Create `example.repack`:

```repack
// Define output targets for different purposes
output rust @src/models #model;
output postgres @database #model;
output typescript @frontend/types #model;

// Define reusable configuration templates
configuration DatabaseConfig {
    host
    port  
    username
    password
    database_name
}

// Create environment-specific instances
instance Production: DatabaseConfig @prod {
    host "prod.example.com"
    port "5432"
    username "prod_user"
    password "secure_password"
    database_name "app_production"
}

instance Development: DatabaseConfig @dev {
    host "localhost"
    port "5432"
    username "dev_user"
    password "dev_password"
    database_name "app_development"
}

// Define an enum
enum UserRole #model {
    Admin
    Moderator
    Member
    Guest
}

// Define a database record
record User @users #model {
    id uuid db:pk
    created_at datetime db:default("NOW()")
    updated_at datetime?
    email string db:unique
    username string db:unique
    role UserRole
    is_active boolean db:default("true")
    last_login datetime?
    
    // Define database indexes
    db:index("email")
    db:index("username") 
    db:index("role", "is_active")
}

// Define related record with explicit join
record Profile @user_profiles #model {
    id uuid db:pk
    user_id ref(User.id)
    display_name string
    bio string?
    avatar_url string?
    
    // Explicit join definition
    ^ user_profile self.user_id = User.id
}

// Create a synthetic view combining data
synthetic UserWithProfile: Profile #model {
    *  // Include all Profile fields
    email from(user_id.email)        // Implicit join
    username from(user_id.username)
    role from(user_id.role)
    user_display_name with(user_profile.display_name)  // Explicit join
}

// In-memory data structure for API responses
struct UserListResponse #model {
    users UserWithProfile[]
    total_count int32
    page int32
    per_page int32
}
```

### 3. Generate Your Code

```bash
# Generate all code files (Rust, SQL, TypeScript)
repack build example.repack

# Generate only documentation
repack document example.repack

# Generate configuration files for production environment  
repack configure prod example.repack

# Clean all generated files
repack clean example.repack
```

### 4. What Gets Generated

**Database Schema (`database/model.sql`):**
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    email TEXT UNIQUE NOT NULL,
    username TEXT UNIQUE NOT NULL,
    role user_role NOT NULL,
    is_active BOOLEAN DEFAULT true,
    last_login TIMESTAMPTZ
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_role_is_active ON users(role, is_active);
-- ... more tables and views
```

**Rust Models (`src/models/model.rs`):**
```rust
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum UserRole {
    Admin,
    Moderator,
    Member,
    Guest,
}

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub email: String,
    pub username: String,
    pub role: UserRole,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
}
// ... more structs
```

**TypeScript Interfaces (`frontend/types/*.ts`):**
```typescript
export enum UserRole {
    Admin = "Admin",
    Moderator = "Moderator", 
    Member = "Member",
    Guest = "Guest"
}

export interface User {
    id: string;
    createdAt: Date;
    updatedAt?: Date;
    email: string;
    username: string;
    role: UserRole;
    isActive: boolean;
    lastLogin?: Date;
}
// ... more interfaces
```

**Production Configuration (`prod_config.json`):**
```json
{
    "database": {
        "host": "prod.example.com",
        "port": "5432",
        "username": "prod_user", 
        "password": "secure_password",
        "database_name": "app_production"
    }
}
```

## Core Concepts

### Schema Language Fundamentals

#### Object Types

Repack supports three distinct object types, each optimized for different use cases:

| Object Type | Purpose | Database Storage | Inheritance | Arrays | References |
|-------------|---------|------------------|-------------|--------|------------|
| **`record`** | Database entities | ✅ Required table name | ❌ No | ❌ No | ✅ Foreign keys |
| **`struct`** | In-memory data | ❌ Memory only | ❌ No | ✅ Yes | ✅ All types |
| **`synthetic`** | Computed views | ✅ Inherits from parent | ✅ Yes | ❌ No | ✅ Via joins |

**Examples:**

```repack
// Record: Stored in database
record User @users #model {
    id uuid db:pk
    name string
    email string db:unique
}

// Struct: In-memory only, supports arrays
struct ApiResponse #api {
    users User[]
    metadata struct {
        total_count int32
        page_size int32
    }
}

// Synthetic: Extends record with computed fields
synthetic PublicUser: User #api {
    *           // Include all User fields
    - password  // Exclude sensitive fields
    full_name string  // Add computed fields
}
```

#### Field Types and Modifiers

**Core Types:**
```repack
name string              // UTF-8 text
age int32               // 32-bit integer  
big_number int64        // 64-bit integer
price float64           // 64-bit floating point
is_active boolean       // True/false value
created_at datetime     // Timestamp with timezone
id uuid                 // UUID v4 identifier
```

**Field Modifiers:**
```repack
required_field type          // Required field
optional_field type?         // Optional/nullable field
array_field type[]          // Array of values (structs only)
optional_array type[]?      // Optional array
```

#### Advanced Relationships

**Reference Types:**
```repack
// Direct type reference
user_role UserRole

// Foreign key reference  
user_id ref(User.id)

// Implicit join (automatic relationship)
user_name from(user_id.name)

// Explicit join (defined relationship)
profile_name with(user_profile.display_name)
```

**Explicit Join Definitions:**
```repack
record Post @posts #model {
    id uuid db:pk
    author_id ref(User.id)
    title string
    
    // Define explicit join for later use
    ^ post_author self.author_id = User.id
}

synthetic PostWithAuthor: Post #view {
    *  // Include all Post fields
    author_name with(post_author.name)
    author_email with(post_author.email)
}
```

#### Configuration and Instance System

**Configuration Templates:**
```repack
// Define reusable configuration schema
configuration ApiConfig {
    base_url
    api_key
    timeout_seconds
    enable_caching
}

configuration DatabaseConfig {
    host
    port
    database_name
    ssl_mode
}
```

**Environment Instances:**
```repack
// Create environment-specific configurations
instance Production: ApiConfig @prod {
    base_url "https://api.production.com"
    api_key "prod_api_key_here"
    timeout_seconds "30"
    enable_caching "true"
}

instance Development: ApiConfig @dev {
    base_url "http://localhost:3000"
    api_key "dev_api_key"
    timeout_seconds "5"
    enable_caching "false"
}

instance TestDB: DatabaseConfig @test {
    host "localhost"
    port "5433"
    database_name "test_db"
    ssl_mode "disable"
}
```

#### Field Functions

Functions provide metadata and behavior for fields and objects:

**Database Functions:**
```repack
record User @users #model {
    id uuid db:pk                           // Primary key
    email string db:unique                  // Unique constraint
    created_at datetime db:default("NOW()") // Default value
    sequence_id int32 db:identity           // Auto-increment
    full_name string db:generated           // Generated column
    
    // Object-level indexes
    db:index("email")                       // Single field index
    db:index("created_at", "updated_at")    // Composite index
}
```

#### Snippets for Code Reuse

```repack
// Define reusable field groups
snippet timestamps {
    created_at datetime db:default("NOW()")
    updated_at datetime?
}

snippet audit_fields {
    created_by uuid
    updated_by uuid?
}

snippet base_entity {
    id uuid db:pk
    !timestamps  // Include timestamps snippet
    !audit_fields // Include audit fields snippet
}

// Use in objects
record User @users #model {
    !base_entity  // Include all base entity fields
    name string
    email string db:unique
}
```

#### Categories and Output Filtering

Control which objects are generated for each output:

```repack
// Generate different objects for different outputs
output rust @backend #model #internal;
output typescript @frontend #model #api;
output postgres @database #model !temp_tables;

enum Status #model #api {    // Generated for both rust and typescript
    Active
    Inactive
}

record User @users #model {  // Generated for rust and postgres only
    id uuid db:pk
    name string
}

struct UserResponse #api {   // Generated for typescript only
    user User
    permissions string[]
}

record TempData @temp #model temp_tables {  // Excluded from postgres output
    id uuid db:pk
    data string
}
```

## Command Reference

### Build Commands

```bash
# Build all code generation targets
repack build schema.repack

# Build specific environment configurations
repack configure production schema.repack
repack configure development schema.repack
repack configure staging schema.repack

# Generate documentation
repack document schema.repack

# Clean generated files
repack clean schema.repack

# Verbose output for debugging
repack build schema.repack --verbose
```

### Schema File Organization

```repack
// Import other schema files
import "common/base.repack"
import "modules/*.repack"          // Import all .repack files in directory

// Load custom blueprints
blueprint "custom/rust_enhanced.blueprint"
blueprint "deployment/kubernetes.blueprint"

// Configure outputs with options
output rust @src/models #model {
    derive_debug true
    serde_support true
    package_name my_models
}

output postgres @database #model #view {
    schema_name public
    include_migrations true
}
```

## Blueprint Development

### Creating Custom Blueprints

Blueprints are template files that define how to generate code for specific targets. Create a custom blueprint:

**`custom_rust.blueprint`:**
```blueprint
[meta id]custom_rust[/meta]
[meta name]Enhanced Rust Generator[/meta]
[meta kind]code[/meta]

// Type mappings
[define string]String[/define]
[define int32]i32[/define]
[define boolean]bool[/define]
[define datetime]DateTime<Utc>[/define]
[define uuid]Uuid[/define]

// Import management
[link uuid]use uuid::Uuid;[/link]
[link datetime]use chrono::{DateTime, Utc};[/link]
[link custom]use crate::types::$;[/link]

[file]models.rs[/file]
//! Generated models
[imports]

[each enum]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum [name] {
[each case]
    [name][if sep],[/if]
[/each]
}

[/each]

[each object]
[if record]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct [name] {
[each field]
    pub [name]: [if optional]Option<[/if][type][if optional]>[/if][if sep],[/if]
[/each]
}

impl [name] {
    pub fn table_name() -> &'static str {
        "[table_name]"
    }
}
[/if]
[/each]
```

### Configuration Blueprint Example

**`deployment.blueprint`:**
```blueprint
[meta id]k8s_deploy[/meta]
[meta name]Kubernetes Deployment[/meta] 
[meta kind]configure[/meta]

[each DatabaseConfig]
[file]k8s/[name.lowercase]-configmap.yaml[/file]
apiVersion: v1
kind: ConfigMap
metadata:
  name: [name.lowercase]-config
data:
  DATABASE_HOST: "[host]"
  DATABASE_PORT: "[port]"
  DATABASE_NAME: "[database_name]"
  DATABASE_SSL_MODE: "[ssl_mode]"
---
apiVersion: v1
kind: Secret
metadata:
  name: [name.lowercase]-secret
type: Opaque
stringData:
  DATABASE_USERNAME: "[username]"
  DATABASE_PASSWORD: "[password]"
[/each]
```

### Documentation Blueprint Example

**`docs.blueprint`:**
```blueprint
[meta id]api_docs[/meta]
[meta name]API Documentation[/meta]
[meta kind]document[/meta]

[file]API_REFERENCE.md[/file]
# API Reference

## Data Models

[each object]
[if record]
### [name]
**Database Table**: `[table_name]`

[each field]
- **[name]**: `[type]`[if optional] (optional)[/if]
[func db.pk]  - 🔑 Primary Key[/func]
[func db.unique]  - ⚠️ Must be unique[/func]
[func db.default]  - 📝 Default: `[0]`[/func]
[/each]

[/if]
[if struct]
### [name]
**Data Structure** (in-memory only)

[each field]
- **[name]**: `[type]`[if optional] (optional)[/if][if array] (array)[/if]
[/each]

[/if]
[/each]

## Enums

[each enum]
### [name]
[each case]
- `[name]`
[/each]

[/each]

## Configuration Templates

[each configuration]
### [name]
[each field]
- **[name]**: Configuration parameter
[/each]

**Instances:**
[each instance]
- **[name]** (`@[environment]`): [environment.titlecase] environment configuration
[/each]

[/each]
```

### Advanced Template Features

**Template Constructs:**
```blueprint
[each object]               // Iterate over objects
[eachr object]              // Reverse iteration
[if record]content[/if]     // Conditional rendering
[ifn record]content[/ifn]   // Negative conditional
[func db.pk]content[/func]  // Function-based conditional
[nfunc db.pk]content[/nfunc] // Negative function conditional

// Variable transformations
[name]                      // Original: "user_profile"
[name.camelcase]           // camelCase: "userProfile"
[name.titlecase]           // TitleCase: "UserProfile"
[name.snakecase]           // snake_case: "user_profile"
[name.uppercase]           // UPPERCASE: "USER_PROFILE"
[name.lowercase]           // lowercase: "user_profile"

// Advanced transformations
[package.split_period_first]  // "com.example.app" -> "com"
[package.split_period_last]   // "com.example.app" -> "app"
```

**Shell Integration:**
```blueprint
[exec]npm install[/exec]    // Execute commands (with user confirmation)
[exec]cargo fmt[/exec]      // Format generated Rust code
```

### Built-in Blueprint Reference

Repack includes several built-in blueprints:

- **`rust`** - Rust structs with serde support and comprehensive derive attributes
- **`typescript`** - TypeScript interfaces with proper type mapping
- **`postgres`** - PostgreSQL DDL with indexes, constraints, and relationships
- **`go`** - Go structs with JSON tags and proper type mapping
- **`markdown`** - Documentation generator for data models

See [`src/blueprint/core/`](src/blueprint/core/) for implementation details.

## Real-World Examples

### E-commerce Platform Schema

```repack
import "common/audit.repack"

configuration AppConfig {
    app_name
    base_url
    api_version
    enable_debug
}

instance Production: AppConfig @prod {
    app_name "E-commerce Platform"
    base_url "https://api.shop.com"
    api_version "v2"
    enable_debug "false"
}

output rust @src/models #model #api;
output postgres @database #model;
output typescript @frontend/types #model #api;

snippet auditable {
    created_at datetime db:default("NOW()")
    updated_at datetime?
    created_by uuid
    updated_by uuid?
}

enum OrderStatus #model #api {
    Pending
    Processing
    Shipped
    Delivered
    Cancelled
    Refunded
}

enum ProductCategory #model #api {
    Electronics
    Clothing
    Books
    Home
    Sports
}

record User @users #model {
    !auditable
    id uuid db:pk
    email string db:unique
    username string db:unique
    first_name string
    last_name string
    is_active boolean db:default("true")
    
    db:index("email")
    db:index("username")
}

record Product @products #model {
    !auditable
    id uuid db:pk
    sku string db:unique
    name string
    description string?
    price float64
    category ProductCategory
    stock_quantity int32
    is_active boolean db:default("true")
    
    db:index("sku")
    db:index("category", "is_active")
    db:index("price")
}

record Order @orders #model {
    !auditable
    id uuid db:pk
    user_id ref(User.id)
    status OrderStatus
    total_amount float64
    shipping_address string
    
    ^ order_user self.user_id = User.id
    
    db:index("user_id", "status")
    db:index("created_at")
}

record OrderItem @order_items #model {
    id uuid db:pk
    order_id ref(Order.id)
    product_id ref(Product.id)
    quantity int32
    unit_price float64
    
    ^ item_order self.order_id = Order.id
    ^ item_product self.product_id = Product.id
    
    db:index("order_id")
    db:index("product_id")
}

synthetic OrderWithDetails: Order #model #api {
    *
    user_email with(order_user.email)
    user_name with(order_user.first_name)
}

struct OrderResponse #api {
    order OrderWithDetails
    items struct {
        product_name string
        product_sku string
        quantity int32
        unit_price float64
    }[]
    total_items int32
}

struct UserDashboard #api {
    user User
    recent_orders OrderWithDetails[]
    order_count int32
    total_spent float64
}
```

### Microservices Configuration

```repack
configuration ServiceConfig {
    service_name
    port
    log_level
    metrics_enabled
}

configuration DatabaseConfig {
    host
    port
    database_name
    max_connections
    ssl_mode
}

configuration RedisConfig {
    host
    port
    database_index
    password
}

// Production environment
instance UserServiceProd: ServiceConfig @prod {
    service_name "user-service"
    port "8080"
    log_level "info"
    metrics_enabled "true"
}

instance OrderServiceProd: ServiceConfig @prod {
    service_name "order-service"
    port "8081"
    log_level "info"
    metrics_enabled "true"
}

instance ProdDB: DatabaseConfig @prod {
    host "prod-db.cluster.local"
    port "5432"
    database_name "ecommerce"
    max_connections "20"
    ssl_mode "require"
}

instance ProdRedis: RedisConfig @prod {
    host "redis.cluster.local"
    port "6379"
    database_index "0"
    password "secure_redis_password"
}

// Development environment
instance DevDB: DatabaseConfig @dev {
    host "localhost"
    port "5432"
    database_name "ecommerce_dev"
    max_connections "5"
    ssl_mode "disable"
}
```

## Best Practices

### Schema Organization

1. **Separate Concerns**: Keep models, configurations, and deployment settings in separate files
2. **Use Imports**: Organize related schemas in modules and import them
3. **Consistent Naming**: Use clear, consistent naming conventions across your schema
4. **Categories**: Use categories to control what gets generated for each target

### Performance Optimization

1. **Selective Generation**: Use categories to generate only what you need for each target
2. **Index Strategy**: Define database indexes for frequently queried fields
3. **Join Optimization**: Use explicit joins for complex relationships

### Development Workflow

1. **Version Control**: Keep generated files out of version control, generate during build
2. **CI/CD Integration**: Generate code and configurations as part of your build pipeline
3. **Environment Management**: Use instance configurations for environment-specific deployments
4. **Documentation**: Keep your schema files well-documented with comments

### Blueprint Development

1. **Start Simple**: Begin with existing blueprints and modify them
2. **Test Incrementally**: Test blueprint changes with small schemas first
3. **Use Variables**: Leverage variable transformations for consistent naming
4. **Error Handling**: Include validation and error checking in your templates

## Migration Guide

### From Version 1.x

If you're upgrading from an earlier version:

1. **Configuration System**: The new configuration/instance system replaces environment-specific output paths
2. **Join Syntax**: Explicit joins now use `^` syntax instead of inline definitions
3. **Command Structure**: Commands now use `build`, `configure`, `document` subcommands
4. **Blueprint Kinds**: Blueprints now specify their purpose with `[meta kind]` tags

### Schema Updates

Update your existing schemas to take advantage of new features:

```repack
// Old approach
output rust @src/models/prod #model;
output rust @src/models/dev #model;

// New approach with configurations
configuration AppSettings {
    database_url
    api_key
}

instance Production: AppSettings @prod {
    database_url "prod://..."
    api_key "prod_key"
}

output rust @src/models #model;
```

## Troubleshooting

### Common Issues

**Build Failures:**
- Check that all referenced objects exist and are properly categorized
- Verify blueprint syntax and ensure required meta tags are present
- Confirm file paths are accessible and directories exist

**Type Errors:**
- Ensure custom types (enums) are defined before use
- Check field reference syntax for joins and relationships
- Verify array types are only used in struct objects

**Configuration Problems:**
- Confirm instance names match configuration templates
- Check that environment tags are consistent
- Verify configuration values are properly quoted

### Debug Mode

Use verbose output for detailed information:

```bash
repack build schema.repack --verbose
```

### Getting Help

- Check the [`test/`](test/) directory for working examples
- Review built-in blueprints in [`src/blueprint/core/`](src/blueprint/core/)
- File issues at [GitHub Issues](https://github.com/jacksonzamorano/repack/issues)

## Contributing

Repack is open source and welcomes contributions:

1. **Bug Reports**: File detailed issues with reproduction steps
2. **Feature Requests**: Propose new features with use cases
3. **Blueprint Contributions**: Share useful blueprints for new targets
4. **Documentation**: Help improve examples and guides

### Building from Source

```bash
git clone https://github.com/jacksonzamorano/repack
cd repack
cargo build --release
cargo test
cargo install --path .
```

## License

Licensed under the MIT License. See [LICENSE.txt](LICENSE.txt) for details.

---

**Ready to streamline your development workflow?** Start with the [Quick Start Guide](#quick-start-guide) and explore the comprehensive examples in the [`test/`](test/) directory.