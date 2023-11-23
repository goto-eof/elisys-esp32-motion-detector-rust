use serde::Serialize;

#[derive(Serialize)]
#[warn(non_snake_case)]
pub struct RequestAlert {
    macAddress: String,
}

impl RequestAlert {
    pub fn new(mac_address: String) -> RequestAlert {
        RequestAlert {
            macAddress: mac_address,
        }
    }
}
