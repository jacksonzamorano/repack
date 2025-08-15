use std::{io::Write, path::PathBuf, process::exit};

use blueprint::BlueprintRenderer;
use syntax::{FileContents, ParseResult, RepackError, RepackErrorKind};

use crate::blueprint::BlueprintStore;

mod blueprint;
mod syntax;

const WIDTH: usize = 60;

pub struct Console;
impl Console {
    fn begin() {
        println!("[] Loading...");
        print!("");
    }
    fn update_ct(i: usize, n: usize, title: &str) {
        print!("\x1B[1A");
        print!("\r\x1B[2K[{i}/{n}] {title:<WIDTH$}\n");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
    fn update_msg(msg: &str) {
        print!("\r\x1B[2K  {msg:<WIDTH$}");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
    fn finalize() {
        println!()
    }
    fn error(message: &str) {
        print!("\n{message}");
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
    fn ask_confirmation() -> bool {
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            return false;
        }
        print!("\x1B[1A");
        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    }
}

/// Defines the operational mode for the repack code generator.
///
/// This enum determines what action the tool will take when executed.
/// The behavior is determined by command-line arguments passed to the application.
enum Behavior {
    /// Generate code files from the schema using blueprint templates.
    /// This is the default mode that creates output files in target languages
    /// like Rust, TypeScript, PostgreSQL, Go, and Markdown.
    Build,
    /// Remove previously generated code files, cleaning up the output directories.
    /// Uses blueprint metadata to determine which files to delete.
    Clean,
}

fn print_usage() {
    let msg = include_bytes!("usage.txt");
    _ = std::io::stdout().write_all(msg);
    exit(1);
}

/// Entry point for the repack code generation tool.
///
/// This function orchestrates the complete code generation process:
/// 1. Parses command-line arguments to determine operation mode and input file
/// 2. Loads and parses the .repack schema file with tokenization
/// 3. Loads built-in blueprints (rust, typescript, postgres, go, markdown)
/// 4. Loads any external blueprint files specified in the schema
/// 5. Filters and processes outputs based on blueprint types and categories
/// 6. Executes the requested operation (build, clean, document, or configure)
///
/// The tool supports four operation modes:
/// - `repack build file.repack` - Generate code files (default)
/// - `repack clean file.repack` - Remove generated files
/// - `repack document file.repack` - Generate documentation
/// - `repack configure env file.repack` - Generate configuration files
fn main() {
    Console::begin();
    let mut task_index = 1;
    let mut task_count = 1;
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage();
    }

    let (command, file) = match (args.get(1), args.get(2)) {
        (Some(file), None) => (Behavior::Build, file),
        (Some(arg), Some(file)) if arg == "build" => (Behavior::Build, file),
        (Some(arg), Some(file)) if arg == "clean" => (Behavior::Clean, file),
        _ => {
            print_usage();
            return;
        }
    };

    Console::update_ct(task_index, task_count, "Planning...");

    let contents = FileContents::new(file);
    let parse_result = match ParseResult::from_contents(contents) {
        Ok(res) => res,
        Err(e) => {
            for err in e {
                Console::error(&err.into_string());
            }
            exit(1);
        }
    };

    let mut store = match BlueprintStore::new() {
        Ok(res) => res,
        Err(e) => {
            println!("{}", e.into_string());
            exit(1);
        }
    };
    for add in &parse_result.include_blueprints {
        let mut path = PathBuf::from(&file);
        path.pop();
        path.push(add);
        if store.load_file(&path).is_err() {
            let path_str = path.to_str().unwrap_or("<invalid path>");
            Console::error(
                &RepackError::global(RepackErrorKind::CannotRead, path_str.to_string())
                    .into_string(),
            );
            exit(1);
        }
    }

    let outputs = parse_result
        .languages
        .iter()
        .map(|lng| {
            let Some(bp) = store.blueprint(&lng.profile) else {
                Console::error(&format!(
                    "[{}] Could not find this blueprint. Have you imported it?",
                    lng.profile
                ));
                exit(2)
            };
            match command {
                Behavior::Build => ("Building", lng, bp),
                Behavior::Clean => ("Cleaning", lng, bp),
            }
        })
        .collect::<Vec<_>>();
    task_count += outputs.len();

    for (task_string, output, bp) in outputs {
        task_index += 1;
        Console::update_ct(
            task_index,
            task_count,
            &format!("{} {}...", task_string, bp.name),
        );
        let mut builder = BlueprintRenderer::new(&parse_result, bp, output);
        match command {
            Behavior::Build => match builder.build(None) {
                Ok(_) => {}
                Err(e) => {
                    Console::error(&e.into_string());
                }
            },
            Behavior::Clean => match builder.clean() {
                Ok(_) => {}
                Err(e) => {
                    Console::error(&e.into_string());
                }
            },
        }
    }
    Console::update_ct(task_index, task_count, "⚡️ Completed");
    Console::update_msg("Project built.");
    Console::finalize();
}
