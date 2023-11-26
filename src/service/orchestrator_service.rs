use super::{
    client_service::{self, get_configuration},
    peripheral_service::PeripheralService,
};
use crate::{
    config::config::{self, CONFIGURATION_URL},
    dto::config_response::Configuration,
    service::client_service::get_default_configuration,
    util::thread_util,
};
use chrono::Utc;
use core::result::Result::Ok as StandardOk;
use cron::Schedule;
use esp_idf_svc::sntp;
use esp_idf_svc::sntp::SyncStatus;
use log::{error, info};
use std::str::FromStr;
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
            peripheral_service.led_blink_3_time_short();
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

    synchronize_clock();

    let mut schedule = Schedule::from_str(&config::DEFAULT_CRONTAB).unwrap();
    let schedule_result = Schedule::from_str(&configuration.crontab);
    if schedule_result.is_err() {
        error!("invalid crontab value");
    } else {
        schedule = schedule_result.unwrap();
    }
    let mut next_date_time = calculate_next_date_time(&schedule);

    info!("ESP32 TIME: {:?}", Utc::now());
    let mut backup_date_time = "".to_owned();
    loop {
        send_i_am_alive_if_necessary(
            start,
            &configuration,
            &mut timer,
            &client_service,
            &mac_address,
            &mut peripheral_service,
        );

        let now = Utc::now();
        let now_date_time = now.to_rfc2822();
        if now_date_time.eq(&next_date_time) || now_date_time == backup_date_time {
            backup_date_time = now_date_time.clone();
            next_date_time = calculate_next_date_time(&schedule);

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
        }

        thread_util::sleep_time(20);
    }
}

fn send_i_am_alive_if_necessary(
    start: Instant,
    configuration: &Configuration,
    timer: &mut u64,
    client_service: &client_service::ClientService,
    mac_address: &String,
    peripheral_service: &mut PeripheralService,
) {
    let duration = start.elapsed();
    if duration.as_secs() % configuration.i_am_alive_interval_seconds == 0
        && *timer != duration.as_secs()
    {
        if client_service.send_i_am_alive(mac_address).is_err() {
            log::error!("failed to send is alive ack");
            peripheral_service.led_blink_2_time_short();
        }
        *timer = duration.as_secs();
    }
}

fn synchronize_clock() {
    let sntp = sntp::EspSntp::new_default().unwrap();
    info!("SNTP initialized, waiting for status!");
    while sntp.get_sync_status() != SyncStatus::Completed {}
}

fn calculate_next_date_time(schedule: &Schedule) -> String {
    let next_date_time = schedule
        .upcoming(Utc)
        .take(1)
        .into_iter()
        .last()
        .unwrap()
        .to_rfc2822();
    next_date_time
}
