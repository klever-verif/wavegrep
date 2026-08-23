#![allow(dead_code)]

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WavepeekError {
    #[error("fatal: args: {0}")]
    Args(String),
    #[error("fatal: file: {0}")]
    File(String),
    #[error("fatal: scope: {0}")]
    Scope(String),
    #[error("fatal: signal: {0}")]
    Signal(String),
    #[error("fatal: signal: {0}")]
    SignalNotFound(String),
    #[error("fatal: expr: {0}")]
    Expr(String),
    #[error("fatal: internal: {0}")]
    Internal(String),
    #[error("fatal: unimplemented: {0}")]
    Unimplemented(&'static str),
    #[error("broken pipe")]
    BrokenPipe,
}

impl WavepeekError {
    pub(crate) const fn fatal_code(&self) -> Option<&'static str> {
        match self {
            Self::Args(_) => Some("WPK-F0001"),
            Self::File(_) => Some("WPK-F0002"),
            Self::Scope(_) => Some("WPK-F0003"),
            Self::Signal(_) | Self::SignalNotFound(_) => Some("WPK-F0004"),
            Self::Expr(_) => Some("WPK-F0005"),
            Self::Internal(_) => Some("WPK-F0006"),
            Self::Unimplemented(_) => Some("WPK-F0007"),
            Self::BrokenPipe => None,
        }
    }

    pub(crate) fn message(&self) -> Option<&str> {
        match self {
            Self::Args(message)
            | Self::File(message)
            | Self::Scope(message)
            | Self::Signal(message)
            | Self::SignalNotFound(message)
            | Self::Expr(message)
            | Self::Internal(message) => Some(message),
            Self::Unimplemented(message) => Some(message),
            Self::BrokenPipe => None,
        }
    }

    pub const fn exit_code(&self) -> u8 {
        match self {
            Self::BrokenPipe => 0,
            Self::File(_) => 2,
            Self::Args(_)
            | Self::Scope(_)
            | Self::Signal(_)
            | Self::SignalNotFound(_)
            | Self::Expr(_)
            | Self::Internal(_)
            | Self::Unimplemented(_) => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WavepeekError;

    #[test]
    fn file_errors_use_exit_code_two() {
        let error = WavepeekError::File("cannot open dump.vcd".to_string());

        assert_eq!(error.exit_code(), 2);
        assert_eq!(error.to_string(), "fatal: file: cannot open dump.vcd");
    }

    #[test]
    fn scope_and_signal_errors_use_exit_code_one() {
        let scope = WavepeekError::Scope("scope 'top.cpu' not found".to_string());
        let signal = WavepeekError::Signal("signal 'top.cpu.clk' not found".to_string());
        let missing = WavepeekError::SignalNotFound("signal 'top.cpu.clk' not found".to_string());

        assert_eq!(scope.exit_code(), 1);
        assert_eq!(signal.exit_code(), 1);
        assert_eq!(missing.exit_code(), 1);
        assert_eq!(missing.to_string(), signal.to_string());
        assert_eq!(scope.to_string(), "fatal: scope: scope 'top.cpu' not found");
        assert_eq!(
            signal.to_string(),
            "fatal: signal: signal 'top.cpu.clk' not found"
        );
    }

    #[test]
    fn expr_errors_use_exit_code_one() {
        let error = WavepeekError::Expr("parse:EXPR-PARSE-LOGICAL-UNMATCHED-OPEN".to_string());

        assert_eq!(error.exit_code(), 1);
        assert_eq!(
            error.to_string(),
            "fatal: expr: parse:EXPR-PARSE-LOGICAL-UNMATCHED-OPEN"
        );
    }

    #[test]
    fn fatal_codes_and_messages_cover_every_category() {
        let errors = [
            (WavepeekError::Args("args".into()), "WPK-F0001", "args"),
            (WavepeekError::File("file".into()), "WPK-F0002", "file"),
            (WavepeekError::Scope("scope".into()), "WPK-F0003", "scope"),
            (
                WavepeekError::Signal("signal".into()),
                "WPK-F0004",
                "signal",
            ),
            (
                WavepeekError::SignalNotFound("missing".into()),
                "WPK-F0004",
                "missing",
            ),
            (WavepeekError::Expr("expr".into()), "WPK-F0005", "expr"),
            (
                WavepeekError::Internal("internal".into()),
                "WPK-F0006",
                "internal",
            ),
            (WavepeekError::Unimplemented("later"), "WPK-F0007", "later"),
        ];

        for (error, code, message) in errors {
            assert_eq!(error.fatal_code(), Some(code));
            assert_eq!(error.message(), Some(message));
        }
        assert_eq!(WavepeekError::BrokenPipe.fatal_code(), None);
        assert_eq!(WavepeekError::BrokenPipe.message(), None);
    }
}
