use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TuiErrorKind {
    TerminalSetup,
    EventRead,
    Render,
    TerminalCleanup,
    Panic,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct TuiError {
    kind: TuiErrorKind,
    cleanup_kind: Option<TuiErrorKind>,
}

impl TuiError {
    pub const fn new(kind: TuiErrorKind) -> Self {
        Self {
            kind,
            cleanup_kind: None,
        }
    }

    pub const fn with_cleanup(kind: TuiErrorKind, cleanup_kind: TuiErrorKind) -> Self {
        Self {
            kind,
            cleanup_kind: Some(cleanup_kind),
        }
    }

    /// Classifies an owner error without retaining or formatting its source.
    pub fn from_source<E>(kind: TuiErrorKind, _source: &E) -> Self
    where
        E: Error + ?Sized,
    {
        Self::new(kind)
    }

    pub const fn kind(&self) -> TuiErrorKind {
        self.kind
    }

    pub const fn cleanup_kind(&self) -> Option<TuiErrorKind> {
        self.cleanup_kind
    }
}

impl fmt::Debug for TuiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TuiError")
            .field("kind", &self.kind)
            .field("cleanup_kind", &self.cleanup_kind)
            .finish()
    }
}

impl fmt::Display for TuiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TUI operation failed")
    }
}

impl Error for TuiError {}

#[cfg(test)]
mod tests {
    use super::{TuiError, TuiErrorKind};
    use std::error::Error;
    use std::fmt;

    struct SecretSource(&'static str);

    impl fmt::Debug for SecretSource {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.0)
        }
    }

    impl fmt::Display for SecretSource {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.0)
        }
    }

    impl Error for SecretSource {}

    #[test]
    fn error_debug_display_and_chain_drop_source_text() {
        let secret = "fake-token://alice:sentinel@example.invalid Authorization: sentinel";
        let source = SecretSource(secret);
        let error = TuiError::from_source(TuiErrorKind::Render, &source);

        assert_eq!(error.kind(), TuiErrorKind::Render);
        assert!(!format!("{error:?}").contains("sentinel"));
        assert!(!error.to_string().contains("sentinel"));
        assert!(Error::source(&error).is_none());
    }

    #[test]
    fn cleanup_kind_does_not_replace_primary_kind() {
        let error = TuiError::with_cleanup(TuiErrorKind::Render, TuiErrorKind::TerminalCleanup);
        assert_eq!(error.kind(), TuiErrorKind::Render);
        assert_eq!(error.cleanup_kind(), Some(TuiErrorKind::TerminalCleanup));
    }
}
