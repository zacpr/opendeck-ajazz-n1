use std::time::Duration;

use data_url::DataUrl;
use image::load_from_memory_with_format;
use mirajazz::{device::Device, error::MirajazzError, state::DeviceStateUpdate};
use openaction::global_events::SetImageEvent;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;

use crate::{
    DEVICES, TOKENS,
    inputs::opendeck_to_device,
    mappings::{
        CandidateDevice, Kind,
        get_image_format_for_key,
    },
};

/// Initializes a device and listens for events
pub async fn device_task(candidate: CandidateDevice, token: CancellationToken) {
    log::info!("Running device task for {:?}", candidate);

    // Wrap in a closure so we can use `?` operator
    let device = async || -> Result<Device, MirajazzError> {
        let device = connect(&candidate).await?;

        // N1 requires software mode to be set for control
        if matches!(candidate.kind, Kind::N1) {
            device.set_mode(3).await?;  // 3 = Software mode
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        device.set_brightness(50).await?;
        device.clear_all_button_images().await?;
        device.flush().await?;

        Ok(device)
    }()
    .await;

    let device: Device = match device {
        Ok(device) => device,
        Err(err) => {
            handle_error(&candidate.id, err).await;

            log::error!(
                "Had error during device init, finishing device task: {:?}",
                candidate
            );

            return;
        }
    };

    log::info!("Registering device {}", candidate.id);
    let (rows, cols) = candidate.kind.layout();
    let encoder_count = candidate.kind.encoder_count() as u8;
    log::info!("Device layout: {} rows, {} cols, {} encoders", rows, cols, encoder_count);
    
    if let Err(e) = openaction::device_plugin::register_device(
        candidate.id.clone(),
        candidate.kind.human_name(),
        rows as u8,
        cols as u8,
        encoder_count,
        0,
    ).await {
        log::error!("Failed to register device: {}", e);
        return;
    }
    log::info!("Device registered successfully with {} encoders", encoder_count);

    DEVICES.write().await.insert(candidate.id.clone(), device);

    tokio::select! {
        _ = device_events_task(&candidate) => {},
        _ = keepalive_task(&candidate) => {},
        _ = token.cancelled() => {}
    };

    log::info!("Shutting down device {:?}", candidate);

    if let Some(device) = DEVICES.read().await.get(&candidate.id) {
        device.shutdown().await.ok();
    }

    log::info!("Device task finished for {:?}", candidate);
}

/// Handles errors, returning true if should continue, returning false if an error is fatal
pub async fn handle_error(id: &String, err: MirajazzError) -> bool {
    log::error!("Device {} error: {}", id, err);

    // Some errors are not critical and can be ignored without sending disconnected event
    if matches!(err, MirajazzError::ImageError(_) | MirajazzError::BadData) {
        return true;
    }

    log::info!("Deregistering device {}", id);
    if let Err(e) = openaction::device_plugin::unregister_device(id.clone()).await {
        log::error!("Failed to unregister device: {}", e);
    }

    log::info!("Cancelling tasks for device {}", id);
    if let Some(token) = TOKENS.read().await.get(id) {
        token.cancel();
    }

    log::info!("Removing device {} from the list", id);
    DEVICES.write().await.remove(id);

    log::info!("Finished clean-up for {}", id);

    false
}

pub async fn connect(candidate: &CandidateDevice) -> Result<Device, MirajazzError> {
    let result = Device::connect(
        &candidate.dev,
        candidate.kind.protocol_version(),
        candidate.kind.key_count(),
        candidate.kind.encoder_count(),
    )
    .await;

    match result {
        Ok(device) => Ok(device),
        Err(e) => {
            log::error!("Error while connecting to device: {e}");

            Err(e)
        }
    }
}

/// Handles events from device to OpenDeck
async fn device_events_task(candidate: &CandidateDevice) -> Result<(), MirajazzError> {
    log::info!("Connecting to {} for incoming events", candidate.id);

    let process_input = crate::inputs::process_input_n1;

    let devices_lock = DEVICES.read().await;
    let reader = match devices_lock.get(&candidate.id) {
        Some(device) => device.get_reader(process_input),
        None => return Ok(()),
    };
    drop(devices_lock);

    log::info!("Connected to {} for incoming events", candidate.id);

    log::info!("Reader is ready for {}", candidate.id);

    loop {
        log::info!("Reading updates...");

        let updates = match reader.read(None).await {
            Ok(updates) => updates,
            Err(e) => {
                if !handle_error(&candidate.id, e).await {
                    break;
                }

                continue;
            }
        };

        for update in updates {
            match &update {
                DeviceStateUpdate::EncoderDown(enc) => {
                    log::info!("🎯 ENCODER DOWN: encoder={}", enc);
                }
                DeviceStateUpdate::EncoderUp(enc) => {
                    log::info!("🎯 ENCODER UP: encoder={}", enc);
                }
                DeviceStateUpdate::EncoderTwist(enc, val) => {
                    log::info!("🎯 ENCODER TWIST: encoder={} value={}", enc, val);
                }
                _ => {
                    log::info!("New update: {:#?}", update);
                }
            }

            let id = candidate.id.clone();

            let result = match update {
                DeviceStateUpdate::ButtonDown(key) => {
                    openaction::device_plugin::key_down(id, key).await
                }
                DeviceStateUpdate::ButtonUp(key) => {
                    openaction::device_plugin::key_up(id, key).await
                }
                DeviceStateUpdate::EncoderDown(encoder) => {
                    log::info!("📤 Sending encoder_down(id={}, encoder={})", id, encoder);
                    let result = openaction::device_plugin::encoder_down(id, encoder).await;
                    if let Err(ref e) = result {
                        log::error!("Failed to send encoder_down: {}", e);
                    }
                    result
                }
                DeviceStateUpdate::EncoderUp(encoder) => {
                    log::info!("📤 Sending encoder_up(id={}, encoder={})", id, encoder);
                    let result = openaction::device_plugin::encoder_up(id, encoder).await;
                    if let Err(ref e) = result {
                        log::error!("Failed to send encoder_up: {}", e);
                    }
                    result
                }
                DeviceStateUpdate::EncoderTwist(encoder, val) => {
                    log::info!("📤 Sending encoder_change(id={}, encoder={}, val={})", id, encoder, val);
                    let result = openaction::device_plugin::encoder_change(
                        id, encoder, val as i16
                    ).await;
                    if let Err(ref e) = result {
                        log::error!("Failed to send encoder_change: {}", e);
                    }
                    result
                }
            };

            if let Err(e) = result {
                log::error!("Failed to send event to OpenAction: {}", e);
            }
        }
    }

    Ok(())
}

/// Sends periodic keepalives to the device to maintain connection
async fn keepalive_task(candidate: &CandidateDevice) -> Result<(), MirajazzError> {
    let mut interval = interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        log::debug!("Sending keepalive to {}", candidate.id);

        let devices_lock = DEVICES.read().await;

        let device = match devices_lock.get(&candidate.id) {
            Some(device) => device,
            None => return Ok(()),
        };

        if let Err(e) = device.keep_alive().await {
            drop(devices_lock);
            if !handle_error(&candidate.id, e).await {
                break;
            }
        }
    }

    Ok(())
}

