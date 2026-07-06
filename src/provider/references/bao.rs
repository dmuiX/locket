//! Defines the OpenBao / Vault secret reference type and its parsing logic.
use super::{Extract, ReferenceSyntax, SecretReference};
use percent_encoding::percent_decode_str;
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BaoParseError {
    #[error("reference must start with 'bao://'")]
    InvalidScheme,

    #[error("invalid URL structure: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("missing mount name")]
    MissingMount,

    #[error("invalid path segments: expected at least 2 (path/field), got {0}")]
    InvalidSegments(usize),

    #[error("mount, path, or field cannot be empty")]
    EmptyComponent,

    #[error("utf8 decode error: {0}")]
    Utf8(#[from] std::str::Utf8Error),
}

/// Represents a syntactically valid OpenBao / Vault secret reference.
/// Syntax: `bao://<mount>/<path>/<field>`
///
/// * `mount` is the path where the KV v2 secrets engine is mounted (e.g. `secret`)
/// * `path` is the secret's path within that engine, may contain nested segments (e.g. `app/prod`)
/// * `field` is the specific key within the secret's data map
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BaoReference {
    /// The raw original string
    raw: String,

    pub mount: String,
    pub path: String,
    pub field: String,
}

impl FromStr for BaoReference {
    type Err = BaoParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("bao://") {
            return Err(BaoParseError::InvalidScheme);
        }

        let url = url::Url::parse(s)?;

        let host_str = url.host_str().ok_or(BaoParseError::MissingMount)?;
        let mount = percent_decode_str(host_str)
            .decode_utf8()
            .map_err(BaoParseError::Utf8)?
            .to_string();

        let raw_segments = url
            .path_segments()
            .ok_or(BaoParseError::InvalidSegments(0))?;

        let mut segments = Vec::new();
        for segment in raw_segments {
            let decoded = percent_decode_str(segment)
                .decode_utf8()
                .map_err(BaoParseError::Utf8)?
                .to_string();
            segments.push(decoded);
        }

        if segments.len() < 2 {
            return Err(BaoParseError::InvalidSegments(segments.len()));
        }

        let field = segments.pop().expect("segments has at least 2 elements");
        let path = segments.join("/");

        if mount.is_empty() || path.is_empty() || field.is_empty() {
            return Err(BaoParseError::EmptyComponent);
        }

        Ok(Self {
            raw: s.to_string(),
            mount,
            path,
            field,
        })
    }
}

impl From<BaoReference> for SecretReference {
    fn from(r: BaoReference) -> Self {
        Self::Bao(r)
    }
}

impl ReferenceSyntax for BaoReference {
    fn try_parse(raw: &str) -> Option<Self> {
        Self::from_str(raw)
            .inspect_err(|e| {
                if !matches!(e, BaoParseError::InvalidScheme) {
                    tracing::warn!("Invalid OpenBao reference '{}': {}", raw, e);
                }
            })
            .ok()
    }
}

impl Extract for BaoReference {
    fn extract(r: &SecretReference) -> Option<&Self> {
        #[allow(unreachable_patterns)]
        match r {
            SecretReference::Bao(inner) => Some(inner),
            _ => None,
        }
    }
}

impl BaoReference {
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

impl std::fmt::Display for BaoReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw)
    }
}
