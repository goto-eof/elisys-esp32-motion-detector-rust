// rename the file in config.rs
// customize your settings by editing this variables
// ------------------------------------------------------------------
// wifi name
pub const WIFI_SSID: &str = "wifi name";
// wifi password
pub const WIFI_PASS: &str = "wifi password";
// endpoint that is used to send an alert after a movement detection
pub const ALERT_URL: &str = "http://server_url:8080/alert";
// endpoint on which the server is informed that the device is alive
pub const I_AM_ALIVE_URL: &str = "http://server_url:8080/IAmAlive";
// time interval between is alive requests
pub const I_AM_ALIVE_SECONDS: u64 = 30;
