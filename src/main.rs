use crate::service::orchestrator_service::orchestrate;
use anyhow::Ok;
mod config;
use esp_idf_sys::{self as _};
mod dto;
mod service;
mod util;
fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    orchestrate();

    return Ok(());
}
