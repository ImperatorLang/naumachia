use super::*;
use crate::{
    error::Error,
    ledger_client::test_ledger_client::TestBackendsBuilder,
    scripts::TxContext,
    scripts::{MintingPolicy, ScriptError, ScriptResult},
};

struct AliceCanMintPolicy;

impl MintingPolicy for AliceCanMintPolicy {
    fn execute(&self, ctx: TxContext) -> ScriptResult<()> {
        if ctx.signer == Address::new("alice") {
            Ok(())
        } else {
            Err(ScriptError::FailedToExecute(
                "Signer must be `alice`".to_string(),
            ))
        }
    }

    fn id(&self) -> String {
        "OnlyAliceCanMint".to_string()
    }
}

#[tokio::test]
async fn mint__alice_can_mint() {
    let signer = Address::new("alice");
    let backend = TestBackendsBuilder::<(), ()>::new(&signer).build_in_memory();
    let amount = 100;

    let asset_name = None;
    let u_tx: TxActions<(), ()> = TxActions::default().with_mint(
        amount,
        asset_name.clone(),
        &signer,
        (),
        Box::new(AliceCanMintPolicy),
    );

    backend.process(u_tx).await.unwrap();

    let id = AliceCanMintPolicy.id();
    let policy_id = PolicyId::native_token(&id, &asset_name);

    let expected = 100;
    let actual = backend
        .ledger_client
        .balance_at_address(&signer, &policy_id)
        .await
        .unwrap();

    assert_eq!(expected, actual);
}

// TODO: Include mint check in test ledger client plz
// #[tokio::test]
// async fn mint__bob_cannot_mint() {
//     let signer = Address::new("bob");
//     let backend = TestBackendsBuilder::<(), ()>::new(&signer).build_in_memory();
//     let amount = 100;
//
//     let asset_name = None;
//     let u_tx: TxActions<(), ()> = TxActions::default().with_mint(
//         amount,
//         asset_name,
//         &signer,
//         (),
//         Box::new(AliceCanMintPolicy),
//     );
//
//     let actual_err = backend.process(u_tx).await.unwrap_err();
//
//     let matches = matches!(actual_err, Error::Script(ScriptError::FailedToExecute(_)),);
//     assert!(matches);
// }
