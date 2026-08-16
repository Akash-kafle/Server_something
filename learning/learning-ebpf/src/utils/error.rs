#[derive(Debug, Clone, Copy)]
pub enum ParseError {
    InvalidEthernet,
    UnsupportedProtocol,
    Truncated,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::InvalidEthernet => write!(f, "invalid ethernet header"),
            ParseError::UnsupportedProtocol => write!(f, "unsupported protocol"),
            ParseError::Truncated => write!(f, "truncated packet"),
        }
    }
}