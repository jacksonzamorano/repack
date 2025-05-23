use language::FileContents;

mod language;

fn main() {
    let mut contents = FileContents::new();
    contents.read("test.repack");
    let parse_result = language::ParseResult::from_contents(contents);
    parse_result.validate(false);
    // dbg!(parse_result);
}
