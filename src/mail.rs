use std::{borrow::Cow, collections::HashMap, path::PathBuf};

#[derive(Debug)]
pub struct Error {
    pub reason: Cow<'static, str>,
    pub line: String,
    pub path: PathBuf,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?} ({:?})", self.reason, self.line, self.path)
    }
}

impl std::error::Error for Error {}

pub struct EnronMail {
    pub path: PathBuf,
    pub message_id: Box<str>,
    pub date: chrono::DateTime<chrono::FixedOffset>,
    pub header: HashMap<Box<str>, Box<str>>,
    pub body: Box<str>,
}

impl EnronMail {
    pub fn parse(path: PathBuf, input: &str) -> Result<Self, Error> {
        let (body, mut header) = EnronMailHeaderParser::new(path.clone()).parse(input)?;
        let message_id = header.remove("Message-ID").ok_or_else(|| Error {
            reason: "message ID header not found".into(),
            line: String::default(),
            path: path.clone(),
        })?;
        let date = header.remove("Date").ok_or_else(|| Error {
            reason: "date header not found".into(),
            line: Default::default(),
            path: path.clone(),
        })?;
        let date = chrono::DateTime::parse_from_rfc2822(date.as_ref()).map_err(|_| Error {
            reason: "unable to parse date".into(),
            line: Default::default(),
            path: path.clone(),
        })?;
        Ok(Self {
            path,
            message_id,
            date,
            header,
            body: body.into(),
        })
    }

    pub fn read(path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path = path.into();
        let content = std::fs::read_to_string(&path).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                Error {
                    reason: err.to_string().into(),
                    line: Default::default(),
                    path: path.clone(),
                },
            )
        })?;
        Self::parse(path, &content)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
    }
}

#[derive(Debug)]
struct EnronMailHeaderParser<'a> {
    path: PathBuf,
    res: HashMap<Box<str>, Box<str>>,
    previous_line: Option<(Box<str>, Vec<&'a str>)>,
}

impl<'a> EnronMailHeaderParser<'a> {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            res: Default::default(),
            previous_line: None,
        }
    }

    fn parse_line(&mut self) {
        let Some((name, values)) = self.previous_line.take() else {
            return;
        };
        let value = values
            .into_iter()
            .map(|item| item.trim())
            .collect::<Vec<_>>()
            .join(" ")
            .into_boxed_str();

        self.res.insert(name.trim().into(), value);
    }

    fn parse(mut self, input: &'a str) -> Result<(&'a str, HashMap<Box<str>, Box<str>>), Error> {
        let mut buffer = input;
        loop {
            let Some((line, next_buf)) = buffer.split_once("\n") else {
                self.parse_line();
                return Ok(("", self.res));
            };
            if line.trim().is_empty() {
                self.parse_line();
                return Ok((next_buf, self.res));
            }
            if let Some((name, rest)) = line.split_once(':') {
                self.parse_line();
                self.previous_line = Some((name.into(), vec![rest]));
            } else {
                let prev = self.previous_line.as_mut().ok_or_else(|| Error {
                    reason: "unable to concat with previous line".into(),
                    line: line.to_owned(),
                    path: self.path.to_path_buf(),
                })?;
                prev.1.push(line);
            }

            buffer = next_buf;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::EnronMail;

    #[test]
    fn should_parse_email_from_benson() {
        let path = PathBuf::from("resources/benson-r-inbox-1");
        let content = include_str!("../resources/benson-r-inbox-1");
        let mail = EnronMail::parse(path, content).unwrap();
        assert_eq!(
            mail.message_id.as_ref(),
            "<33386806.1075840371945.JavaMail.evans@thyme>"
        );
        assert_eq!(
            mail.header.get("From").unwrap().as_ref(),
            "jennifer.mcquade@enron.com"
        );
        assert_eq!(
            mail.header.get("Cc").unwrap().as_ref(),
            "daniel.diamond@enron.com, teresa.mandola@enron.com, corp <.carter@enron.com>, david.forster@enron.com"
        )
    }

    #[test]
    fn should_parse_email_from_blair() {
        let path = PathBuf::from("resources/blair-l-inbox-1");
        let content = include_str!("../resources/blair-l-inbox-1");
        let mail = EnronMail::parse(path, content).unwrap();
        assert_eq!(
            mail.message_id.as_ref(),
            "<1199981.1075853079812.JavaMail.evans@thyme>"
        );
        assert_eq!(
            mail.header.get("From").unwrap().as_ref(),
            "lynn.blair@enron.com"
        );
    }
}
