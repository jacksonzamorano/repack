use syntax::FileContents;

mod syntax;
mod outputs;

fn main() {
    let mut contents = FileContents::new();
    contents.read("test.repack");
    let parse_result = syntax::ParseResult::from_contents(contents);
    parse_result.validate(false);
    dbg!(parse_result.languages);
}
