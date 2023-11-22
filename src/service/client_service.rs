use anyhow::{Error, Ok};
use embedded_svc::{http::client::Client as HttpClient, io::Write, utils::io};
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_sys as _;
use log::{error, info};
use serde::Serialize;

pub struct ClientService {
    alert_url: String,
}

impl ClientService {
    pub fn new(alert_url: &str) -> ClientService {
        ClientService {
            alert_url: alert_url.to_owned(),
        }
    }

    pub fn post_request(&self, mac_address: &str) -> anyhow::Result<(), anyhow::Error> {
        let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

        let payload = serde_json::to_string(&RequestAlert::new(mac_address.to_owned())).unwrap();
        let payload = payload.as_bytes();

        let content_length_header = format!("{}", payload.len());
        let headers = [
            ("content-type", "application/json"),
            ("content-length", &*content_length_header),
        ];

        let request = client.post(&self.alert_url, &headers);

        if request.is_err() {
            let message = format!("connection error: {:?}", request.err());
            error!("{}", message);
            return Err(Error::msg(message));
        }
        let mut request = request.unwrap();

        if request.write_all(payload).is_err() {
            let message = format!("connection error while trying to write all");
            error!("{}", message);
            return Err(Error::msg(message));
        }
        if request.flush().is_err() {
            let message = format!("connection error while trying to flush");
            error!("{}", message);
            return Err(Error::msg(message));
        }
        info!("-> POST {}", self.alert_url);
        let response = request.submit();
        if response.is_err() {
            let message = format!("connection error while trying to read response");
            error!("{}", message);
            return Err(Error::msg(message));
        }
        let mut response = response.unwrap();

        let status = response.status();
        info!("<- {}", status);
        let mut buf = [0u8; 1024];
        let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0);

        if bytes_read.is_err() {
            let message = format!(
                "connection error while trying to read response: {:?}",
                bytes_read.err()
            );
            error!("{}", message);
            return Err(Error::msg(message));
        } else {
            let bytes_read = bytes_read.unwrap();
            info!("Read {} bytes", bytes_read);
            match std::str::from_utf8(&buf[0..bytes_read]) {
                std::result::Result::Ok(body_string) => info!(
                    "Response body (truncated to {} bytes): {:?}",
                    buf.len(),
                    body_string
                ),
                Err(e) => error!("Error decoding response body: {}", e),
            };

            // Drain the remaining response bytes
            while response.read(&mut buf).unwrap_or(0) > 0 {}
        }
        Ok(())
    }
}

#[derive(Serialize)]
#[warn(non_snake_case)]
struct RequestAlert {
    macAddress: String,
}

impl RequestAlert {
    pub fn new(mac_address: String) -> RequestAlert {
        RequestAlert {
            macAddress: mac_address,
        }
    }
}
