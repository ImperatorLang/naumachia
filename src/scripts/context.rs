use crate::scripts::raw_validator_script::plutus_data::PlutusData;
use crate::values::Values;
use crate::{Address, PolicyId};
use std::collections::HashMap;

// TODO: Flesh out and probably move https://github.com/MitchTurner/naumachia/issues/39
#[derive(Clone)]
pub struct TxContext {
    pub signer: Address,
    pub range: ValidRange,
    pub inputs: Vec<Input>,
}

#[derive(Clone)]
pub struct ValidRange {
    pub lower: Option<(i64, bool)>,
    pub upper: Option<(i64, bool)>,
}

#[derive(Clone)]
pub struct Input {
    pub transaction_id: String,
    pub output_index: u64,
    pub address: String,
    pub value: CtxValue,
    pub datum: Datum,
    pub reference_script: Option<Vec<u8>>,
}

#[derive(Clone)]
pub struct CtxValue {
    pub inner: HashMap<String, HashMap<String, u64>>,
}

impl From<Values> for CtxValue {
    fn from(values: Values) -> Self {
        let mut inner = HashMap::new();
        for (policy, amt) in values.as_iter() {
            let (policy_id, asset_name) = match policy {
                PolicyId::ADA => ("", ""),
                PolicyId::NativeToken(policy_id, a) => {
                    if let Some(asset_name) = a {
                        (policy_id.as_str(), asset_name.as_str())
                    } else {
                        (policy_id.as_str(), "")
                    }
                }
            };
            add_to_nested(&mut inner, policy_id, asset_name, *amt);
        }
        CtxValue { inner }
    }
}

#[derive(Clone)]
pub enum Datum {
    NoDatum,
    DatumHash(Vec<u8>),
    InlineDatum(PlutusData),
}

impl<D: Clone + Into<PlutusData>> From<Option<D>> for Datum {
    fn from(value: Option<D>) -> Self {
        match value {
            None => Datum::NoDatum,
            Some(datum) => Datum::InlineDatum(datum.into()),
        }
    }
}

pub struct ContextBuilder {
    signer: Address,
    range: Option<ValidRange>,
    inputs: Vec<Input>,
}

impl ContextBuilder {
    pub fn new(signer: Address) -> Self {
        ContextBuilder {
            signer,
            range: None,
            inputs: vec![],
        }
    }

    pub fn with_range(mut self, lower: Option<(i64, bool)>, upper: Option<(i64, bool)>) -> Self {
        let valid_range = ValidRange { lower, upper };
        self.range = Some(valid_range);
        self
    }

    pub fn with_input(
        self,
        transaction_id: String,
        output_index: u64,
        address: String,
    ) -> InputBuilder {
        InputBuilder {
            outer: self,
            transaction_id,
            address,
            value: Default::default(),
            datum: Datum::NoDatum,
            reference_script: None,
            output_index,
        }
    }

    fn add_input(mut self, input: Input) -> ContextBuilder {
        self.inputs.push(input);
        self
    }

    pub fn build(&self) -> TxContext {
        let range = if let Some(range) = self.range.clone() {
            range
        } else {
            ValidRange {
                lower: None,
                upper: None,
            }
        };
        TxContext {
            signer: self.signer.clone(),
            range,
            inputs: vec![],
        }
    }
}

pub struct InputBuilder {
    outer: ContextBuilder,
    transaction_id: String,
    output_index: u64,
    address: String,
    value: HashMap<String, HashMap<String, u64>>,
    datum: Datum,
    reference_script: Option<Vec<u8>>,
}

impl InputBuilder {
    pub fn with_value(mut self, policy_id: &str, asset_name: &str, amt: u64) -> InputBuilder {
        add_to_nested(&mut self.value, policy_id, asset_name, amt);
        self
    }

    pub fn with_inline_datum(mut self, plutus_data: PlutusData) -> InputBuilder {
        self.datum = Datum::InlineDatum(plutus_data);
        self
    }

    pub fn with_datum_hash(mut self, datum_hash: Vec<u8>) -> InputBuilder {
        self.datum = Datum::DatumHash(datum_hash);
        self
    }

    pub fn finish_input(self) -> ContextBuilder {
        let value = CtxValue { inner: self.value };
        let input = Input {
            transaction_id: self.transaction_id,
            output_index: self.output_index,
            address: self.address,
            value,
            datum: self.datum,
            reference_script: self.reference_script,
        };
        self.outer.add_input(input)
    }
}

fn add_to_nested(
    values: &mut HashMap<String, HashMap<String, u64>>,
    policy_id: &str,
    asset_name: &str,
    amt: u64,
) {
    let new_assets = if let Some(mut assets) = values.remove(policy_id) {
        if let Some(mut total_amt) = assets.remove(asset_name) {
            total_amt += amt;
            assets.insert(asset_name.to_string(), total_amt);
        } else {
            assets.insert(asset_name.to_string(), amt);
        }
        assets
    } else {
        let mut assets = HashMap::new();
        assets.insert(asset_name.to_string(), amt);
        assets
    };
    values.insert(policy_id.to_string(), new_assets);
}