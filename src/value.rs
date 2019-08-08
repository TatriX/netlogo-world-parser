//! Value type for custom fields.

use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum Value {
    Bool(bool),
    U64(u64),
    I64(i64),
    Float(f64),
    String(String),
}

/// Allow convection to a desired type via `try_into`.
macro_rules! impl_value_try_from {
    ($from:path, $to:path) => {
        impl TryFrom<Value> for $to {
            type Error = String;

            fn try_from(value: Value) -> Result<Self, Self::Error> {
                match value {
                    $from(x) => Ok(x),
                    _ => Err(format!("Expected '{}' got {:?}", stringify!($from), value)),
                }
            }
        }
    };
}

impl_value_try_from!(Value::Bool, bool);
impl_value_try_from!(Value::U64, u64);
impl_value_try_from!(Value::I64, i64);
impl_value_try_from!(Value::Float, f64);
impl_value_try_from!(Value::String, String);
