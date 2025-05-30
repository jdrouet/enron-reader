// To run this, the enron database is needed
// https://www.cs.cmu.edu/~enron/enron_mail_20150507.tar.gz

use std::path::PathBuf;

pub mod mail;

pub struct EnronReaderIterator {
    stack: Vec<PathBuf>,
}

impl EnronReaderIterator {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let stack = vec![path.into()];
        Self { stack }
    }

    fn read_dir(&mut self, path: PathBuf) -> std::io::Result<()> {
        let content = std::fs::read_dir(path)?;
        for item in content {
            if let Ok(item) = item {
                let path = item.path();
                self.stack.push(path);
            }
        }
        Ok(())
    }
}

impl Iterator for EnronReaderIterator {
    type Item = std::io::Result<mail::EnronMail>;

    fn next(&mut self) -> Option<Self::Item> {
        let path = self.stack.pop()?;
        if path.is_file() {
            return Some(mail::EnronMail::read(path));
        }
        if path.is_dir() {
            let _ = self.read_dir(path);
        }
        self.next()
    }
}

pub fn read(path: impl Into<PathBuf>) -> EnronReaderIterator {
    EnronReaderIterator::new(path)
}
