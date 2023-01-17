use crate::scripts::context::{CtxDatum, CtxValue, Input, TxContext, ValidRange};
use crate::scripts::ScriptError;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum PlutusData {
    Constr(Constr<PlutusData>),
    Map(BTreeMap<PlutusData, PlutusData>),
    BigInt(BigInt),
    BoundedBytes(Vec<u8>),
    Array(Vec<PlutusData>),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Constr<T> {
    pub tag: u64,
    pub any_constructor: Option<u64>,
    pub fields: Vec<T>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum BigInt {
    Int { neg: bool, val: u64 },
    BigUInt(Vec<u8>),
    BigNInt(Vec<u8>),
}

impl From<i64> for BigInt {
    fn from(num: i64) -> Self {
        let neg = num.is_negative();
        let val = num.unsigned_abs();
        BigInt::Int { neg, val }
    }
}

impl From<BigInt> for i64 {
    fn from(big_int: BigInt) -> Self {
        match big_int {
            BigInt::Int { neg, val } => {
                let value = val as i64;
                if neg {
                    -value
                } else {
                    value
                }
            }
            BigInt::BigUInt(_) => todo!(),
            BigInt::BigNInt(_) => todo!(),
        }
    }
}

impl From<i64> for PlutusData {
    fn from(num: i64) -> Self {
        let neg = num.is_negative();
        let val = num.unsigned_abs();
        PlutusData::BigInt(BigInt::Int { neg, val })
    }
}

impl TryFrom<PlutusData> for i64 {
    type Error = ScriptError;

    fn try_from(data: PlutusData) -> Result<Self, Self::Error> {
        match data {
            PlutusData::BigInt(inner) => Ok(inner.into()),
            _ => Err(ScriptError::DatumDeserialization(format!("{:?}", data))),
        }
    }
}

// TODO: Don't hardcode values!
// TODO: THIS IS V2 only right now! Add V1!
impl From<TxContext> for PlutusData {
    fn from(ctx: TxContext) -> Self {
        let inputs = PlutusData::Array(ctx.inputs.into_iter().map(Into::into).collect());
        let reference_inputs = PlutusData::Array(vec![]);
        let outputs = PlutusData::Array(vec![]);
        let fee = PlutusData::Map(BTreeMap::from([(
            PlutusData::BoundedBytes(Vec::new()),
            PlutusData::Map(BTreeMap::from([(
                PlutusData::BoundedBytes(Vec::new()),
                PlutusData::BigInt(999_i64.into()),
            )])),
        )]));
        let mint = PlutusData::Map(BTreeMap::from([(
            PlutusData::BoundedBytes(Vec::new()),
            PlutusData::Map(BTreeMap::from([(
                PlutusData::BoundedBytes(Vec::new()),
                PlutusData::BigInt(0_i64.into()),
            )])),
        )]));
        let dcert = PlutusData::Array(vec![]);
        // let wdrl = PlutusData::Array(vec![]);
        let wdrl = PlutusData::Map(BTreeMap::new());
        let valid_range = ctx.range.into();
        let signatories = PlutusData::Array(vec![]);
        // let redeemers = PlutusData::Array(vec![]);
        let redeemers = PlutusData::Map(BTreeMap::new());
        // let data = PlutusData::Array(vec![]);
        let data = PlutusData::Map(BTreeMap::new());
        let id = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![PlutusData::BoundedBytes(Vec::new())],
        });
        let tx_info = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![
                inputs,
                reference_inputs,
                outputs,
                fee,
                mint,
                dcert,
                wdrl,
                valid_range,
                signatories,
                redeemers,
                data,
                id,
            ],
        });
        // Spending
        let purpose = PlutusData::Constr(Constr {
            tag: 122,
            any_constructor: None,
            fields: vec![],
        });
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![tx_info, purpose],
        })
    }
}

impl From<ValidRange> for PlutusData {
    fn from(value: ValidRange) -> Self {
        match (value.lower, value.upper) {
            (None, None) => no_time_bound(),
            (Some((bound, _)), None) => lower_bound(bound),
            (None, Some(_)) => todo!(),
            (Some(_), Some(_)) => todo!(),
        }
    }
}

fn no_time_bound() -> PlutusData {
    PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: vec![
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![
                    // NegInf
                    PlutusData::Constr(Constr {
                        tag: 121,
                        any_constructor: None,
                        fields: vec![],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        tag: 122,
                        any_constructor: None,
                        fields: vec![],
                    }),
                ],
            }),
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![
                    // PosInf
                    PlutusData::Constr(Constr {
                        tag: 123,
                        any_constructor: None,
                        fields: vec![],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        tag: 122,
                        any_constructor: None,
                        fields: vec![],
                    }),
                ],
            }),
        ],
    })
}

