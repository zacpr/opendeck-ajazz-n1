/// Debug encoder events for N1
/// Shows exactly what inputs are received and what events are generated

use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    error::MirajazzError,
    types::DeviceInput,
};

const N1_KEY_COUNT: usize = 18;
const N1_FACE_BUTTON_LEFT: u8 = 30;
const N1_FACE_BUTTON_RIGHT: u8 = 31;
const N1_DIAL_PRESS: u8 = 35;
const N1_DIAL_ROTATE_CCW: u8 = 50;
const N1_DIAL_ROTATE_CW: u8 = 51;

fn device_to_opendeck_n1(key: usize) -> usize {
    match key {
        16 => 0, 17 => 1, 18 => 2,
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
        return Ok(DeviceInput::ButtonStateChange(read_button_states(&button_states, N1_KEY_COUNT)));
    }
    let pressed_index: usize = device_to_opendeck_n1(input as usize);
    if pressed_index < N1_KEY_COUNT {
        button_states[pressed_index + 1] = state;
    }
    Ok(DeviceInput::ButtonStateChange(read_button_states(&button_states, N1_KEY_COUNT)))
}

pub fn process_input_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let result = match input {
        1..=18 => read_button_press_n1(input, state),
        N1_FACE_BUTTON_LEFT => {
            println!("  → Mapping to EncoderStateChange([true, false, false])");
            Ok(DeviceInput::EncoderStateChange(vec![
                state != 0,
                false,
                false,
            ]))
        }
        N1_FACE_BUTTON_RIGHT => {
            println!("  → Mapping to EncoderStateChange([false, true, false])");
            Ok(DeviceInput::EncoderStateChange(vec![
                false,
                state != 0,
                false,
            ]))
        }
        N1_DIAL_PRESS => {
            println!("  → Mapping to EncoderStateChange([false, false, true])");
            Ok(DeviceInput::EncoderStateChange(vec![
                false,
                false,
                state != 0,
            ]))
        }
        N1_DIAL_ROTATE_CCW => {
            println!("  → Mapping to EncoderTwist([0, 0, -1])");
            Ok(DeviceInput::EncoderTwist(vec![0, 0, -1]))
        }
        N1_DIAL_ROTATE_CW => {
            println!("  → Mapping to EncoderTwist([0, 0, 1])");
            Ok(DeviceInput::EncoderTwist(vec![0, 0, 1]))
        }
        _ => {
            println!("  → Unknown input!");
            Err(MirajazzError::BadData)
        }
    };
    result
}

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  N1 Encoder Debug Tool");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let devices = list_devices(&[DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID)]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("Found: {}", dev_info.name);
    
    // Connect with 3 encoders
    let device = Device::connect(&dev_info, 3, 18, 3).await?;
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("✅ Connected with 3 encoders");
    println!("\nListening for encoder events...\n");
    println!("Try these actions:");
    println!("  • Left face button (should show EncoderStateChange)");
    println!("  • Right face button (should show EncoderStateChange)");
    println!("  • Dial press (should show EncoderStateChange)");
    println!("  • Dial rotate (should show EncoderTwist)");
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
                            println!("🔄 [ENCODER TWIST] encoder={} value={}", enc, val);
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
