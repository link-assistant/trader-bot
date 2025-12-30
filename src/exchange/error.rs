//! Exchange-related error types.

use thiserror::Error;

/// Result type for exchange operations.
pub type ExchangeResult<T> = Result<T, ExchangeError>;

/// Errors that can occur when interacting with an exchange.
#[derive(Debug, Error)]
pub enum ExchangeError {
    /// Authentication failed.
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Rate limit exceeded.
    #[error("rate limit exceeded, retry after {retry_after_ms}ms")]
    RateLimitExceeded {
        /// Milliseconds to wait before retrying.
        retry_after_ms: u64,
    },

    /// Network error.
    #[error("network error: {0}")]
    NetworkError(String),

    /// Order rejected by the exchange.
    #[error("order rejected: {0}")]
    OrderRejected(String),

    /// Insufficient funds.
    #[error("insufficient funds: required {required}, available {available}")]
    InsufficientFunds {
        /// Required amount.
        required: String,
        /// Available amount.
        available: String,
    },

    /// Instrument not found.
    #[error("instrument not found: {0}")]
    InstrumentNotFound(String),

    /// Market is closed.
    #[error("market closed for {instrument}")]
    MarketClosed {
        /// The instrument that's not tradeable.
        instrument: String,
    },

    /// Invalid order parameters.
    #[error("invalid order: {0}")]
    InvalidOrder(String),

    /// Order not found.
    #[error("order not found: {0}")]
    OrderNotFound(String),

    /// Exchange returned an unexpected response.
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),

    /// Configuration error.
    #[error("configuration error: {0}")]
    ConfigurationError(String),

    /// Exchange-specific error.
    #[error("exchange error [{code}]: {message}")]
    ExchangeSpecific {
        /// Error code from the exchange.
        code: String,
        /// Error message from the exchange.
        message: String,
    },

    /// Operation timed out.
    #[error("operation timed out after {0}ms")]
    Timeout(u64),

    /// Generic internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ExchangeError {
    /// Returns true if the error is retryable.
    #[must_use]
    pub const fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::RateLimitExceeded { .. } | Self::NetworkError(_) | Self::Timeout(_)
        )
    }

    /// Returns the retry delay in milliseconds if applicable.
    #[must_use]
    pub const fn retry_delay_ms(&self) -> Option<u64> {
        match self {
            Self::RateLimitExceeded { retry_after_ms } => Some(*retry_after_ms),
            Self::NetworkError(_) | Self::Timeout(_) => Some(1000), // Default 1s retry
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retryable_errors() {
        let rate_limit = ExchangeError::RateLimitExceeded {
            retry_after_ms: 5000,
        };
        assert!(rate_limit.is_retryable());
        assert_eq!(rate_limit.retry_delay_ms(), Some(5000));

        let network = ExchangeError::NetworkError("connection reset".into());
        assert!(network.is_retryable());
        assert_eq!(network.retry_delay_ms(), Some(1000));

        let rejected = ExchangeError::OrderRejected("invalid price".into());
        assert!(!rejected.is_retryable());
        assert_eq!(rejected.retry_delay_ms(), None);
    }
}
