pub mod cluster;
pub mod peer;
pub mod topology;
pub mod util;
pub mod vpn;

use std::{convert::Infallible, marker::PhantomData};

#[derive(thiserror::Error, Debug, Eq, PartialEq)]
#[error("Could not convert from `{from}` to `{to}`: {details}")]
pub struct ConversionError {
    from: &'static str,
    to: &'static str,
    details: String,
}

impl ConversionError {
    pub fn new<From, To>(details: impl Into<String>) -> Self {
        Self {
            from: std::any::type_name::<From>(),
            to: std::any::type_name::<To>(),
            details: details.into(),
        }
    }
}

impl From<Infallible> for ConversionError {
    fn from(_: Infallible) -> Self {
        // Since Infallible can never actually be created, we can just panic here
        panic!("Attempted to convert Infallible into ConversionError")
    }
}

pub struct ConversionErrorBuilder<From, To> {
    _from: PhantomData<From>,
    _to: PhantomData<To>,
}

#[allow(clippy::new_ret_no_self)]
impl<From, To> ConversionErrorBuilder<From, To> {
    pub fn message(details: impl Into<String>) -> ConversionError {
        ConversionError::new::<From, To>(details)
    }
    pub fn field_not_set(field: impl Into<String>) -> ConversionError {
        let details = format!("Field '{}' not set", field.into());
        ConversionError::new::<From, To>(details)
    }
}
