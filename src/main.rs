use std::process::exit;

use outputs::OutputDescription;
use profiles::OutputProfile;
use syntax::FileContents;

mod outputs;
mod profiles;
mod syntax;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        exit(1);
    }
    let input_file = &args[1];
    let mut contents = FileContents::new();
    contents.read(input_file);
    let parse_result = syntax::ParseResult::from_contents(contents).unwrap();
    parse_result.validate(true);

    for output in &parse_result.languages {
        let mut description = OutputDescription::new(&parse_result, output);
        let profile = OutputProfile::from_keyword(&output.profile).unwrap();
        let builder = profile.builder();
        match builder.build(&mut description) {
            Ok(_) => {
                if let Err(e) = description.flush() {
                    println!("[{}] Failed to build: {}", output.profile, e.description());
                } else {
                    println!("[{}] Built successfully!", output.profile);
                }
            }
            Err(e) => {
                println!("[{}] Failed to build: {}", output.profile, e.description());
            }
        };
    }
}
