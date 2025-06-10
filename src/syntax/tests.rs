use super::*;

#[test]
fn parse_sample_file() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test")
        .join("test.repack");
    let contents = FileContents::new(path.to_str().unwrap());
    let result = ParseResult::from_contents(contents).expect("parse failed");

    assert!(result.objects.iter().any(|o| o.name == "User"));
    assert!(result.objects.iter().any(|o| o.name == "Todo"));

    let user = result.objects.iter().find(|o| o.name == "User").unwrap();
    let field_names: Vec<_> = user.fields.iter().map(|f| f.name.as_str()).collect();
    assert!(field_names.contains(&"id"));
    assert!(field_names.contains(&"org_id"));

    let langs: Vec<_> = result.languages.iter().map(|l| l.profile.as_str()).collect();
    assert!(langs.contains(&"postgres"));
    assert!(langs.contains(&"rust"));
}

#[test]
fn parse_invalid_emits_errors() {
    use std::path::PathBuf;

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("test")
        .join("invalid.repack");
    let contents = FileContents::new(path.to_str().unwrap());
    let errs = ParseResult::from_contents(contents).expect_err("expected errors");

    let kinds: Vec<_> = errs.iter().map(|e| e.error).collect();
    assert!(kinds.contains(&RepackErrorKind::UnknownLanguage));
    assert!(kinds.contains(&RepackErrorKind::NoTableName));
    assert!(kinds.contains(&RepackErrorKind::NoFields));
}
