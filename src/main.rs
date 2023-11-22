use anyhow::Ok;
use esp_idf_sys as _;
mod service;
mod util;
use crate::service::orchestrator_service::orchestrate;
mod config;
fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    orchestrate();

    return Ok(());
}
