use crate::dto::{
    config_request::ConfigRequest, config_response::Configuration, request_alert::RequestAlert,
    request_i_am_alive::RequestIAmAlive,
};
use anyhow::{Error, Ok};
use embedded_svc::{http::client::Client as HttpClient, io::Write, utils::io};
use esp_idf_svc::http::client::EspHttpConnection;
use esp_idf_sys as _;
use log::{error, info};

pub struct ClientService {
    alert_url: String,
    i_am_alive_url: String,
}

impl ClientService {
    pub fn new(alert_url: &str, i_am_alive_url: &str) -> ClientService {
        ClientService {
            alert_url: alert_url.to_owned(),
            i_am_alive_url: i_am_alive_url.to_owned(),
        }
    }

    pub fn send_alert(&self, mac_address: &str) -> anyhow::Result<(), anyhow::Error> {
        let client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

        let payload = serde_json::to_string(&RequestAlert::new(mac_address.to_owned())).unwrap();
        let payload = payload.as_bytes();

        info!("trying to send alertnotification...");
        let result = self.post_request(payload, client, &self.alert_url);
        info!("notification sent? {}", !result.is_err());
        return result;
    }

    pub fn send_i_am_alive(&self, mac_address: &str) -> anyhow::Result<(), anyhow::Error> {
        let client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
        let payload = serde_json::to_string(&RequestIAmAlive::new(mac_address.to_owned())).unwrap();
        let payload = payload.as_bytes();

        info!("trying to send is alive ack...");
        let result = self.post_request(payload, client, &self.i_am_alive_url);
        info!("ack sent? {}", !result.is_err());
        return result;
    }

    fn post_request(
        &self,
        payload: &[u8],
        mut client: HttpClient<EspHttpConnection>,
        url: &str,
    ) -> Result<(), Error> {
        let content_length_header = format!("{}", payload.len());
        let headers = [
            ("content-type", "application/json"),
            ("content-length", &*content_length_header),
        ];

        let request = client.post(url, &headers);

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
        info!("-> POST {}", url);
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

pub fn get_configuration(
    configuration_uri: &str,
    mac_address: &str,
) -> anyhow::Result<Configuration, anyhow::Error> {
    let client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);
    let payload = serde_json::to_string(&ConfigRequest::new(mac_address.to_owned())).unwrap();
    let payload = payload.as_bytes();

    info!("[config downloader]: trying to get remote configuration...");
    let result = post_request_config(payload, client, configuration_uri);
    info!(
        "[config downloader]: configuration retrieved with success? {}",
        !result.is_err()
    );
    return result;
}

fn post_request_config(
    payload: &[u8],
    mut client: HttpClient<EspHttpConnection>,
    url: &str,
) -> Result<Configuration, Error> {
    let content_length_header = format!("{}", payload.len());
    let headers = [
        ("content-type", "application/json"),
        ("content-length", &*content_length_header),
    ];

    let request = client.post(url, &headers);

    if request.is_err() {
        let message = format!("[config downloader]: connection error: {:?}", request.err());
        error!("{}", message);
        return Err(Error::msg(message));
    }
    let mut request = request.unwrap();

    if request.write_all(payload).is_err() {
        let message = format!("[config downloader]: connection error while trying to write all");
        error!("{}", message);
        return Err(Error::msg(message));
    }
    if request.flush().is_err() {
        let message = format!("[config downloader]: connection error while trying to flush");
        error!("{}", message);
        return Err(Error::msg(message));
    }
    info!("-> GET {}", url);
    let response = request.submit();
    if response.is_err() {
        let message =
            format!("[config downloader]: connection error while trying to read response");
        error!("{}", message);
        return Err(Error::msg(message));
    }
    let mut response = response.unwrap();

    let status = response.status();
    info!("<- {}", status);
    let mut buf = [0u8; 4086];
    let bytes_read = io::try_read_full(&mut response, &mut buf).map_err(|e| e.0);

    if bytes_read.is_err() {
        let message = format!(
            "[config downloader]: connection error while trying to read response: {:?}",
            bytes_read.err()
        );
        error!("{}", message);
        return Err(Error::msg(message));
    } else {
        let bytes_read = bytes_read.unwrap();

        match std::str::from_utf8(&buf[0..bytes_read]) {
            std::result::Result::Ok(body_string) => {
                let configuration: Result<Configuration, serde_json::Error> =
                    serde_json::from_str(body_string);
                info!("{:?}", configuration);

                if configuration.is_err() {
                    let err = configuration.err().unwrap();
                    error!(
                        "[config downloader]: error while trying to parse the configuration response: {}",
                        &err
                    );
                    return Err(err.into());
                }

                let configuration = configuration.unwrap();
                info!(
                    "[config downloader]: Remote configuration loaded successfully: {:?}",
                    configuration
                );

                info!(
                    "[config downloader]: Response body (truncated to {} bytes): {:?}",
                    buf.len(),
                    body_string
                );
                return Ok(configuration);
            }
            Err(e) => {
                error!("[config downloader]: Error decoding response body: {}", e);
                return Err(e.into());
            }
        };
    }
}
