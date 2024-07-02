use std::{error::Error, sync::Arc};

use lemmy_api_common::LemmyErrorType;
use serde::Deserialize;
use thiserror::Error as ThisError;

#[derive(Debug, Clone, ThisError, Deserialize)]
/// An error returned from the API.
#[error("Lemmy Error: {0}")]
#[serde(untagged)]
pub enum LemmyClientError {
    /// Error type returned by Lemmy.
    Lemmy(LemmyErrorType),
    /// Only will get returned in following cases:
    /// - Sending the request fails.
    /// - Parsing the response fails. (Likely due to version differences between domain and client)
    #[serde(skip)]
    Other(Arc<dyn 'static + Error + Sync + Send>),
}

#[cfg(not(target_family = "wasm"))]
impl From<reqwest::Error> for LemmyClientError {
    fn from(e: reqwest::Error) -> Self {
        Self::Other(Arc::new(e))
    }
}

#[cfg(target_family = "wasm")]
impl From<gloo_net::Error> for Error {
    fn from(e: gloo_net::Error) -> Self {
        Self(e.to_string())
    }
}
