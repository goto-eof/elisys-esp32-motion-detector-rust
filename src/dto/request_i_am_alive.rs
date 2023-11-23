use serde::Serialize;

#[derive(Serialize)]
#[warn(non_snake_case)]
pub struct RequestIAmAlive {
    macAddress: String,
}

// perhaps I will need to add new fields
impl RequestIAmAlive {
    pub fn new(mac_address: String) -> RequestIAmAlive {
        RequestIAmAlive {
            macAddress: mac_address,
        }
    }
}
