use std::{io::Write, path::PathBuf, process::exit};

use blueprint::BlueprintRenderer;
// use blueprint::BlueprintBuilder;
use syntax::{FileContents, ParseResult};

use crate::blueprint::BlueprintStore;

mod blueprint;
// mod outputs;
mod syntax;

/// Defines the operational mode for the repack code generator.
/// 
/// This enum determines what action the tool will take when executed.
/// The behavior is determined by command-line flags passed to the application.
enum Behavior {
    /// Generate code files from the schema using the configured blueprints.
    /// This is the default mode that creates output files in target languages.
    Build,
    /// Remove previously generated code files, cleaning up the output directories.
    /// Useful for starting fresh or removing outdated generated code.
    Clean,
}

/// Entry point for the repack code generation tool.
/// 
/// This function orchestrates the complete code generation process:
/// 1. Parses command-line arguments to determine input file and behavior
/// 2. Loads and parses the schema file with all its dependencies
/// 3. Loads required blueprints (both core and external)
/// 4. Generates code for each configured output target
/// 5. Handles both build and clean operations based on flags
/// 
/// The tool expects at least one argument (the input schema file) and supports
/// the --clean flag to remove previously generated files instead of building.
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        let msg = include_bytes!("usage.txt");
        _ = std::io::stdout().write_all(msg);
        exit(1);
    }

    let msg = include_bytes!("welcome.txt");
    _ = std::io::stdout().write_all(msg);

    let mut behavior = Behavior::Build;

    if args.contains(&"--clean".to_string()) {
        behavior = Behavior::Clean;
    }

    let input_file = &args[1];
    let contents = FileContents::new(input_file);
    let parse_result = match ParseResult::from_contents(contents) {
        Ok(res) => res,
        Err(e) => {
            for err in e {
                println!("{}", err.into_string());
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
        let mut path = PathBuf::from(&input_file);
        path.pop();
        path.push(add);
        match store.load_file(&path) {
            Err(_) => {
                panic!(
                    "Could not load external library '{}'",
                    path.to_str().unwrap()
                );
            }
            _ => {
                println!("[BLUEPRINT] Loaded '{}'", path.to_str().unwrap());
            }
        }
    }

    match behavior {
        Behavior::Build => {
            for output in &parse_result.languages {
                let Some(bp) = store.blueprint(&output.profile) else {
                    println!(
                        "[{}] Could not find this blueprint. Have you imported it?",
                        output.profile
                    );
                    continue;
                };
                let mut builder = BlueprintRenderer::new(&parse_result, bp, output);
                match builder.build() {
                    Ok(_) => {
                        println!("[{}] Built successfully!", output.profile);
                    }
                    Err(e) => {
                        println!("{}", e.into_string());
                    }
                }
            }
            let msg = include_bytes!("done.txt");
            _ = std::io::stdout().write_all(msg);
        }
        Behavior::Clean => {
            for output in &parse_result.languages {
                let Some(bp) = store.blueprint(&output.profile) else {
                    println!(
                        "[{}] Could not find this blueprint. Have you imported it?",
                        output.profile
                    );
                    continue;
                };
                let mut builder = BlueprintRenderer::new(&parse_result, bp, output);
                match builder.clean() {
                    Ok(_) => {
                        println!("[{}] Built successfully!", output.profile);
                    }
                    Err(e) => {
                        println!("{}", e.into_string());
                    }
                }
            }
        }
    }
    exit(1);
}