/// Handles different combinations of "set image" event, including clearing the specific buttons and whole device
pub async fn handle_set_image(device: &Device, evt: SetImageEvent) -> Result<(), MirajazzError> {
    // Get position from the event - it's Option<u8> in v2
    let position = evt.position;
    
    match (position, evt.image) {
        (Some(position), Some(image)) => {
            log::info!("Setting image for button {}", position);

            // OpenDeck sends image as a data url, so parse it using a library
            let url = DataUrl::process(image.as_str()).unwrap(); // Isn't expected to fail, so unwrap it is
            let (body, _fragment) = url.decode_to_vec().unwrap(); // Same here

            // Allow only image/jpeg mime for now
            if url.mime_type().subtype != "jpeg" {
                log::error!("Incorrect mime type: {}", url.mime_type());

                return Ok(()); // Not a fatal error, enough to just log it
            }

            let image = load_from_memory_with_format(body.as_slice(), image::ImageFormat::Jpeg)?;

            let kind = Kind::from_vid_pid(device.vid, device.pid).unwrap(); // Safe to unwrap here, because device is already filtered

            device
                .set_button_image(
                    opendeck_to_device(position),
                    get_image_format_for_key(&kind, position),
                    image,
                )
                .await?;
            device.flush().await?;
            
            // Small delay for N1 to ensure device processes the image
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        (Some(position), None) => {
            device
                .clear_button_image(opendeck_to_device(position))
                .await?;
            device.flush().await?;
        }
        (None, None) => {
            device.clear_all_button_images().await?;
            device.flush().await?;
        }
        _ => {}
    }

    Ok(())
}
