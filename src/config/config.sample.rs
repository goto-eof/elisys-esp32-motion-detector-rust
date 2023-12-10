// rename the file in config.rs
// customize your settings by editing this variables
// ------------------------------------------------------------------
// wifi name
pub const WIFI_SSID: &str = "wifi name";
// wifi password
pub const WIFI_PASS: &str = "wifi password";
// endpoint that is used to send an alert after a movement detection
pub const DEFAULT_ALERT_URL: &str = "http://server_url:8080/alert";
// endpoint on which the server is informed that the device is alive
pub const DEFAULT_I_AM_ALIVE_URL: &str = "http://server_url:8080/IAmAlive";
// time interval between is alive requests
pub const DEFAULT_I_AM_ALIVE_INTERVAL_SECONDS: u64 = 30;
// endpoint for configuration download
pub const CONFIGURATION_URL: &str = "http://server_url:8080/api/v1/motion-sensor/configuration";
// if enabled, if cannot download configuration then will terminate the application
pub const IS_REMOTE_CONFIGURATION_MANDATORY: bool = false;
// This is the default crontab value if server value is wrong
pub const DEFAULT_CRONTAB: &str =
    "0-59   0-59   0-23     1-31       Jan-Dec  Mon,Tue,Wed,Thu,Fri,Sat,Sun  2023-2100";
// Device registration endpoint
pub const REGISTER_DEVICE_URL: &str = "http://192.168.1.102:8080/api/v1/device/register";
// Device name
pub const DEVICE_NAME: &str = "Motion Detector";
// Device description
pub const DEVICE_DESCRIPTION: &str = "Motion Detector Device";
// Device type
pub const DEVICE_TYPE: &str = "MotionDetector";
