use std::{io::Write, path::PathBuf, process::exit};

use blueprint::BlueprintRenderer;
// use blueprint::BlueprintBuilder;
use syntax::{FileContents, ParseResult};

use crate::blueprint::BlueprintStore;

mod blueprint;
// mod outputs;
mod syntax;

enum Behavior {
    Build,
    Clean,
}

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
