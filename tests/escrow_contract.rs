#![allow(non_snake_case)]
use naumachia::{
    address::{Address, ADA},
    backend::{
        fake_backend::{FakeBackendsBuilder, FakeRecord},
        TxORecord,
    },
    error::Result,
    logic::Logic,
    output::Output,
    smart_contract::SmartContract,
    transaction::UnBuiltTransaction,
    validator::{TxContext, ValidatorCode},
};
use std::collections::HashMap;

pub struct EscrowValidatorScript;

impl ValidatorCode<EscrowDatum, ()> for EscrowValidatorScript {
    fn execute(&self, datum: EscrowDatum, _redeemer: (), ctx: TxContext) -> Result<()> {
        signer_is_recipient(&datum, &ctx)?;
        Ok(())
    }

    fn address(&self) -> Address {
        Address::new("escrow validator")
    }
}

fn signer_is_recipient(datum: &EscrowDatum, ctx: &TxContext) -> Result<()> {
    if datum.receiver != ctx.signer {
        Err(format!(
            "Signer: {:?} doesn't match receiver: {:?}",
            ctx.signer, datum.receiver
        ))
    } else {
        Ok(())
    }
}

#[derive(Clone)]
struct EscrowContract;

#[derive(Clone)]
enum Endpoint {
    Escrow { amount: u64, receiver: Address },
    Claim { output: Output<EscrowDatum> },
}

#[derive(Clone, PartialEq, Debug)]
struct EscrowDatum {
    receiver: Address,
}

impl Logic for EscrowContract {
    type Endpoint = Endpoint;
    type Datum = EscrowDatum;
    type Redeemer = ();

    fn handle_endpoint(
        endpoint: Self::Endpoint,
        _issuer: &Address,
    ) -> Result<UnBuiltTransaction<EscrowDatum, ()>> {
        match endpoint {
            Endpoint::Escrow { amount, receiver } => escrow(amount, receiver),
            Endpoint::Claim { output } => claim(output),
        }
    }
}

fn escrow(amount: u64, receiver: Address) -> Result<UnBuiltTransaction<EscrowDatum, ()>> {
    let script = EscrowValidatorScript;
    let address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script);
    let datum = EscrowDatum { receiver };
    let mut values = HashMap::new();
    values.insert(ADA, amount);
    let u_tx = UnBuiltTransaction::default().with_script_init(datum, values, address);
    Ok(u_tx)
}

// TODO: Check if can claim first
fn claim(output: Output<EscrowDatum>) -> Result<UnBuiltTransaction<EscrowDatum, ()>> {
    let script = Box::new(EscrowValidatorScript);
    let u_tx = UnBuiltTransaction::default().with_script_redeem(output, (), script);
    Ok(u_tx)
}

#[test]
fn escrow__can_create_instance() {
    let me = Address::new("me");
    let alice = Address::new("alice");
    let start_amount = 100;
    let mut backend = FakeBackendsBuilder::new(EscrowContract, me.clone())
        .start_output(me.clone())
        .with_value(ADA, start_amount)
        .finish_output()
        .build();

    let escrow_amount = 25;
    let call = Endpoint::Escrow {
        amount: escrow_amount,
        receiver: alice.clone(),
    };
    let script = EscrowValidatorScript;
    backend.hit_endpoint(call).unwrap();

    let escrow_address = <dyn ValidatorCode<EscrowDatum, ()>>::address(&script);
    let expected = escrow_amount;
    let actual = <FakeRecord<EscrowDatum> as TxORecord<EscrowDatum, ()>>::balance_at_address(
        &backend.txo_record,
        &script.address(),
        &ADA,
    );
    assert_eq!(expected, actual);

    let expected = start_amount - escrow_amount;
    let actual = <FakeRecord<EscrowDatum> as TxORecord<EscrowDatum, ()>>::balance_at_address(
        &backend.txo_record,
        &me,
        &ADA,
    );
    assert_eq!(expected, actual);

    let instance = <FakeRecord<EscrowDatum> as TxORecord<EscrowDatum, ()>>::outputs_at_address(
        &backend.txo_record,
        &script.address(),
    )
    .pop()
    .unwrap();
    // The creator tries to spend escrow but fails because not recipient
    let call = Endpoint::Claim { output: instance };

    let attempt = backend.hit_endpoint(call.clone());
    assert!(attempt.is_err());

    // The recipient tries to spend and succeeds
    backend.txo_record.signer = alice.clone();
    backend.hit_endpoint(call).unwrap();

    let alice_balance = <FakeRecord<EscrowDatum> as TxORecord<EscrowDatum, ()>>::balance_at_address(
        &backend.txo_record,
        &alice,
        &ADA,
    );
    assert_eq!(alice_balance, escrow_amount);

    let script_balance =
        <FakeRecord<EscrowDatum> as TxORecord<EscrowDatum, ()>>::balance_at_address(
            &backend.txo_record,
            &escrow_address,
            &ADA,
        );
    assert_eq!(script_balance, 0);
}
