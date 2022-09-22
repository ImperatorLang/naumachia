use crate::address::Address;
use crate::PolicyId;
use thiserror::Error;

// TODO: Flesh out and probably move https://github.com/MitchTurner/naumachia/issues/39
#[derive(Clone)]
pub struct TxContext {
    pub signer: Address,
}

pub trait ValidatorCode<D, R>: Send + Sync {
    fn execute(&self, datum: D, redeemer: R, ctx: TxContext) -> ScriptResult<()>;
    // TODO: Add network param!
    fn address(&self) -> Address;
    fn script_hex(&self) -> &str;
}

pub trait MintingPolicy: Send + Sync {
    fn execute(&self, ctx: TxContext) -> ScriptResult<()>;
    // TODO: Add network param!
    fn id(&self) -> PolicyId;
}

#[derive(Debug, Error)]
pub enum ScriptError {
    #[error("Failed to execute: {0:?}")]
    FailedToExecute(String),
}

pub type ScriptResult<T> = Result<T, ScriptError>;
