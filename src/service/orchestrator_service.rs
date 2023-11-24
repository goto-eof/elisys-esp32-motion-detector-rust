use super::{
    client_service::{self, get_configuration},
    peripheral_service::PeripheralService,
};
use crate::{
    config::config::{
        self, CONFIGURATION_URL, DEFAULT_ALERT_URL, DEFAULT_I_AM_ALIVE_INTERVAL_SECONDS,
        DEFAULT_I_AM_ALIVE_URL,
    },
    dto::config_response::Configuration,
    util::thread_util,
};
use anyhow::Error;
use core::result::Result::Ok as StandardOk;
use log::{error, info};
use std::time::Instant;

pub fn orchestrate() {
    let mut peripheral_service = PeripheralService::new(config::WIFI_SSID, config::WIFI_PASS);
    let mac_address = peripheral_service.get_mac_address();

    let configuration: Result<Configuration, anyhow::Error> =
        get_configuration(CONFIGURATION_URL, &mac_address);

    let configuration = match configuration {
        Err(e) => Some({
            if config::IS_REMOTE_CONFIGURATION_MANDATORY {
                error!("Could not download the remote configuration. REMOTE CONFIGURATION DOWNLOAD IS MANDATORY. Terminating the application...");
                return;
            }
            get_default_configuration(e)
        }),
        StandardOk(config) => Some(config),
    };

    let configuration = configuration.unwrap();
    info!(
        "{}",
        format!("configuration (remote || default): {:?}", &configuration)
    );
    let client_service = client_service::ClientService::new(
        &configuration.alert_endpoint,
        &configuration.i_am_alive_endpoint,
    );

    let mut detection = false;
    let mut timer: u64 = 0;
    peripheral_service.led_blink_1_time_long();
    let start = Instant::now();

    loop {
        let duration = start.elapsed();
        if duration.as_secs() % configuration.i_am_alive_interval_seconds == 0
            && timer != duration.as_secs()
        {
            if client_service.send_i_am_alive(&mac_address).is_err() {
                log::error!("failed to send is alive ack");
            }
            timer = duration.as_secs();
        }

        // info!(
        //     "START: motion: {}, detected: {}",
        //     peripheral_service.is_motion_detected(),
        //     detection
        // );
        if !peripheral_service.is_motion_detected() && detection {
            info!("no detection");
            detection = false;
            peripheral_service.power_off_output_devices();
        } else if peripheral_service.is_motion_detected() && !detection {
            info!("---<< MOVEMENT DETECTED >>---");
            while !peripheral_service.retry_wifi_connection_if_necessary_and_return_status() {
                peripheral_service.led_blink_3_time_long();
                thread_util::sleep_short();
            }
            if client_service.send_alert(&mac_address).is_err() {
                peripheral_service.led_blink_2_time_long();
                detection = false;
            } else {
                peripheral_service.led_blink_1_time_short();
                peripheral_service.buzz_1_time_short();
                detection = true;
            }
        }

        // info!(
        //     "END: motion: {}, detected: {}",
        //     peripheral_service.is_motion_detected(),
        //     detection
        // );
        thread_util::sleep_time(20);
    }
}

pub fn get_default_configuration(e: Error) -> Configuration {
    error!(
        "Error while trying to load configuration from remote server: {:?}",
        e
    );
    Configuration {
        alert_endpoint: DEFAULT_ALERT_URL.to_owned(),
        crontab: "* * * * *".to_owned(),
        i_am_alive_endpoint: DEFAULT_I_AM_ALIVE_URL.to_owned(),
        i_am_alive_interval_seconds: DEFAULT_I_AM_ALIVE_INTERVAL_SECONDS,
    }
}
