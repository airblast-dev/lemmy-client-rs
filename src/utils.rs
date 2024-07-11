macro_rules! impl_marker_trait {
    ($trait_name:ty, [$( $impler:ty ),+$(,)?]) => {
        $(
            impl $trait_name for $impler {}
        )*
    };
}

pub(crate) use impl_marker_trait;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Options for instantiating a `LemmyClient`.
pub struct ClientOptions {
    /// Domain of the instance the client will send requests to.
    /// ```
    /// use lemmy_client::ClientOptions;
    /// // ❌ You do not have to include the scheme for the domain.
    /// let options = ClientOptions::new("http://lemmy.ml", true);
    ///
    /// // ✅ All you need is the domain (including subdomain, if applicaple).
    /// let options = ClientOptions::new("lemmy.ml", true);
    /// ```
    pub domain: String,
    /// If true, use HTTPS. If false, use HTTP
    pub secure: bool,
    /// The jwt token to be used with all requests.
    ///
    /// Ignored if a token was specifically provided for a request.
    pub jwt: Option<String>,
}

impl ClientOptions {
    /// Create a new [`ClientOptions`].
    pub fn new<S>(domain: S, secure: bool) -> Self
    where
        S: ToString,
    {
        Self {
            domain: domain.to_string(),
            secure,
            jwt: None,
        }
    }

    /// Add a JWT token to the [`ClientOptions`].
    pub fn with_jwt(&mut self, jwt: String) {
        self.jwt = Some(jwt)
    }
}
