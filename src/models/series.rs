use std::fmt;

/// Identifier kinds supported by the VWD charting API.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SeriesIdentifierKind {
    IssueId,
    VwdKey,
    Other(String),
}

impl SeriesIdentifierKind {
    pub fn from_raw<S: AsRef<str>>(raw: S) -> Self {
        match raw.as_ref().to_ascii_lowercase().as_str() {
            "issueid" => Self::IssueId,
            "vwdkey" => Self::VwdKey,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SeriesIdentifierKind::IssueId => "issueid",
            SeriesIdentifierKind::VwdKey => "vwdkey",
            SeriesIdentifierKind::Other(raw) => raw.as_str(),
        }
    }
}

impl fmt::Display for SeriesIdentifierKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Fully-qualified quote series identifier (prefix + value).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SeriesIdentifier {
    kind: SeriesIdentifierKind,
    value: String,
}

impl SeriesIdentifier {
    pub fn new(kind: SeriesIdentifierKind, value: impl Into<String>) -> Self {
        Self {
            kind,
            value: value.into(),
        }
    }

    pub fn issue_id(value: impl Into<String>) -> Self {
        Self::new(SeriesIdentifierKind::IssueId, value)
    }

    pub fn kind(&self) -> &SeriesIdentifierKind {
        &self.kind
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
