/// Test that OpenAction device_plugin functions work correctly
/// This simulates what the main plugin does but with detailed error checking

use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    error::MirajazzError,
    types::DeviceInput,
};
use std::sync::Mutex;

const N1_KEY_COUNT: usize = 18;
static ENCODER_STATES: Mutex<[bool; 3]> = Mutex::new([false, false, false]);

fn device_to_opendeck_n1(key: usize) -> usize {
    match key {
        16 => 0, 17 => 1, 18 => 2,
        1..=15 => key + 2,
        _ => key.saturating_sub(1),
    }
}

fn read_button_press_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; N1_KEY_COUNT + 1]);
    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(&button_states, N1_KEY_COUNT)));
    }
    let pressed_index: usize = device_to_opendeck_n1(input as usize);
    if pressed_index < N1_KEY_COUNT {
        button_states[pressed_index + 1] = state;
    }
    Ok(DeviceInput::ButtonStateChange(read_button_states(&button_states, N1_KEY_COUNT)))
}

fn read_button_states(states: &[u8], key_count: usize) -> Vec<bool> {
    let mut bools = vec![];
    for i in 0..key_count {
        bools.push(states.get(i + 1).copied().unwrap_or(0) != 0);
    }
    bools
}

pub fn process_input_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    match input {
        1..=18 => read_button_press_n1(input, state),
        30 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            states[0] = state != 0;
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        31 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            states[1] = state != 0;
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        35 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            states[2] = state != 0;
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        50 => Ok(DeviceInput::EncoderTwist(vec![0, 0, -1])),
        51 => Ok(DeviceInput::EncoderTwist(vec![0, 0, 1])),
        _ => Err(MirajazzError::BadData),
    }
}

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  OpenAction Event Sending Test");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let devices = list_devices(&[DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID)]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("Found: {}", dev_info.name);
    
    let device = Device::connect(&dev_info, 3, 18, 3).await?;
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!("✅ Connected!\n");
    
    // Generate a fake device ID
    let device_id = "N1-TEST123".to_string();
    
    println!("Registering device with OpenAction...");
    println!("  ID: {}", device_id);
    println!("  Rows: 6, Cols: 3, Encoders: 3");
    
    match openaction::device_plugin::register_device(
        device_id.clone(),
        "Ajazz N1".to_string(),
        6,  // rows
        3,  // columns
        3,  // encoders
        0,  // type
    ).await {
        Ok(_) => println!("✅ Device registered\n"),
        Err(e) => {
            println!("❌ Failed to register device: {}", e);
            println!("\n⚠️  OpenAction not connected - this is expected if OpenDeck isn't running.");
            println!("The plugin would normally be started by OpenDeck with CLI arguments.\n");
            return Ok(());
        }
    }
    
    let reader = device.get_reader(process_input_n1);
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Listening for events and sending to OpenAction...");
    println!("═══════════════════════════════════════════════════════════════\n");
    println!("Press buttons on your N1 now.\n");
    
    loop {
        match reader.read(None).await {
            Ok(updates) => {
                for update in updates {
                    let result = match &update {
                        DeviceStateUpdate::EncoderDown(enc) => {
                            println!("🎯 Sending EncoderDown(encoder={})...", enc);
                            openaction::device_plugin::encoder_down(device_id.clone(), *enc).await
                        }
                        DeviceStateUpdate::EncoderUp(enc) => {
                            println!("🎯 Sending EncoderUp(encoder={})...", enc);
                            openaction::device_plugin::encoder_up(device_id.clone(), *enc).await
                        }
                        DeviceStateUpdate::EncoderTwist(enc, val) => {
                            println!("🎯 Sending EncoderChange(encoder={}, val={})...", enc, val);
                            openaction::device_plugin::encoder_change(device_id.clone(), *enc, *val as i16).await
                        }
                        _ => Ok(()),
                    };
                    
                    match result {
                        Ok(_) => println!("   ✅ Success"),
                        Err(e) => println!("   ❌ Error: {}", e),
                    }
                }
            }
            Err(e) => {
                println!("❌ Read error: {:?}", e);
                break;
            }
        }
    }
    
    Ok(())
}
