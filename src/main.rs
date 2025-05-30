use std::process::exit;

use outputs::OutputDescription;
use profiles::OutputProfile;
use syntax::{FileContents, ParseResult};

mod outputs;
mod profiles;
mod syntax;

enum Behavior {
    Build,
    Clean,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        exit(1);
    }

    let mut behavior = Behavior::Build;

    if args.contains(&"--clean".to_string()) {
        behavior = Behavior::Clean;
    }

    let input_file = &args[1];
    let mut contents = FileContents::new();
    contents.read(input_file);
    let parse_result = ParseResult::from_contents(contents).unwrap();
    parse_result.validate(true);

    match behavior {
        Behavior::Build => {
            for output in &parse_result.languages {
                match OutputDescription::new(&parse_result, output)
                    .and_then(|mut desc| {
                        let profile = OutputProfile::from_keyword(&output.profile).unwrap();
                        let builder = profile.builder();
                        builder.build(&mut desc).map(|_| desc)
                    })
                    .and_then(|mut desc| desc.flush())
                {
                    Ok(_) => {
                        println!("[{}] Built successfully!", output.profile);
                    }
                    Err(e) => {
                        println!("[{}] Failed to build: {}", output.profile, e.into_string());
                    }
                }
            }
        }
        Behavior::Clean => {
            for output in &parse_result.languages {
                match OutputDescription::new(&parse_result, output)
                    .and_then(|mut desc| {
                        let profile = OutputProfile::from_keyword(&output.profile).unwrap();
                        let builder = profile.builder();
                        builder.build(&mut desc).map(|_| desc)
                    })
                    .and_then(|mut desc| desc.clean())
                {
                    Ok(_) => {
                        println!("[{}] Built successfully!", output.profile);
                    }
                    Err(e) => {
                        println!("[{}] Failed to build: {}", output.profile, e.into_string());
                    }
                }
            }
        }
    }
}
