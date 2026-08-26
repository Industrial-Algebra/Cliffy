// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Serde serialization helpers for GA3 multivectors
//!
//! Since amari-core's Multivector doesn't implement Serialize/Deserialize,
//! we provide manual serialization as coefficient arrays.

use cliffy_core::GA3;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serialize a `GA3` as a `Vec<f64>` of coefficients
pub fn serialize<S>(mv: &GA3, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    mv.as_slice().serialize(serializer)
}

/// Deserialize a `GA3` from a `Vec<f64>` of coefficients
pub fn deserialize<'de, D>(deserializer: D) -> Result<GA3, D::Error>
where
    D: Deserializer<'de>,
{
    let coeffs = Vec::<f64>::deserialize(deserializer)?;
    Ok(GA3::from_slice(&coeffs))
}

/// Module for serializing `Option<GA3>`
pub mod option {
    use super::*;

    pub fn serialize<S>(mv: &Option<GA3>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match mv {
            Some(m) => serializer.serialize_some(&m.as_slice().to_vec()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<GA3>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt = Option::<Vec<f64>>::deserialize(deserializer)?;
        Ok(opt.map(|coeffs| GA3::from_slice(&coeffs)))
    }
}