fn lower_bound(bound: i64) -> PlutusData {
    PlutusData::Constr(Constr {
        tag: 121,
        any_constructor: None,
        fields: vec![
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![
                    // Finite
                    PlutusData::Constr(Constr {
                        tag: 122,
                        any_constructor: None,
                        fields: vec![PlutusData::BigInt(bound.into())],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        tag: 122,
                        any_constructor: None,
                        fields: vec![],
                    }),
                ],
            }),
            PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![
                    // PosInf
                    PlutusData::Constr(Constr {
                        tag: 123,
                        any_constructor: None,
                        fields: vec![],
                    }),
                    // Closure
                    PlutusData::Constr(Constr {
                        tag: 122,
                        any_constructor: None,
                        fields: vec![],
                    }),
                ],
            }),
        ],
    })
}

impl From<Input> for PlutusData {
    fn from(input: Input) -> Self {
        let output_reference = CtxOutputReference {
            transaction_id: hex::decode(input.transaction_id).unwrap(), // TODO: make transaction_id bytes
            output_index: input.output_index,
        }
        .into();
        let output = CtxOutput {
            address: hex::decode(input.address).unwrap(), // TODO
            value: input.value,
            datum: input.datum,
            reference_script: input.reference_script,
        }
        .into();
        let data = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![output_reference, output],
        });
        // dbg!(&data);
        data
    }
}

// TODO: Move into `Input`
struct CtxOutputReference {
    transaction_id: Vec<u8>,
    output_index: u64,
}

impl From<CtxOutputReference> for PlutusData {
    fn from(out_ref: CtxOutputReference) -> Self {
        let tx_id_bytes = out_ref.transaction_id;
        let transaction_id = PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![PlutusData::BoundedBytes(tx_id_bytes)],
        });
        let output_index = PlutusData::BigInt((out_ref.output_index as i64).into()); // TODO: panic
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![transaction_id, output_index],
        })
    }
}

// TODO: Move into `Input`
struct CtxOutput {
    address: Vec<u8>,
    value: CtxValue,
    datum: CtxDatum,
    reference_script: Option<Vec<u8>>,
}

impl From<CtxOutput> for PlutusData {
    fn from(output: CtxOutput) -> Self {
        let address = PlutusData::BoundedBytes(output.address);
        let value = output.value.into();
        let datum = output.datum.into();
        let reference_script = output.reference_script.into();
        PlutusData::Constr(Constr {
            tag: 121,
            any_constructor: None,
            fields: vec![address, value, datum, reference_script],
        })
    }
}

impl From<CtxValue> for PlutusData {
    fn from(value: CtxValue) -> Self {
        let converted_inner = value
            .inner
            .iter()
            .map(|(p, a)| {
                let policy_id = PlutusData::BoundedBytes(hex::decode(p).unwrap()); // TODO
                let assets = a
                    .iter()
                    .map(|(an, amt)| {
                        let asset_name = PlutusData::BoundedBytes(hex::decode(an).unwrap()); // TODO
                        let amount = PlutusData::BigInt((*amt as i64).into()); // TODO
                        (asset_name, amount)
                    })
                    .collect();
                (policy_id, PlutusData::Map(assets))
            })
            .collect();
        let data = PlutusData::Map(converted_inner);
        // PlutusData::Constr(Constr {
        //     tag: 121,
        //     any_constructor: None,
        //     fields: vec![data],
        // })
        data
    }
}

impl From<CtxDatum> for PlutusData {
    fn from(value: CtxDatum) -> Self {
        match value {
            CtxDatum::NoDatum => PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![],
            }),
            CtxDatum::DatumHash(hash) => PlutusData::Constr(Constr {
                tag: 122,
                any_constructor: None,
                fields: vec![PlutusData::BoundedBytes(hash)],
            }),
            CtxDatum::InlineDatum(data) => PlutusData::Constr(Constr {
                tag: 123,
                any_constructor: None,
                fields: vec![data],
            }),
        }
    }
}

impl<T: Into<PlutusData>> From<Option<T>> for PlutusData {
    fn from(value: Option<T>) -> Self {
        match value {
            None => PlutusData::Constr(Constr {
                tag: 121,
                any_constructor: None,
                fields: vec![],
            }),
            Some(inner) => PlutusData::Constr(Constr {
                tag: 122,
                any_constructor: None,
                fields: vec![inner.into()],
            }),
        }
    }
}

impl From<Vec<u8>> for PlutusData {
    fn from(value: Vec<u8>) -> Self {
        PlutusData::BoundedBytes(value)
    }
}

impl From<()> for PlutusData {
    fn from(_: ()) -> Self {
        PlutusData::BoundedBytes(Vec::new())
    }
}

impl From<PlutusData> for () {
    fn from(_: PlutusData) -> Self {}
}
