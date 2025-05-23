use outputs::{OutputBuilder, OutputDescription};
use profiles::OutputProfile;
use syntax::FileContents;

mod profiles;
mod syntax;
mod outputs;

fn main() {
    let mut contents = FileContents::new();
    contents.read("test.repack");
    let parse_result = syntax::ParseResult::from_contents(contents);
    parse_result.validate(false);

    for output in &parse_result.languages {
        let mut description = OutputDescription::new(&parse_result, output);
        let profile = OutputProfile::from_keyword(&output.profile).unwrap();
        let builder = profile.builder();
        match builder.build(output, &mut description) {
            Ok(_) => {
                println!("[{}] Built successfully!", output.profile);
            }
            Err(e) => {
                eprintln!("[{}] Failed to build: {}", output.profile, e.to_string());
            }
        };
    }
}
