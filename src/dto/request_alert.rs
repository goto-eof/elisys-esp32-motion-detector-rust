use serde::Serialize;

#[derive(Serialize)]
#[warn(non_snake_case)]
pub struct RequestAlert {
    #[serde(rename = "macAddress")]
    mac_address: String,
}

impl RequestAlert {
    pub fn new(mac_address: String) -> RequestAlert {
        RequestAlert { mac_address }
    }
}
