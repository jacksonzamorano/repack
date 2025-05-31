# REPACK

`repack` is a tool to define data models and generate corresponding code in various languages and formats. You define your models in `.repack` files using a simple syntax.

* Note: Repack is not an ORM. Repack is a codegen tool. *

* Another note: documentation below is AI generated and has only been lightly reviewed. While this will change in the near future, to learn more now, check out the example files in test/test.repack *

## Key Concepts

*   **Snippets:** Reusable blocks of fields. Include them in records using `!snippet_name`.
    Example: `base` snippet for `id` and `created_date`.
*   **Records:** Define your data models (e.g., `User`, `Organization`). Similar to classes or tables.
*   **Structs:** Define data structures not necessarily database models but useful for your application (e.g., `OrganizationDirectory`).
*   **Outputs:** Specify desired output formats and locations. Currently supported outputs (more to come):
    *   `description`: Text description of models.
    *   `postgres`: SQL for PostgreSQL.
    *   `typescript_class`: TypeScript classes.
    *   `typescript_interface`: TypeScript interfaces.
    *   `typescript_drizzle`: TypeScript code for Drizzle ORM.
    *   `rust`: Rust structs.
*   **Tags:** Categorize models using `#tag_name` (e.g., `#models`, `#data`, `#private`). Used by output generators.
*   **Fields:** Define fields with types (e.g., `name string`, `id int32`).
*   **Functions** Add functions like `db:primary_key`, `db:default("NOW()")`, `db:index("org_id")`.
*   **References:** Define relationships using `ref(OtherRecord.field)`.
*   **Joins** Define views and join data using `from(local_ref_field.field)`.
*   **Inheritance/Composition:** Create record variations by including all fields (`*`) and adding/removing specific fields (`- field_name`).
*   **Imports:** Import definitions from other `.repack` files using `import "filename.repack"`.

## How to Create Your Own Models

1.  **Create a `.repack` file:** This is where you'll define your models.
2.  **Define Snippets (Optional but Recommended):**
    ```repack
    snippet base {
        id int32 db:primary_key db:identity db:unique
        created_date datetime db:default("NOW()")
    }
    ```
3.  **Define Records:**
    ```repack
    record User @users #private #models {
        !base // Includes id and created_date
        name string
        email string
        // ... other fields
    }

    record Post @posts #models {
        !base
        title string
        content string
        author_id ref(User.id) // Foreign key to User table
        db:index("author_id")
    }
    ```
4.  **Define Structs (Optional):**
    ```repack
    struct UserProfile #data {
        user User
        posts Post[]
    }
    ```
5.  **Specify Outputs:**
    ```repack
    // Output all records tagged with #models to a PostgreSQL schema file
    output postgres @./output/postgres_schema #models;

    // Output all records and structs tagged with #data as TypeScript interfaces
    output typescript_interface @./output/ts_interfaces #data {
        make_index true // Option to create an index.ts file
    }

    // Output Rust structs for items tagged #data
    output rust @./output/rust_structs #data;
    ```
6.  **Run the `repack` CLI:**
    ```bash
    repack your_definitions.repack
    ```
    This will process your definitions and generate files in the specified output directories.


By combining these features, you can create a rich and well-structured definition of your data model that can then be translated into various code artifacts, saving time and ensuring consistency.
