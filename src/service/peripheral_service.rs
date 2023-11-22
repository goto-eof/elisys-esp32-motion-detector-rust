use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::{
    gpio::{Gpio15, Gpio4, Gpio5, Input, Output, PinDriver},
    peripherals::Peripherals,
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi, WifiDeviceId},
};
use log::info;

use crate::util::thread_util;

const TIME_SHORT: u64 = 20;
const TIME_LONG: u64 = 1000;

pub struct PeripheralService {
    led: PinDriver<'static, Gpio5, Output>,
    buzzer: PinDriver<'static, Gpio15, Output>,
    sensor: PinDriver<'static, Gpio4, Input>,
    wifi: BlockingWifi<EspWifi<'static>>,
    wifi_ssid: String,
    wifi_password: String,
}

impl PeripheralService {
    pub fn new(wifi_ssid: &str, wifi_password: &str) -> Self {
        let peripherals = Peripherals::take().unwrap();
        let led = PinDriver::output(peripherals.pins.gpio5).unwrap();
        let sensor = PinDriver::input(peripherals.pins.gpio4).unwrap();
        let buzzer = PinDriver::output(peripherals.pins.gpio15).unwrap();

        let sys_loop = EspSystemEventLoop::take().unwrap();
        let nvs = EspDefaultNvsPartition::take().unwrap();

        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs)).unwrap(),
            sys_loop,
        )
        .unwrap();
        let mut wifi_connection = connect_wifi(&mut wifi, wifi_ssid, wifi_password);
        while wifi_connection.is_err() {
            thread_util::sleep_time(TIME_LONG);
            wifi_connection = connect_wifi(&mut wifi, wifi_ssid, wifi_password);
        }

        let peripheral_service = PeripheralService {
            led,
            buzzer,
            sensor,
            wifi,
            wifi_ssid: wifi_ssid.to_owned(),
            wifi_password: wifi_password.to_owned(),
        };
        return peripheral_service;
    }

    pub fn retry_wifi_connection_if_necessary_and_return_status(&mut self) -> bool {
        if !self.wifi.is_connected().unwrap() {
            if connect_wifi(&mut self.wifi, &self.wifi_ssid, &self.wifi_password).is_err() {
                self.led_blink_3_time_long();
                return false;
            }
        }
        return true;
    }

    pub fn buzz_1_time_short(&mut self) {
        self.buzzer.set_high().unwrap();
        thread_util::sleep_time(TIME_SHORT);
        self.buzzer.set_low().unwrap();
        thread_util::sleep_time(TIME_SHORT);
    }

    pub fn power_off_output_devices(&mut self) {
        self.led.set_low().unwrap();
        self.buzzer.set_low().unwrap();
    }

    pub fn is_motion_detected(&self) -> bool {
        self.sensor.is_high()
    }

    pub fn led_blink_3_time_short(&mut self) {
        self.led_blink_1_time(TIME_SHORT);
        self.led_blink_1_time(TIME_SHORT);
        self.led_blink_1_time(TIME_SHORT);
    }

    pub fn led_blink_3_time_long(&mut self) {
        self.led_blink_1_time(TIME_LONG);
        self.led_blink_1_time(TIME_LONG);
        self.led_blink_1_time(TIME_LONG);
    }

    pub fn led_blink_2_time_short(&mut self) {
        self.led_blink_1_time(TIME_SHORT);
        self.led_blink_1_time(TIME_SHORT);
    }

    pub fn led_blink_2_time_long(&mut self) {
        self.led_blink_1_time(TIME_LONG);
        self.led_blink_1_time(TIME_LONG);
    }

    pub fn led_blink_1_time_short(&mut self) {
        self.led_blink_1_time(TIME_SHORT);
    }

    pub fn led_blink_1_time_long(&mut self) {
        self.led_blink_1_time(TIME_LONG);
    }

    pub fn get_mac_address(&self) -> String {
        let mav = &self
            .wifi
            .wifi()
            .driver()
            .get_mac(WifiDeviceId::Sta)
            .unwrap();
        let mac_address_obj =
            macaddr::MacAddr6::new(mav[0], mav[1], mav[2], mav[3], mav[4], mav[5]);
        let mac_address_value = mac_address_obj.to_string();
        info!("MAC_ADDRESS: {:?}", mac_address_value);
        mac_address_value
    }

    fn led_blink_1_time(&mut self, time: u64) {
        self.led.set_high().unwrap();
        thread_util::sleep_time(time);
        self.led.set_low().unwrap();
        thread_util::sleep_time(time);
    }
}

fn connect_wifi(
    wifi: &mut BlockingWifi<EspWifi<'static>>,
    ssid: &str,
    password: &str,
) -> anyhow::Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(ClientConfiguration {
        ssid: ssid.into(),
        bssid: None,
        auth_method: AuthMethod::WPA2Personal,
        password: password.into(),
        channel: None,
    });
    info!("Connecting to SSID: {}", ssid);
    wifi.set_configuration(&wifi_configuration)?;

    wifi.start()?;
    info!("Wifi started");

    wifi.connect()?;
    info!("Wifi connected: {}", ssid);

    wifi.wait_netif_up()?;
    info!("Wifi netif up");

    Ok(())
}
