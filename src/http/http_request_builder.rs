use std::collections::HashMap;
use std::time::Duration;

use crate::data::live_common::{HttpData, TikTokLiveSettings};
use bytes::Bytes;
use isahc::{
    config::RedirectPolicy, prelude::*, AsyncBody, Body, HttpClient, Response, ResponseFuture,
};
// use reqwest::{Client, RequestBuilder};
use urlencoding::encode;

pub struct HttpRequestFactory {
    pub(crate) settings: TikTokLiveSettings,
}

impl HttpRequestFactory {
    pub fn request(&self) -> HttpRequestBuilder {
        HttpRequestBuilder {
            url: "".to_string(),
            http_data: self.settings.http_data.clone(),
        }
    }
}

pub struct HttpRequestBuilder {
    url: String,
    http_data: HttpData,
}

impl HttpRequestBuilder {
    pub fn with_reset(&mut self) -> &mut Self {
        self.http_data = HttpData::default();
        self
    }

    pub fn with_time_out(&mut self, time_out: Duration) -> &mut Self {
        self.http_data.time_out = time_out;
        self
    }
    pub fn with_url(&mut self, url: &str) -> &mut Self {
        self.url = url.to_string();
        self
    }

    pub fn with_param(&mut self, name: &str, value: &str) -> &mut Self {
        self.http_data
            .params
            .insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_params(&mut self, params: &HashMap<String, String>) -> &mut Self {
        for entry in params {
            self.with_param(entry.0, entry.1);
        }

        self
    }

    pub fn with_header(&mut self, name: &str, value: &str) -> &mut Self {
        self.http_data
            .headers
            .insert(name.to_string(), value.to_string());
        self
    }

    pub fn with_cookie(&mut self, name: &str, value: &str) -> &mut Self {
        self.http_data
            .cookies
            .insert(name.to_string(), value.to_string());
        self
    }

    pub fn build_client(&mut self) -> HttpClient {
        let client = HttpClient::builder()
            .timeout(Duration::from_secs(5))
            .redirect_policy(RedirectPolicy::Follow)
            .build()
            .expect("Failed to build HTTP client");

        client
    }
    pub async fn get_request(&mut self) -> Result<Response<AsyncBody>, isahc::Error> {
        let client = self.build_client();
        let url = self.as_url();
        client.get_async(url).await
        // for header in self.http_data.headers.clone() {
        //     res = res.header(header.0, header.1);
        // }
        // res
    }

    pub async fn as_json(&mut self) -> Option<String> {
        let client = self.build_client();
        let mut result = client.get_async(self.as_url()).await.unwrap();
        if result.status().is_success() {
            let json_res = result.text().await.unwrap();
            Some(json_res)
        } else {
            None
        }
    }

    pub async fn as_bytes(&mut self) -> Option<Bytes> {
        let client = self.build_client();
        let mut result = client.get_async(self.as_url()).await.unwrap();

        if result.status().is_success() {
            let bytes = result.bytes().await.unwrap().into();
            Some(bytes)
        } else {
            None
        }
    }

    pub fn as_url(&mut self) -> String {
        if self.http_data.params.len() == 0 {
            return self.url.to_string();
        }

        let query = self
            .http_data
            .params
            .iter()
            .map(|(key, value)| format!("{}={}", key, encode(value)))
            .collect::<Vec<_>>()
            .join("&");
        let url = format!("{}?{}", self.url, query);
        url
    }
}
