use gpui::App;
use http_client::{HttpClientWithUrl, read_proxy_from_env};
use serde::Deserialize;
use settings::{RegisterSetting, Settings};
use std::sync::Arc;
use url::Url;

#[derive(Deserialize, Default, RegisterSetting)]
pub struct ProxySettings {
    pub proxy: Option<String>,
}

impl ProxySettings {
    pub fn proxy_url(&self) -> Option<Url> {
        self.proxy
            .as_deref()
            .map(str::trim)
            .filter(|input| !input.is_empty())
            .and_then(|input| {
                input
                    .parse::<Url>()
                    .inspect_err(|e| log::error!("Error parsing proxy settings: {}", e))
                    .ok()
            })
            .or_else(read_proxy_from_env)
    }
}

impl Settings for ProxySettings {
    fn from_settings(content: &settings::SettingsContent) -> Self {
        Self {
            proxy: content
                .proxy
                .as_deref()
                .map(str::trim)
                .filter(|proxy| !proxy.is_empty())
                .map(ToOwned::to_owned),
        }
    }
}

pub struct Client(Arc<HttpClientWithUrl>);

impl Client {
    pub fn new(http: Arc<HttpClientWithUrl>) -> Arc<Self> {
        Arc::new(Self(http))
    }

    pub fn production(cx: &mut App) -> Arc<Self> {
        let http = Arc::new(HttpClientWithUrl::new_url(
            cx.http_client(),
            cx.http_client().proxy().cloned(),
        ));
        Self::new(http)
    }

    pub fn http_client(&self) -> Arc<HttpClientWithUrl> {
        self.0.clone()
    }
}
