use std::{error::Error, sync::Arc};

use lemmy_api_common::LemmyErrorType;
use serde::Deserialize;
use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError, Deserialize)]
/// An error returned from the API.
#[error("Lemmy Error: {0}")]
#[serde(untagged)]
pub enum LemmyClientError {
    /// Only will get returned in following cases:
    /// - Sending the request fails.
    /// - Parsing the response fails. (Likely due to version differences between domain and client)
    #[serde(skip)]
    Other(Arc<dyn 'static + Error + Sync + Send>),
    /// Error type returned by Lemmy.
    #[serde(untagged)]
    Lemmy(LemmyErrorType),
}

#[cfg(not(target_family = "wasm"))]
impl From<reqwest::Error> for LemmyClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Other(Arc::new(e))
    }
}

impl From<LemmyErrorType> for LemmyClientError {
    fn from(value: LemmyErrorType) -> Self {
        Self::Lemmy(value)
    }
}

#[cfg(target_family = "wasm")]
impl From<gloo_net::Error> for Error {
    fn from(e: gloo_net::Error) -> Self {
        Self(e.to_string())
    }
}
