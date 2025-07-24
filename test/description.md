# Repack Test Schema Documentation

This document provides a comprehensive overview of the test schema used to demonstrate Repack's capabilities.

## Generated from Schema

**Source**: [`test.repack`](test.repack)  
**Blueprint**: [`output_config.blueprint`](output_config.blueprint)

## Overview

The test schema demonstrates a user management system with contact information, showcasing:

- **Configuration Management**: Environment-specific deployment settings
- **Advanced Relationships**: Implicit and explicit joins between records
- **Multiple Object Types**: Records, synthetic views, and structs
- **Database Functions**: Indexes, primary keys, and default values
- **Code Generation**: Output to multiple target languages

## Configuration System

### ApiDeployment Configuration

Defines deployment configuration template with the following parameters:

- **`host_ip`**: Server IP address
- **`db_username`**: Database username
- **`db_password`**: Database password

### Environment Instances

#### Production (`@prod`)
- **Host IP**: `192.168.0.1`
- **Username**: `admin`
- **Password**: `test`

#### Staging (`@staging`)
- **Host IP**: `10.0.0.1`
- **Username**: `admin2`
- **Password**: `test2`

## Data Models

### Enums

#### UserType
Represents different user privilege levels:

- **`Admin`**: Full system access
- **`User`**: Standard user privileges
- **`Guest`**: Limited access

### Records (Database Tables)

#### User (`users` table)

**Purpose**: Core user entity for authentication and profile management

**Fields**:
- **`id`**: UUID (Primary Key)
- **`created_date`**: Timestamp (Default: `NOW()`)
- **`last_login`**: Optional timestamp for tracking user activity
- **`name`**: User's display name
- **`user_type`**: Role/privilege level (UserType enum)
- **`subscription_id`**: Optional subscription identifier

**Database Features**:
- Primary key on `id`
- Automatic timestamp on creation
- Supports optional fields for flexible user data

#### ContactInfo (`contacts` table)

**Purpose**: User contact information with foreign key relationship

**Fields**:
- **`id`**: UUID (Primary Key)
- **`created_date`**: Timestamp (Default: `NOW()`)
- **`email`**: User's email address
- **`user_id`**: Foreign key reference to User.id

**Database Features**:
- **Indexes**:
  - Single index on `email` for fast lookups
  - Single index on `user_id` for join optimization
  - Composite index on `user_id, id` for complex queries
- Foreign key relationship to User table

### Synthetic Views

#### FullUser

**Purpose**: Combined view of User and ContactInfo data using joins

**Base**: Inherits from `ContactInfo` record  
**Additional Fields**:
- **`name`**: Retrieved from related User via `from(user_id.name)` implicit join

**Features**:
- **Automatic Join**: Uses implicit join `j_user_id: self.user_id = users.id`
- **Data Combination**: Provides unified access to user and contact data
- **Database View**: Can be materialized as a database view
- **Type Safety**: Maintains type safety across joined fields

### Structs (In-Memory Data)

#### UserList

**Purpose**: API response structure for user collections

**Fields**:
- **`users`**: Array of User objects

**Features**:
- **Array Support**: Demonstrates struct-only array capabilities
- **API Response**: Designed for JSON serialization
- **Memory Only**: Not stored in database

## Relationships and Joins

### Implicit Join System

The schema demonstrates Repack's automatic join resolution:

```repack
synthetic FullUser: ContactInfo #model {
    *                           // Include all ContactInfo fields
    name from(user_id.name)     // Automatic join via foreign key
}
```

**Generated Join**: `j_user_id: self.user_id = users.id`

This creates an automatic relationship that:
- Links ContactInfo to User via the `user_id` foreign key
- Allows accessing User fields from the FullUser synthetic view
- Generates appropriate SQL JOINs in database queries
- Maintains type safety across the relationship

### Database Indexes

Strategic indexing for performance:

1. **Email Lookup**: Single index on `ContactInfo.email`
2. **User Association**: Single index on `ContactInfo.user_id`
3. **Complex Queries**: Composite index on `ContactInfo(user_id, id)`

## Generated Output

### Target Languages

The schema generates code for multiple targets:

- **PostgreSQL** (`test/postgres/`): Database schema with tables, indexes, and constraints
- **TypeScript** (`test/typescript/`): Interfaces and enums for frontend development
- **Rust** (`test/rust/`): Structs and enums for backend services
- **Go** (`test/go/`): Structs with JSON tags for microservices
- **Markdown** (`test/`): This documentation file

### File Structure

```
test/
├── test.repack              # Source schema
├── output_config.blueprint  # Custom blueprint
├── description.md          # This documentation
├── deployment.md           # Environment configuration
├── postgres/
│   └── model.sql           # Database DDL
├── typescript/
│   ├── ContactInfo.ts      # ContactInfo interface
│   ├── FullUser.ts         # FullUser interface
│   ├── User.ts             # User interface
│   ├── UserList.ts         # UserList interface
│   ├── UserType.ts         # UserType enum
│   └── index.ts            # Barrel exports
├── rust/
│   └── model.rs            # Rust structs and enums
└── go/
    ├── go.mod              # Go module
    ├── go.sum              # Dependencies
    ├── mod.go              # Module definition
    └── model.go            # Go structs
```

## Usage Examples

### Database Queries

**Find user with contact info**:
```sql
SELECT u.name, u.user_type, c.email
FROM users u
JOIN contacts c ON u.id = c.user_id
WHERE c.email = 'user@example.com';
```

**Use generated view**:
```sql
SELECT * FROM FullUser
WHERE email = 'user@example.com';
```

### API Responses

**TypeScript frontend**:
```typescript
import { UserList, FullUser } from './types';

const response: UserList = {
    users: [
        {
            id: "123e4567-e89b-12d3-a456-426614174000",
            name: "John Doe",
            email: "john@example.com",
            userType: UserType.User
        }
    ]
};
```

**Rust backend**:
```rust
use crate::models::{UserList, FullUser, UserType};

let user_list = UserList {
    users: vec![
        FullUser {
            id: Uuid::new_v4(),
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            user_type: UserType::User,
        }
    ]
};
```

## Schema Evolution

This test schema demonstrates how to evolve data models:

1. **Start Simple**: Basic User record with essential fields
2. **Add Relationships**: ContactInfo with foreign key to User
3. **Create Views**: FullUser synthetic for combined data access
4. **Add Configurations**: Environment-specific deployment settings
5. **Optimize Performance**: Database indexes for common queries

## Best Practices Demonstrated

1. **Separation of Concerns**: User identity separate from contact information
2. **Foreign Key Relationships**: Proper referential integrity
3. **Performance Optimization**: Strategic database indexing
4. **Type Safety**: Consistent types across all generated languages
5. **Configuration Management**: Environment-specific settings
6. **Documentation**: Comprehensive schema documentation

This test schema serves as a reference implementation for building production-ready systems with Repack.