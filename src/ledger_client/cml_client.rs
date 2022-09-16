use crate::ledger_client::cml_client::error::CMLLCError::JsError;
use crate::{
    address,
    ledger_client::{LedgerClient, LedgerClientError, LedgerClientResult},
    output::Output,
    values::Values,
    Address, PolicyId, Transaction,
};
use async_trait::async_trait;
use blockfrost_http_client::{
    keys::TESTNET,
    models::{UTxO as BFUTxO, Value},
    BlockFrostHttp, BlockFrostHttpTrait,
};
use cardano_multiplatform_lib::address::{Address as CMLAddress, BaseAddress, RewardAddress};
use cardano_multiplatform_lib::crypto::PrivateKey;
use error::*;
use futures::{future::join_all, FutureExt};
use std::marker::PhantomData;
use std::path::Path;
use thiserror::Error;

mod error;
#[cfg(test)]
mod tests;

pub struct CMLLedgerCLient<L, K, Datum, Redeemer>
where
    L: Ledger,
    K: Keys,
{
    ledger: L,
    keys: K,
    _datum: PhantomData<Datum>,
    _redeemer: PhantomData<Redeemer>,
}

#[async_trait]
pub trait Keys {
    async fn base_addr(&self) -> Result<CMLAddress>;
    async fn private_key(&self) -> Result<PrivateKey>;
    async fn addr_from_bech_32(&self, addr: &str) -> Result<CMLAddress>;
}

pub struct KeyManager {
    config_path: String,
}

impl KeyManager {
    pub fn new(config_path: String) -> Self {
        KeyManager { config_path }
    }
}

#[async_trait]
impl Keys for KeyManager {
    async fn base_addr(&self) -> Result<CMLAddress> {
        todo!()
    }

    async fn private_key(&self) -> Result<PrivateKey> {
        todo!()
    }

    async fn addr_from_bech_32(&self, addr: &str) -> Result<CMLAddress> {
        let cml_address =
            CMLAddress::from_bech32(addr).map_err(|e| CMLLCError::JsError(e.to_string()))?;
        Ok(cml_address)
    }
}

#[async_trait]
pub trait Ledger {
    async fn get_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<BFUTxO>>; // TODO: Don't take dep on BF UTxO
}

pub struct BlockFrostLedger {
    client: BlockFrostHttp,
}

impl BlockFrostLedger {
    pub fn new(url: &str, key: &str) -> Self {
        let client = BlockFrostHttp::new(url, &key);
        BlockFrostLedger { client }
    }
}

#[async_trait]
impl Ledger for BlockFrostLedger {
    async fn get_utxos_for_addr(&self, addr: &CMLAddress) -> Result<Vec<BFUTxO>> {
        let addr_string = addr.to_bech32(None).map_err(|e| JsError(e.to_string()))?;
        let utxos = self
            .client
            .utxos(&addr_string)
            .await
            .map_err(|e| CMLLCError::LedgerError(Box::new(e)))?;
        Ok(utxos)
    }
}

impl<L, K, D, R> CMLLedgerCLient<L, K, D, R>
where
    L: Ledger,
    K: Keys,
{
    pub fn new(ledger: L, keys: K) -> Self {
        CMLLedgerCLient {
            ledger,
            keys,
            _datum: Default::default(),
            _redeemer: Default::default(),
        }
    }
}

fn output_cml_err(
    address: &Address,
) -> impl Fn(cardano_multiplatform_lib::error::JsError) -> LedgerClientError + '_ {
    move |e| {
        LedgerClientError::FailedToRetrieveOutputsAt(
            address.to_owned(),
            Box::new(CMLLCError::JsError(format!("{:?}", e))),
        )
    }
}

fn output_http_err(
    address: &Address,
) -> impl Fn(blockfrost_http_client::error::Error) -> LedgerClientError + '_ {
    move |e| LedgerClientError::FailedToRetrieveOutputsAt(address.to_owned(), Box::new(e))
}

fn invalid_base_addr(address: &Address) -> LedgerClientError {
    LedgerClientError::FailedToRetrieveOutputsAt(
        address.to_owned(),
        Box::new(CMLLCError::InvalidBaseAddr),
    )
}

