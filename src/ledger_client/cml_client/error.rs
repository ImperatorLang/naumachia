use crate::ledger_client::LedgerClientError;
use crate::Address;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CMLLCError {
    #[error("CML JsError: {0:?}")]
    JsError(String),
    #[error("Not a valid BaseAddress")]
    InvalidBaseAddr,
    #[error("Error from ledger implementation: {0:?}")]
    LedgerError(Box<dyn std::error::Error + Send>),
    #[error("Error in key manager implementation: {0:?}")]
    KeyError(Box<dyn std::error::Error + Send>),
    #[error("Unbuilt output does not have sufficient ADA")]
    InsufficientADA,
}

pub fn as_failed_to_retrieve_by_address(
    addr: &Address,
) -> impl Fn(CMLLCError) -> LedgerClientError + '_ {
    move |e| LedgerClientError::FailedToRetrieveOutputsAt(addr.to_owned(), Box::new(e))
}

pub fn as_failed_to_issue_tx(error: CMLLCError) -> LedgerClientError {
    LedgerClientError::FailedToIssueTx(Box::new(error))
}

pub type Result<E, T = CMLLCError> = std::result::Result<E, T>;