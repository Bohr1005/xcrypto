use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use log::*;
use openssl::pkey::{PKey, Private};
use openssl::sign::Signer;
use reqwest::{Method, Response};
use std::fs::File;
use std::io::Read;

pub trait IntoIterTuple<K, V> {
    type Iter: Iterator<Item = (K, V)>;
    fn into_iter(self) -> Self::Iter;
}

#[derive(Debug)]
pub struct Rest {
    base_uri: String,
    apikey: String,
    private_key: PKey<Private>,
    recvwindow: i64,
}

impl Rest {
    pub fn new(base_uri: &str, apikey: &str, pem: &str, recvwindow: i64) -> anyhow::Result<Self> {
        let mut buf = Vec::new();
        File::open(pem).unwrap().read_to_end(&mut buf)?;
        let private_key = PKey::private_key_from_pem(&buf)?;

        Ok(Self {
            base_uri: base_uri.trim_end_matches("/").into(),
            apikey: apikey.into(),
            private_key,
            recvwindow,
        })
    }

    pub fn apikey(&self) -> &str {
        &self.apikey
    }
    
    pub async fn get(
        &self,
        path: &str,
        params: &[(String, String)],
        signature: bool,
    ) -> anyhow::Result<Response> {
        self.send(Method::GET, path, params, signature).await
    }

    pub async fn post(
        &self,
        path: &str,
        params: &[(String, String)],
        signature: bool,
    ) -> anyhow::Result<Response> {
        self.send(Method::POST, path, params, signature).await
    }

    pub async fn delete(
        &self,
        path: &str,
        params: &[(String, String)],
        signature: bool,
    ) -> anyhow::Result<Response> {
        self.send(Method::DELETE, path, params, signature).await
    }

    pub async fn put(
        &self,
        path: &str,
        params: &[(String, String)],
        signature: bool,
    ) -> anyhow::Result<Response> {
        self.send(Method::PUT, path, params, signature).await
    }

    pub async fn patch(
        &self,
        path: &str,
        params: &[(String, String)],
        signature: bool,
    ) -> anyhow::Result<Response> {
        self.send(Method::PATCH, path, params, signature).await
    }

    pub fn sign(&self, data: &String) -> anyhow::Result<String> {
        let mut signer = Signer::new_without_digest(&self.private_key).unwrap();
        let signature = signer.sign_oneshot_to_vec(data.as_bytes())?;

        Ok(BASE64_STANDARD.encode(signature))
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        params: &[(String, String)],
        signature: bool,
    ) -> anyhow::Result<Response> {
        let mut params: Vec<_> = params.into_iter().cloned().collect();

        if signature {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis();
            params.push(("timestamp".into(), ts.to_string()));
            if self.recvwindow > 0 {
                params.push(("recvWindow".into(), self.recvwindow.to_string()));
            }
        }

        let mut builder = reqwest::Client::new()
            .request(
                method,
                format!("{}/{}", self.base_uri, path.trim_start_matches("/")),
            )
            .header("X-MBX-APIKEY", &self.apikey)
            .query(&params);

        if signature {
            let query: String = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join("&");
            builder = builder.query(&[("signature", &self.sign(&query)?)]);
        }

        let rsp = builder.send().await?;
        debug!("{:?}", rsp);

        Ok(rsp)
    }

    pub async fn add_order(
        &self,
        path: &str,
        symbol: String,
        price: String,
        quantity: String,
        side: String,
        order_type: String,
        tif: String,
        session_id: u16,
        id: u32,
    ) -> anyhow::Result<Response> {
        let client_order_id = u64::from(session_id) << 32 | u64::from(id);
        self.post(
            path,
            &[
                ("symbol".into(), symbol),
                ("side".into(), side),
                ("type".into(), order_type),
                ("timeInForce".into(), tif),
                ("quantity".into(), quantity),
                ("price".into(), price),
                ("newClientOrderId".into(), client_order_id.to_string()),
                ("newOrderRespType".into(), "RESULT".into()),
            ],
            true,
        )
        .await
    }

    pub async fn cancel(&self, path: &str, symbol: String, orig: u64) -> anyhow::Result<Response> {
        self.delete(
            path,
            &[
                ("symbol".into(), symbol),
                ("origClientOrderId".into(), orig.to_string()),
            ],
            true,
        )
        .await
    }
}