#[async_trait]
impl<L, K, Datum, Redeemer> LedgerClient<Datum, Redeemer> for CMLLedgerCLient<L, K, Datum, Redeemer>
where
    L: Ledger + Send + Sync,
    K: Keys + Send + Sync,
    Datum: Send + Sync,
    Redeemer: Send + Sync,
{
    async fn signer(&self) -> LedgerClientResult<&Address> {
        todo!()
    }

    async fn outputs_at_address(
        &self,
        address: &Address,
    ) -> LedgerClientResult<Vec<Output<Datum>>> {
        match address {
            Address::Base(addr_string) => {
                // let cml_address =
                //     CMLAddress::from_bech32(addr_string).map_err(output_cml_err(address))?;
                // let base_addr = BaseAddress::from_address(&cml_address)
                //     .ok_or_else(|| invalid_base_addr(address))?;
                // let staking_cred = base_addr.stake_cred();
                //
                // let reward_addr = RewardAddress::new(TESTNET, &staking_cred)
                //     .to_address()
                //     .to_bech32(None)
                //     .map_err(output_cml_err(address))?;
                //
                // let addresses = self
                //     .client
                //     .assoc_addresses(&reward_addr)
                //     .await
                //     .map_err(output_http_err(address))?;
                //
                // let nested_utxos_futs: Vec<_> = addresses
                //     .iter()
                //     .map(|addr| {
                //         dbg!(&addr);
                //         self.client
                //             .utxos(addr.as_string())
                //             .map(|utxos| (addr.to_owned(), utxos))
                //     })
                //     .collect();
                //
                // let nested_utxos: Vec<_> = join_all(nested_utxos_futs).await;

                let base_addr = self
                    .keys
                    .addr_from_bech_32(addr_string)
                    .await
                    .map_err(as_failed_to_retrieve_by_address(address))?;

                let bf_utxos = self
                    .ledger
                    .get_utxos_for_addr(&base_addr)
                    .await
                    .map_err(as_failed_to_retrieve_by_address(address))?;

                let utxos = bf_utxos
                    .iter()
                    .map(|utxo| bf_utxo_to_utxo(utxo, address))
                    .collect();

                // let mut outputs_for_all_addresses = Vec::new();
                //
                // for (addr, utxos_res) in nested_utxos {
                //     let utxos = utxos_res.map_err(output_http_err(address))?;
                //     let nau_addr = convert_address(addr);
                //     let nau_outputs: Vec<_> = utxos
                //         .iter()
                //         .map(|utxo| into_nau_output(utxo, &nau_addr))
                //         .collect();
                //     outputs_for_all_addresses.extend(nau_outputs);
                // }
                // Ok(outputs_for_all_addresses)
                Ok(utxos)
            }
            Address::Raw(_) => unimplemented!("Doesn't make sense here"),
        }
    }

    async fn issue(&self, _tx: Transaction<Datum, Redeemer>) -> LedgerClientResult<()> {
        todo!()
    }
}

fn bf_utxo_to_utxo<Datum>(utxo: &BFUTxO, owner: &Address) -> Output<Datum> {
    let tx_hash = utxo.tx_hash().to_owned();
    let index = utxo.output_index().to_owned();
    let mut values = Values::default();
    utxo.amount()
        .iter()
        .map(as_nau_value)
        .for_each(|(policy_id, amount)| values.add_one_value(&policy_id, amount));
    Output::new_wallet(tx_hash, index, owner.to_owned(), values)
}

fn as_nau_value(value: &Value) -> (PolicyId, u64) {
    let policy_id = match value.unit() {
        "lovelace" => PolicyId::ADA,
        native_token => {
            let policy = &native_token[..56]; // TODO: Use the rest as asset info
            PolicyId::native_token(policy)
        }
    };
    let amount = value.quantity().parse().unwrap(); // TODO: unwrap
    (policy_id, amount)
}

fn convert_address(bf_addr: blockfrost_http_client::models::Address) -> Address {
    Address::new(bf_addr.as_string())
}
