use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use log::debug;
use openssl::pkey::{PKey, Private};
use openssl::sign::Signer;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use reqwest::{blocking, Method};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use crate::PremiumIndex;

#[pyclass]
pub struct Rest {
    base_uri: String,
    apikey: String,
    private_key: PKey<Private>,
    recvwindow: i64,
}

impl Rest {
    fn send(
        &self,
        method: Method,
        path: &str,
        mut params: HashMap<String, String>,
        authenticate: bool,
    ) -> String {
        if authenticate {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();

            params.insert("timestamp".into(), ts.to_string());
            if self.recvwindow > 0 {
                params.insert("recvWindow".into(), self.recvwindow.to_string());
            }
        }

        let mut builder = blocking::Client::new()
            .request(
                method,
                format!("{}/{}", self.base_uri, path.trim_start_matches("/")),
            )
            .header("X-MBX-APIKEY", &self.apikey)
            .query(&params);

        if authenticate {
            let data = params
                .into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<String>>()
                .join("&");
            builder = builder.query(&[("signature", self.sign(&data))]);
        }

        debug!("{:?}", builder);
        let rsp = builder.send().unwrap();
        rsp.text().unwrap()
    }
}

#[pymethods]
impl Rest {
    #[new]
    pub fn new(base_uri: &str, apikey: &str, pem: &str, recvwindow: i64) -> Self {
        let mut buf = Vec::new();
        File::open(pem).unwrap().read_to_end(&mut buf).unwrap();
        let private_key = PKey::private_key_from_pem(&buf).unwrap();

        Self {
            base_uri: base_uri.trim_end_matches("/").into(),
            apikey: apikey.into(),
            private_key,
            recvwindow,
        }
    }

    pub fn sign(&self, data: &str) -> String {
        let mut signer = Signer::new_without_digest(&self.private_key).unwrap();
        let signature = signer.sign_oneshot_to_vec(data.as_bytes());
        BASE64_STANDARD.encode(signature.unwrap())
    }

    pub fn get(
        &self,
        path: &str,
        params: Bound<'_, PyDict>,
        authenticate: bool,
    ) -> PyResult<String> {
        let params: HashMap<String, String> = params.extract()?;
        Ok(self.send(Method::GET, path, params, authenticate))
    }

    pub fn post(
        &self,
        path: &str,
        params: Bound<'_, PyDict>,
        authenticate: bool,
    ) -> PyResult<String> {
        let params: HashMap<String, String> = params.extract()?;
        Ok(self.send(Method::POST, path, params, authenticate))
    }

    pub fn delete(
        &self,
        path: &str,
        params: Bound<'_, PyDict>,
        authenticate: bool,
    ) -> PyResult<String> {
        let params: HashMap<String, String> = params.extract()?;
        Ok(self.send(Method::DELETE, path, params, authenticate))
    }

    pub fn put(
        &self,
        path: &str,
        params: Bound<'_, PyDict>,
        authenticate: bool,
    ) -> PyResult<String> {
        let params: HashMap<String, String> = params.extract()?;
        Ok(self.send(Method::PUT, path, params, authenticate))
    }

    pub fn patch(
        &self,
        path: &str,
        params: Bound<'_, PyDict>,
        authenticate: bool,
    ) -> PyResult<String> {
        let params: HashMap<String, String> = params.extract()?;
        Ok(self.send(Method::PATCH, path, params, authenticate))
    }

    fn get_premium_index(&self) -> Vec<PremiumIndex> {
        let res = self.send(
            Method::GET,
            "/fapi/v1/premiumIndex",
            HashMap::default(),
            false,
        );
        serde_json::from_str(&res).unwrap()
    }
}
