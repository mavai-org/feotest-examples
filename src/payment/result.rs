//! Payment transaction result.

/// The outcome of a payment transaction.
///
/// # Examples
///
/// ```
/// use feotest_examples::payment::PaymentResult;
///
/// let success = PaymentResult::success("txn_abc123");
/// assert!(success.is_success());
/// assert_eq!(success.transaction_id(), Some("txn_abc123"));
/// assert!(success.error_code().is_none());
///
/// let failure = PaymentResult::failure("DECLINED");
/// assert!(!failure.is_success());
/// assert_eq!(failure.error_code(), Some("DECLINED"));
/// ```
#[derive(Debug, Clone)]
pub struct PaymentResult {
    success: bool,
    transaction_id: Option<String>,
    error_code: Option<String>,
}

impl PaymentResult {
    /// Creates a successful payment result with a transaction ID.
    #[must_use]
    pub fn success(transaction_id: impl Into<String>) -> Self {
        Self {
            success: true,
            transaction_id: Some(transaction_id.into()),
            error_code: None,
        }
    }

    /// Creates a failed payment result with an error code.
    #[must_use]
    pub fn failure(error_code: impl Into<String>) -> Self {
        Self {
            success: false,
            transaction_id: None,
            error_code: Some(error_code.into()),
        }
    }

    /// Whether the transaction succeeded.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.success
    }

    /// The transaction ID, if successful.
    #[must_use]
    pub fn transaction_id(&self) -> Option<&str> {
        self.transaction_id.as_deref()
    }

    /// The error code, if failed.
    #[must_use]
    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }
}
