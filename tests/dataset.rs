#[test]
fn read_all() {
    let mut errors = 0usize;
    for item in enron_reader::read("./maildir") {
        if let Err(err) = item {
            eprintln!("{err:?}");
            errors += 1;
        }
    }
    assert_eq!(errors, 0, "expected no error, got {errors}");
}
