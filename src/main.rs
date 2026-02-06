use device::{handle_error, handle_set_image};
use mirajazz::device::Device;
use std::{collections::HashMap, process::exit, sync::LazyLock};
use tokio::sync::{Mutex, RwLock};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use watcher::watcher_task;

#[cfg(not(target_os = "windows"))]
use tokio::signal::unix::{SignalKind, signal};

mod device;
mod inputs;
mod mappings;
mod watcher;

pub static DEVICES: LazyLock<RwLock<HashMap<String, Device>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
pub static TOKENS: LazyLock<RwLock<HashMap<String, CancellationToken>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));
pub static TRACKER: LazyLock<Mutex<TaskTracker>> = LazyLock::new(|| Mutex::new(TaskTracker::new()));

use openaction::global_events::{
    GlobalEventHandler, SetBrightnessEvent, SetImageEvent,
};
use openaction::OpenActionResult;
use openaction::async_trait;

struct GlobalEventHandlerImpl {}

#[async_trait]
impl GlobalEventHandler for GlobalEventHandlerImpl {
    async fn plugin_ready(&self) -> OpenActionResult<()> {
        let tracker = TRACKER.lock().await.clone();

        let token = CancellationToken::new();
        tracker.spawn(watcher_task(token.clone()));

        TOKENS
            .write()
            .await
            .insert("_watcher_task".to_string(), token);

        log::info!("Plugin initialized");

        Ok(())
    }

    async fn device_plugin_set_image(
        &self,
        event: SetImageEvent,
    ) -> OpenActionResult<()> {
        log::debug!("Asked to set image: {:#?}", event);

        // Skip knob images
        if event.controller.as_deref() == Some("Encoder") {
            log::debug!("Looks like a knob, no need to set image");
            return Ok(());
        }

        let id = event.device.clone();

        if let Some(device) = DEVICES.read().await.get(&event.device) {
            handle_set_image(device, event)
                .await
                .map_err(async |err| handle_error(&id, err).await)
                .ok();
        } else {
            log::error!("Received event for unknown device: {}", event.device);
        }

        Ok(())
    }

    async fn device_plugin_set_brightness(
        &self,
        event: SetBrightnessEvent,
    ) -> OpenActionResult<()> {
        log::debug!("Asked to set brightness: {:#?}", event);

        let id = event.device.clone();

        if let Some(device) = DEVICES.read().await.get(&event.device) {
            device
                .set_brightness(event.brightness)
                .await
                .map_err(async |err| handle_error(&id, err).await)
                .ok();
        } else {
            log::error!("Received event for unknown device: {}", event.device);
        }

        Ok(())
    }
}

async fn shutdown() {
    let tokens = TOKENS.write().await;

    for (_, token) in tokens.iter() {
        token.cancel();
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn sigterm() -> Result<(), Box<dyn std::error::Error>> {
    let mut sig = signal(SignalKind::terminate())?;

    sig.recv().await;

    Ok(())
}

#[cfg(target_os = "windows")]
async fn sigterm() -> Result<(), Box<dyn std::error::Error>> {
    // Future that would never resolve, so select only acts on OpenDeck connection loss
    std::future::pending::<()>().await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    simplelog::TermLogger::init(
        simplelog::LevelFilter::Info,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Never,
    )
    .unwrap();

    // Set the global event handler (must be static)
    static HANDLER: GlobalEventHandlerImpl = GlobalEventHandlerImpl {};
    openaction::global_events::set_global_event_handler(&HANDLER);

    tokio::select! {
        result = openaction::run(std::env::args().collect()) => {
            if let Err(e) = result {
                log::error!("OpenAction error: {}", e);
            }
        }
        _ = sigterm() => {}
    }

    log::info!("Shutting down");

    shutdown().await;

    let tracker = TRACKER.lock().await.clone();

    log::info!("Waiting for tasks to finish");

    tracker.close();
    tracker.wait().await;

    log::info!("Tasks are finished, exiting now");

    Ok(())
}
