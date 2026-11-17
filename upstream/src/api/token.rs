#[derive(Clone, PartialEq, Eq)]
pub struct Token(std::sync::Arc<str>);

impl Token {
    pub fn new(s: String) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// keep secrets out of logs
impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Token(***)")
    }
}
