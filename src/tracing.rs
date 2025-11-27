use std::fs;
use tracing_subscriber::{Layer, filter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::Result;
use crate::error::Error;

pub struct Tracing {}

impl Tracing {
    pub fn new() -> Result<()> {
        let file_path = dirs::config_dir()
            .map(|mut path| {
                path.push(env!("CARGO_PKG_NAME")); // append the package name
                path.push("log");
                path
            })
            .unwrap();

        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        let file_layer = tracing_subscriber::fmt::layer().compact().with_writer(
            fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(&file_path)?,
        );

        tracing_subscriber::registry()
            .with(
                file_layer
                    .with_ansi(true)
                    .with_target(true)
                    .with_filter(filter::LevelFilter::INFO),
            )
            .try_init()
            .map_err(|e| Error::Unknown { msg: e.to_string() })?;

        Ok(())
    }
}
