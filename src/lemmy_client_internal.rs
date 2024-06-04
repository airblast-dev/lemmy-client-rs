use crate::utils::ClientOptions;
use std::collections::HashMap;

trait WithHeaders {
    fn with_headers(self, headers: &HashMap<String, String>) -> Self;
}

trait MaybeWithJwt {
    fn maybe_with_jwt(self, jwt: Option<String>) -> Self;
}

fn build_route(route: &str, ClientOptions { domain, secure }: &ClientOptions) -> String {
    format!(
        "http{}://{domain}/api/v3/{route}",
        if *secure { "s" } else { "" }
    )
}

#[cfg(target_family = "wasm")]
mod goober {
    use gloo_net::http::{Request, RequestBuilder};
    use web_sys::wasm_bindgen::UnwrapThrowExt;
    pub struct Fetch(ClientOptions);

    impl Fetch {
        pub fn new(options: ClientOptions) -> Self {
            Self(options)
        }

        fn build_fetch_query<T: serde::Serialize>(&self, path: &str, form: &T) -> String {
            let form_str = serde_urlencoded::to_string(form).unwrap_or_else(|_| path.to_string());
            format!("{}?{}", build_route(path, &self.0), form_str)
        }
    }

    impl WithHeaders for RequestBuilder {
        fn with_headers(self, headers: &HashMap<String, String>) -> Self {
            headers.iter().fold(self, |acc, (header, value)| {
                acc.header(header.as_str(), value.as_str())
            })
        }
    }

    impl MaybeWithJwt for RequestBuilder {
        fn maybe_with_jwt(self, jwt: Option<String>) -> Self {
            if let Some(jwt) = jwt {
                self.header(
                    http::header::AUTHORIZATION.as_str(),
                    format!("Bearer {jwt}").as_str(),
                )
            } else {
                self
            }
        }
    }

    impl private_trait::LemmyClientInternal for Fetch {
        async fn make_request<Response, Form>(
            &self,
            method: Method,
            path: &str,
            request: LemmyRequest<Form>,
            headers: &HashMap<String, String>,
        ) -> LemmyResult<Response>
        where
            Response: LemmyResponse,
            Form: LemmyForm,
        {
            let route = &build_route(path, &self.0);
            let LemmyRequest { body, jwt } = request;

            #[allow(unused_mut)]
            let mut req = match method {
                Method::GET => Request::get(&self.build_fetch_query(path, &body)),
                Method::POST => Request::post(route),
                Method::PUT => Request::put(route),
                method => unreachable!(
                    "This crate only uses GET, POST, and PUT HTTP methods. Got {method:?}"
                ),
            }
            .with_headers(headers)
            .maybe_with_jwt(jwt);

            #[cfg(all(feature = "leptos", target_family = "wasm"))]
            {
                use web_sys::AbortController;
                let abort_controller = AbortController::new().ok();
                let abort_signal = abort_controller.as_ref().map(AbortController::signal);
                leptos::on_cleanup(move || {
                    if let Some(abort_controller) = abort_controller {
                        abort_controller.abort()
                    }
                });
                req = req.abort_signal(abort_signal.as_ref());
            }

            match method {
                Method::GET => req.build().expect_throw("Could not parse query params"),
                Method::POST | Method::PUT => {
                    req.json(&body).expect_throw("Could not parse JSON body")
                }
                method => unreachable!(
                    "This crate only uses GET, POST, and PUT HTTP methods. Got {method:?}"
                ),
            }
            .send()
            .await?
            .json::<Response>()
            .await
            .map_err(Into::into)
        }
    }

    impl LemmyClientInternal for Fetch {}
}

#[cfg(not(target_family = "wasm"))]
mod goober {
    use std::collections::HashMap;

    use http::Method;

    use crate::{
        form::LemmyForm,
        lemmy_client_trait::{private_trait, LemmyClientInternal},
        response::{LemmyResponse, LemmyResult},
        ClientOptions, LemmyRequest,
    };

    use super::{build_route, MaybeWithJwt, WithHeaders};

    impl WithHeaders for reqwest::RequestBuilder {
        fn with_headers(self, headers: &HashMap<String, String>) -> Self {
            let mut client = headers
                .iter()
                .fold(self, |acc, (header, value)| acc.header(header, value));

            if !headers
                .keys()
                .any(|key| key.eq_ignore_ascii_case("user-agent"))
            {
                client = client.header("user-agent", "Lemmy-Client-rs/0.19.3");
            }

            client
        }
    }

    impl MaybeWithJwt for reqwest::RequestBuilder {
        fn maybe_with_jwt(self, jwt: Option<String>) -> Self {
            if let Some(jwt) = jwt {
                self.bearer_auth(jwt)
            } else {
                self
            }
        }
    }

    pub struct ClientWrapper {
        client: reqwest::Client,
        options: ClientOptions,
    }

    impl ClientWrapper {
        pub fn new(options: ClientOptions) -> Self {
            Self {
                client: reqwest::Client::new(),
                options,
            }
        }
    }

    impl private_trait::LemmyClientInternal for ClientWrapper {
        async fn make_request<Response, Form>(
            &self,
            method: Method,
            path: &str,
            request: LemmyRequest<Form>,
            headers: &HashMap<String, String>,
        ) -> LemmyResult<Response>
        where
            Response: LemmyResponse,
            Form: LemmyForm,
        {
            let route = build_route(path, &self.options);
            let LemmyRequest { body, jwt } = request;

            match method {
                Method::GET => self.client.get(route),
                Method::POST => self.client.post(route),
                Method::PUT => self.client.put(route),
                _ => unreachable!("This crate does not use other HTTP methods."),
            }
            .with_headers(headers)
            .maybe_with_jwt(jwt)
            .json(&body)
            .send()
            .await?
            .json::<Response>()
            .await
            .map_err(Into::into)
        }
    }

    impl LemmyClientInternal for ClientWrapper {}
}

pub use goober::*;
