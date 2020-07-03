#[derive(Debug, Clone)]
pub struct Message {
    pub ts: chrono::DateTime<chrono::Utc>,
    pub source: String,
    pub text: String,
}

impl Message {
    pub fn user<S, T>(source: S, text: T) -> Self
    where
        S: AsRef<str>,
        T: AsRef<str>,
    {
        Self {
            ts: chrono::Utc::now(),
            source: source.as_ref().to_string(),
            text: text.as_ref().to_string(),
        }
    }

    pub fn notice<T: AsRef<str>>(text: T) -> Self {
        Self {
            ts: chrono::Utc::now(),
            source: format!("NOTICE"),
            text: text.as_ref().to_string(),
        }
    }
}

impl From<(&str, &str)> for Message {
    fn from(data: (&str, &str)) -> Self {
        Self::user(data.0, data.1)
    }
}
