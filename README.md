# Repack Language & Blueprint Specification

Authoritative specification for the Repack schema language ("Repack") and the Blueprint templating language. This document omits roadmap, marketing, and non-speculative content.

## 1. Repack Schema Language

### 1.1 Lexical Elements
Identifiers: /[A-Za-z_][A-Za-z0-9_]*/
Strings: double-quoted sequences preserving inner content.
Comments: // to end-of-line.
Whitespace: insignificant except inside string literals.

### 1.2 Keywords
Reserved (recognized as tokens):
- output
- struct
- enum
- snippet
- import
- blueprint
- query
- insert
- update
- one
- many
- join

Deprecated (tokenized but not part of public spec): where, with, except.

### 1.3 Primitive Types
string, int64, int32, float64, boolean, datetime, uuid, bytes

### 1.4 Type Shapes
Exactly four canonical shapes:
- T (required scalar)
- T? (optional scalar)
- T[] (required array)
- T[]? (optional array)  // Array marker precedes optional marker.

### 1.5 Field Forms
fieldName TypeShape
fieldName External.Location( )?  (See 1.9) where External.Location can itself be array/optional with suffixes.

### 1.6 Object Kinds
struct Name [: Parent]? [@table]? [#category]* { ... }
- Inheritance: single parent; table name inherited automatically.
- Categories: arbitrary tags used for output filtering.
- @table: associates table name with struct for query/table output.

### 1.7 Enums
enum Name [#category]* { CaseA CaseB ... }
Enum cases optionally may have explicit string value by writing a second literal on the same line: CaseA "custom".

### 1.8 Snippets
snippet Name { fields / functions }
Included inside struct body with: !snippetName (single token after '!'). Fields/functions merged in-place prior to dependency resolution.

### 1.9 External Field References
Syntax: fieldName OtherStruct.otherField
Special location name "super" indicates the parent struct when using inheritance.
During resolution the referenced field's resolved type is copied.

### 1.10 Functions (Field / Struct)
Syntax inside a struct body or after a field definition:
namespace:funcName(arg1, arg2)
Parentheses optional if zero args.
Parsed as Literal(namespace) Colon Literal(funcName) [OpenParen args CloseParen]. Stored with namespace, name, arg list.

### 1.11 Joins
join(name ForeignStruct) = predicateTemplate
- name: local join alias
- ForeignStruct: referenced struct name
- predicateTemplate: literal captured token sequence (single literal token) possibly containing interpolation variables:
  - $name => "<foreign_table> <joinAlias>"
  - $super => parent table name (when inheritance used)
  - $joinAlias => emits joinAlias
Used to build $locations expansion (see 1.13). Query builder expands joins into location clause segments.

### 1.12 Queries
Queries attach to a struct. Three forms:
1. Manual:
   query Name(argName type ...)= SQLLiteral [: one|many]?
2. Auto Insert:
   insert Name(field1, field2, ...) [: one|many]?  // Generates WITH $table AS (INSERT ... RETURNING *) SELECT $fields FROM $locations
3. Auto Update:
   update Name(argName type ...)= Fragment [: one|many]? // Fragment inserted after UPDATE $table; becomes WITH $table AS (UPDATE $table Fragment RETURNING *) SELECT $fields FROM $locations
Return annotation semantics:
- omitted => returns_none
- : one => returns_one
- : many => returns_many
Arguments create positional parameters during interpolation.

### 1.13 Query Interpolation Variables
Within query contents (after parsing, before emission):
- $fields: comma-separated "<table>.<col> AS <alias>" entries for all fields, respecting external locations and db:as alias functions.
- $locations: base table plus each join expanded from join predicates.
- $table: the struct's table name.
- $fieldName / $#fieldName: field reference; isolated variant ($#) emits only column, non-isolated emits qualified form (table.column) considering join/super mapping.
- $argName: converted to positional $1,$2,... in order of first appearance.
Unknown interpolation yields [err: name].

### 1.14 Inheritance
struct Child: Parent { ... }
- Parent must precede or be resolvable; its @table propagates to child.
- External reference syntax super.field references parent's field.

### 1.15 Validation & Resolution Steps
1. Parse declarations.
2. Expand snippet inclusions.
3. Reorder structs to satisfy dependency ordering.
4. Resolve inheritance table propagation.
5. Resolve external field references (super or join alias) & copy types.
6. Resolve custom field types (structs, enums).
7. Generate auto queries (insert/update) into struct query list.
8. Accumulate errors (duplicate fields, unresolved types, invalid references, etc.).

### 1.16 Output Configuration
output blueprintId @location [#category]* [!excludeName]* [key=value]*;
- profile (blueprintId) must match loaded blueprint id.
- Categories filter included structs/enums (logical OR). Empty categories => include all.
- Exclusions remove named structs/enums.
- key=value options become initial variables in blueprint context.

### 1.17 Imports / Blueprints
import "path/or/pattern*"  (relative to current file root; * loads all .repack files in folder)
blueprint path/to/file.blueprint (queued for later loading)

### 1.18 Error Codes
Error kinds (E#### codes map to enum order):
- CircularDependancy
- ParentObjectDoesNotExist
- CustomTypeNotDefined
- TypeNotResolved
- SnippetNotFound
- DuplicateFieldNames
- CannotCreateContext
- FunctionInvalidSyntax
- TypeNotSupported
- CannotRead
- CannotWrite
- SnippetNotClosed
- UnknownSnippet
- VariableNotInScope
- InvalidVariableModifier
- UnknownLink
- UnknownObject
- QueryArgInvalidSyntax
- QueryInvalidSyntax
- InvalidSuper
- FieldNotOnSuper
- InvalidJoin
- FieldNotOnJoin
- SyntaxError
- ProcessExecutionFailed (NEW: Process execution failed during blueprint rendering)
- PathNotValid (NEW: Path could not be converted to string representation) 
- ParseIncomplete (NEW: Parsing failed due to missing expected tokens)
- FieldNotFound (NEW: Field could not be found in struct during query processing)
- UnknownError

## 2. Blueprint Templating Language
Blueprints define target generation via bracketed directives.

### 2.1 Token Forms
General block: [main secondary optional_inline_contents] ... [/main]
Auto-close tokens (no closing tag): variable, imports, import, increment, br.
Escaping: prefix '[' with '\\' to treat literally.

### 2.2 Main Tokens
meta, file, if, ifn, each, eachr, define, func, nfunc, join, ref, link, import, trim, imports, br, exec, increment, snippet, render, variable (any unrecognized main token treated as variable reference), plus internal typedef (define) and TypeDef handling.
(Note: join/ref main tokens are currently parsed; rendering branch for join iteration not implemented intentionally.)

### 2.3 Secondary Tokens
id, name, kind, struct, field, enum, case, debug, arg, query, join, and any primitive type names (string,int32,int64,float64,boolean,datetime,uuid,bytes). Others become Arbitrary.

### 2.4 Iteration
[each struct] ... [/each]
[each field] ... [/each] (requires struct context)
[each enum] ... [/each]
[each case] ... [/each] (requires enum context)
[each query] ... [/each] (requires struct context)
[each arg] ... [/each] (inside func or query arg context)
Reverse order: eachr (same secondary semantics).
Each iteration sets flag sep = true on all but last element (useful for commas).

### 2.5 Conditionals
[if flag]...[/if] executes if context.flags[flag]==true.
[ifn flag] inverse.
Flags defined:
- queries (struct has queries)
- optional, array (field modifiers)
- returns_many, returns_one, returns_none (query return type)
- has_args (function invocation context)
- sep (iteration separator helper)

### 2.6 Variables
[name], [table_name], [struct_name], [type], [type_raw], [enum_name], [value], [query], numeric indices for function args ([0], [1], ...), [arg] inside arg iteration.
Transform modifiers via dotted suffix: uppercase, lowercase, titlecase, camelcase, split_period_first, split_period_last, split_dash_first, split_dash_last.

### 2.7 Imports
[link key]content[/link] defines link mapping.
[import key] places import (deduped per file).
[imports] designates insertion point for all collected imports for current file (emitted with surrounding blank lines; each import on its own line).

### 2.8 File Selection
[file]filename[/file] or [file] sets currently active output file (filename may be built from nested variables rendered inside block contents if inline contents empty and body present).

### 2.9 Snippets & Render
[snippet name] ... [/snippet] defines reusable inline snippet (captured literal content only, no nested variable expansion at definition time; expanded at [render]).
[render]snippetName[/render] inserts snippet literal.

### 2.10 Functions
[func ns.name] body [/func] executes body once per matching struct or field function with that namespace/name. Inside body: [each arg] iterates argument strings; numeric variables 0..N and flag has_args available.
[nfunc ns.name] executes body if no matching function exists in current field/struct context.

### 2.11 Query Context
Within [each query] iteration, [query] variable contains fully rendered SQL (with interpolation expansion and trailing semicolon); flags returns_many / returns_one / returns_none set; [each arg] yields query args in order.

### 2.12 Trimming / Breaks / Increment
[trim]content[/trim] deletes matching trailing content from most recent write (used to remove last commas / separators).
[br] inserts newline.
[increment counterName] increments named global counter; referencing [counterName] outputs numeric value.

### 2.13 Exec
[exec]shell script here[/exec] prompts user (y/N) before executing script via sh -s. Output suppressed except errors.

### 2.14 Variable Resolution Errors
Unknown variable => VariableNotInScope error.
Unknown modifier => InvalidVariableModifier.
Unknown link => UnknownLink.
Unknown snippet at render => UnknownSnippet.

### 2.15 Type Definitions
[define primitive]TargetType[/define] maps primitive to blueprint-local representation (used during field/query arg rendering).
[link primitive] may supply import template (with $ placeholder replaced by primitive name).
Custom (non-primitive) types use [link custom] pattern with $ substituted for type name if present.

## 3. Grammar (EBNF Summary)

Schema := { Declaration }
Declaration := StructDecl | EnumDecl | SnippetDecl | OutputDecl | ImportDecl | BlueprintDecl
StructDecl := 'struct' Ident [':' Ident] [TableOpt] { Category } '{' StructBody '}'
EnumDecl := 'enum' Ident { Category } '{' EnumBody '}'
SnippetDecl := 'snippet' Ident '{' SnippetBody '}'
OutputDecl := 'output' Ident [LocationOpt] { Category } { Exclusion } { Option } ';'
ImportDecl := 'import' StringLiteral
BlueprintDecl := 'blueprint' StringLiteral
TableOpt := '@' Ident
Category := '#' Ident
Exclusion := '!' Ident
LocationOpt := '@' PathLike
Option := Ident '=' Ident
StructBody := { StructItem }
StructItem := FieldDecl | FunctionDecl | SnippetUse | QueryDecl | AutoInsertDecl | AutoUpdateDecl | JoinDecl
FieldDecl := Ident FieldTypeSpec FieldFunctions NewLine
FieldTypeSpec := (Ident ['.' Ident]) ArrayOpt OptOpt
ArrayOpt := '[]'?  // Represented as '[' ']' tokens.
OptOpt := '?'?
SnippetUse := '!' Ident
FunctionDecl := Namespace ':' Ident ['(' ArgList ')']
FieldFunctions := { FunctionDecl }
ArgList := Ident { ',' Ident }
QueryDecl := 'query' Ident ['(' QueryArgList ')'] '=' StringLiteral [ReturnType]
QueryArgList := QueryArg { QueryArg }
QueryArg := Ident Ident
AutoInsertDecl := 'insert' Ident ['(' Ident { ',' Ident } ')'] [ReturnType]
AutoUpdateDecl := 'update' Ident ['(' QueryArgList ')'] '=' StringLiteral [ReturnType]
JoinDecl := 'join' '(' Ident Ident ')' '=' StringLiteral
ReturnType := ':' ('one' | 'many')
EnumBody := { EnumCase }
EnumCase := Ident [Ident]
SnippetBody := { FieldDecl | FunctionDecl }

## 4. Implementation Notes
- Tokenization treats strings as single Literal tokens (contents without quotes).
- Query / update fragments store contents as single literal; update fragment internally rewrites '$' to '$#' to force isolated interpolation for user-provided fragments.
- Snippet expansion occurs before dependency ordering and type resolution.
- Auto insert/update queries transformed into full manual queries prior to query rendering contexts.
- Joins influence only $locations string; no direct iteration directive currently used in core blueprints.

## 5. Differences vs Internal Tokens
Internal tokens where/with/except exist for historical reasons but are not part of public syntax surface and should not be used.

## 6. Error Handling Contract
Parsing stops only after full pass; aggregated errors returned. Each error formatted: [E####] (profile -> context) message details stack. Blueprint snippet nesting adds contextual stack lines.

## 7. Security Considerations
[exec] execution requires explicit interactive confirmation to mitigate unintended script execution.

## 8. Determinism
Blueprint rendering order follows token sequence; import collection is per-file and order-insensitive (HashSet) but emitted unsorted; consumers should not rely on ordering of imports.

---
End of specification.
