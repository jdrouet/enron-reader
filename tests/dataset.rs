#[test]
fn read_all() {
    let mut errors = 0usize;
    for item in enron_reader::read("./maildir") {
        if let Err(err) = item {
            eprintln!("{err:?}");
            errors += 1;
        }
    }
    assert_eq!(
        errors, 90,
        "expected no error due to non utf8 data, got {errors}"
    );
}
