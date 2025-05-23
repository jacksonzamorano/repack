use outputs::{OutputBuilder, OutputDescription};
use profiles::OutputProfile;
use syntax::FileContents;

mod outputs;
mod profiles;
mod syntax;

fn main() {
    let mut contents = FileContents::new();
    contents.read("test.repack");
    let parse_result = syntax::ParseResult::from_contents(contents);
    parse_result.validate(false);

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
