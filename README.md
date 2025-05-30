# Repack: Code Generation Tool

*Documentation is currently AI-generated, use at your own risk.*

**Overview:**

Repack is a command-line tool designed to streamline development by allowing you to define your data models once in a `.repack` file and then generate corresponding code in various languages and formats. This helps maintain consistency across different parts of your application (e.g., backend, frontend, database).

**Core Concepts:**

*   **`.repack` Definition File:** This is the central file where you define your data structures (records and structs) and specify the desired outputs.
*   **Records (`record`):** Represent data entities that typically map to database tables or persistent data structures. They can have table names, inherit from other records, and include fields with various types and constraints.
*   **Structs (`struct`):** Represent data structures that are not necessarily tied to database persistence. They are simpler than records and cannot inherit or have table names.
*   **Outputs (`output`):** Define the target languages/formats for code generation. Each output specifies a profile (e.g., `postgres`, `typescript_class`, `rust`), an output location (directory), and can be filtered by categories.
*   **Fields:** Define the properties of your records and structs. Fields have a name, a type (e.g., `string`, `int32`, `ref(OtherRecord.id)`), and can have commands like `#pk` (primary key), `#increment`, etc.
*   **Categories (`#category_name`):** Allow you to group records and structs. Outputs can then be configured to only include objects belonging to specific categories.
*   **Inheritance (`:`):** Records can inherit fields from a parent record.
*   **Field Reuse (`*`, `- field_name`):** When inheriting, you can include all fields from the parent using `*` and exclude specific fields using `- field_name`.
*   **References (`ref(OtherRecord.id)`):** Define relationships between records, typically for foreign keys.
*   **Computed Fields (`from(other_field.name)`):** Define fields whose values are derived from other fields, often in related records.

**Command-Line Usage:**

The primary way to use `repack` is through the command line:

```bash
repack <input_file.repack> [options]
```

*   **`<input_file.repack>` (Required):** The path to your `.repack` definition file.
*   **Options:**
    *   `--clean`: This option will remove the previously generated files for the specified outputs before generating new ones.

**Key Tasks and Functionality:**

1.  **Parsing `.repack` Files:**
    *   The tool reads the `.repack` file, tokenizes its content, and parses it into an internal representation of objects (records and structs) and output configurations.
    *   It handles various syntax elements like object types (`record`, `struct`), output definitions (`output`), field definitions (name, type, commands), inheritance, categories, table names (`@table_name`), and output-specific options.

2.  **Dependency Resolution and Validation:**
    *   **Object Dependencies:** It analyzes dependencies between objects (e.g., due to inheritance or field references) and reorders them to ensure that dependencies are processed before the objects that depend on them.
    *   **Circular Dependency Check:** Validates that there are no circular dependencies between object definitions, which would lead to an unresolvable generation order.
    *   **Type Resolution:** Resolves field types, including references to fields in other objects (`ref` and `from` clauses).
    *   **Error Handling:** Performs extensive validation and reports errors if the `.repack` file contains invalid syntax, unresolved references, or unsupported configurations. Errors include specific codes and messages indicating the location (object and field) of the issue.

3.  **Code Generation:**
    *   For each `output` defined in the `.repack` file, the tool:
        *   Filters the parsed objects based on the categories specified in the `output` definition.
        *   Selects the appropriate output profile (e.g., `postgres`, `typescript_class`, `rust_vanilla`).
        *   Uses a corresponding "builder" for that profile to generate the code.
        *   Writes the generated code to the specified output location.
    *   **Supported Output Profiles (based on the provided code):**
        *   **`description`:** Generates a text file (`description.txt`) summarizing the defined objects and their fields.
            *   Options: `print_commands` (boolean, defaults to `true`) - whether to include field commands in the description.
        *   **`postgres`:** Generates SQL statements (`model.sql`) to create database tables for `record` definitions.
            *   Handles primary keys, nullability, and foreign key constraints (including `ON DELETE CASCADE` if specified with `#cascade`).
            *   Does not support inheritance for Postgres output.
        *   **`typescript_class`:** Generates TypeScript class definitions (`.ts` files) for objects.
            *   Creates individual files for each object and an optional `index.ts` file to export all generated classes.
            *   Handles field types, optionality (`?`), and arrays (`[]` for `#many` command).
            *   Generates import statements for custom types (other generated classes).
            *   Options: `make_index` (boolean, defaults to `false`) - whether to create an `index.ts` file.
        *   **`typescript_interface`:** Generates TypeScript interface definitions (`.ts` files).
            *   Similar to `typescript_class` but generates interfaces instead of classes.
            *   Uses `import type` for dependencies.
            *   Options: `make_index` (boolean, defaults to `false`).
        *   **`rust` (specifically `rust_vanilla`):** Generates Rust struct definitions (`model.rs`).
            *   Handles field types, `Option<>` for optional fields, and `Vec<>` for fields with the `#many` command.
            *   Automatically adds `use chrono::NaiveDateTime;` if `DateTime` fields are used.

4.  **File System Operations:**
    *   Creates output directories if they don't exist.
    *   Writes generated files to the appropriate locations.
    *   Handles cleaning of output directories when the `--clean` flag is used (removes files generated by previous runs for the relevant output profiles).

**Example `.repack` File Structure (from `test.repack`):**

```repack
// Define outputs
output description @test { // Output human-readable description to 'test' directory
	print_commands true
}

output postgres @test/postgres #models; // Output PostgreSQL schema to 'test/postgres' for objects in '#models' category
output typescript_class @test/ts_classes #frontend #orm { // Output TS classes to 'test/ts_classes' for '#frontend' or '#orm'
	make_index true
}
output typescript_interface @test/ts_interfaces #frontend { // Output TS interfaces to 'test/ts_interfaces' for '#frontend'
	make_index true
}
output rust @test/rust_vanilla #frontend; // Output Rust structs to 'test/rust_vanilla' for '#frontend'

// Define records (typically map to database tables)
record User @users #models #frontend { // 'User' record, in 'users' table, part of '#models' and '#frontend' categories
	id int32 #pk #increment       // Integer ID, primary key, auto-incrementing
	name string
	email string
	password string
	org_id ref(Organization.id)   // Foreign key to Organization.id
	personal_org_id ref(Organization.id)
}

record Organization @orgs #models { // 'Organization' record, in 'orgs' table, part of '#models' category
	id int32 #pk #increment
	name string
	email string
}

record UserPublic: User #orm { // 'UserPublic' inherits from 'User', part of '#orm' category
	*                             // Include all fields from User
	org_name from(org_id.name)    // Add 'org_name' field, derived from the 'name' of the related Organization
	- personal_org_id             // Exclude 'personal_org_id' from User
}

record UserPublicNoOrg: UserPublic #orm { // 'UserPublicNoOrg' inherits from 'UserPublic'
	*
	- org_name                    // Exclude 'org_name'
	- org_id                      // Exclude 'org_id'
}

// Define structs (general data structures)
struct OrgModel #frontend { // 'OrgModel' struct, part of '#frontend' category
	user User                   // Field 'user' of type 'User' (references the User record)
}

