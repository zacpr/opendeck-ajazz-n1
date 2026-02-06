/// Test encoder input processing for N1
/// This tests that our input processing correctly generates encoder events

use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    error::MirajazzError,
};

// Copy of process_input_n1 from inputs.rs for testing
use mirajazz::types::DeviceInput;

const AKP153_KEY_COUNT: usize = 18;
const N1_KEY_COUNT: usize = 18;
const N1_FACE_BUTTON_LEFT: u8 = 30;
const N1_FACE_BUTTON_RIGHT: u8 = 31;
const N1_DIAL_PRESS: u8 = 35;
const N1_DIAL_ROTATE_CCW: u8 = 50;
const N1_DIAL_ROTATE_CW: u8 = 51;

fn device_to_opendeck_n1(key: usize) -> usize {
    match key {
        16 => 0,
        17 => 1,
        18 => 2,
        1..=15 => key + 2,
        _ => key.saturating_sub(1),
    }
}

fn read_button_states(states: &[u8], key_count: usize) -> Vec<bool> {
    let mut bools = vec![];
    for i in 0..key_count {
        bools.push(states.get(i + 1).copied().unwrap_or(0) != 0);
    }
    bools
}

fn read_button_press_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; N1_KEY_COUNT + 1]);

    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(
            &button_states,
            N1_KEY_COUNT,
        )));
    }

    let pressed_index: usize = device_to_opendeck_n1(input as usize);

    if pressed_index < N1_KEY_COUNT {
        button_states[pressed_index + 1] = state;
    }

    Ok(DeviceInput::ButtonStateChange(read_button_states(
        &button_states,
        N1_KEY_COUNT,
    )))
}

fn read_face_button_press(_input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let encoder_pressed = state != 0;
    Ok(DeviceInput::EncoderStateChange(vec![encoder_pressed]))
}

fn read_dial_press(state: u8) -> Result<DeviceInput, MirajazzError> {
    let encoder_pressed = state != 0;
    Ok(DeviceInput::EncoderStateChange(vec![encoder_pressed]))
}

pub fn process_input_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    match input {
        1..=18 => read_button_press_n1(input, state),
        N1_FACE_BUTTON_LEFT | N1_FACE_BUTTON_RIGHT => {
            read_face_button_press(input, state)
        }
        N1_DIAL_PRESS => {
            read_dial_press(state)
        }
        N1_DIAL_ROTATE_CCW => {
            Ok(DeviceInput::EncoderTwist(vec![-1]))
        }
        N1_DIAL_ROTATE_CW => {
            Ok(DeviceInput::EncoderTwist(vec![1]))
        }
        _ => {
            Err(MirajazzError::BadData)
        }
    }
}

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  N1 Encoder Test Tool");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let devices = list_devices(&[DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID)]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("Found: {}\n", dev_info.name);
    
    // Connect with 1 encoder
    let device = Device::connect(&dev_info, 3, 18, 1).await?;
    
    // Set software mode
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("✅ Connected in software mode 3");
    println!("Listening for encoder events...\n");
    println!("Try these actions:");
    println!("  • Press left face button (should show EncoderDown/EncoderUp)");
    println!("  • Press right face button (should show EncoderDown/EncoderUp)");
    println!("  • Press dial down (should show EncoderDown/EncoderUp)");
    println!("  • Rotate dial left/right (should show EncoderTwist)");
    println!("\nPress Ctrl+C to exit\n");
    
    let reader = device.get_reader(process_input_n1);
    
    loop {
        match reader.read(None).await {
            Ok(updates) => {
                for update in updates {
                    match update {
                        DeviceStateUpdate::ButtonDown(key) => {
                            println!("📥 [BUTTON DOWN] key={}", key);
                        }
                        DeviceStateUpdate::ButtonUp(key) => {
                            println!("📤 [BUTTON UP]   key={}", key);
                        }
                        DeviceStateUpdate::EncoderDown(enc) => {
                            println!("🔘 [ENCODER DOWN] encoder={}", enc);
                        }
                        DeviceStateUpdate::EncoderUp(enc) => {
                            println!("🔘 [ENCODER UP]   encoder={}", enc);
                        }
                        DeviceStateUpdate::EncoderTwist(enc, val) => {
                            let dir = if val > 0 { "→" } else { "←" };
                            println!("🔄 [ENCODER TWIST] encoder={} value={} {}", enc, val, dir);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Error: {:?}", e);
                break;
            }
        }
    }
    
    Ok(())
}
