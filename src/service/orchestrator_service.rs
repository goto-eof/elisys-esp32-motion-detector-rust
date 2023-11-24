use std::time::Instant;

use super::{
    client_service::{self},
    peripheral_service::PeripheralService,
};
use crate::{config::config, util::thread_util};
use log::info;

pub fn orchestrate() {
    let mut peripheral_service = PeripheralService::new(config::WIFI_SSID, config::WIFI_PASS);
    let client_service =
        client_service::ClientService::new(config::ALERT_URL, config::I_AM_ALIVE_URL);

    let mut detection = false;
    let mut timer: u64 = 0;
    peripheral_service.led_blink_1_time_long();
    let start = Instant::now();
    let mac_address = peripheral_service.get_mac_address();

    loop {
        let duration = start.elapsed();
        if duration.as_secs() % config::I_AM_ALIVE_INTERVAL_SECONDS == 0
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
            info!("MOVEMENT DETECTED");
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
