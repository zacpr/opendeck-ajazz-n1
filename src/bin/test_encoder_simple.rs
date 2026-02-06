/// Simple encoder test - traces state vector initialization
use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    error::MirajazzError,
    types::DeviceInput,
};

const N1_KEY_COUNT: usize = 18;

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
    match input {
        1..=18 => read_button_press_n1(input, state),
        30 => Ok(DeviceInput::EncoderStateChange(vec![state != 0, false, false])),
        31 => Ok(DeviceInput::EncoderStateChange(vec![false, state != 0, false])),
        35 => Ok(DeviceInput::EncoderStateChange(vec![false, false, state != 0])),
        50 => Ok(DeviceInput::EncoderTwist(vec![0, 0, -1])),
        51 => Ok(DeviceInput::EncoderTwist(vec![0, 0, 1])),
        _ => Err(MirajazzError::BadData),
    }
}

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("N1 Encoder Test\n");
    
    let devices = list_devices(&[DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID)]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("Found: {}", dev_info.name);
    
    // Test with 3 encoders
    println!("Connecting with 3 encoders...");
    let device = Device::connect(&dev_info, 3, 18, 3).await?;
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!("Connected!\n");
    
    let reader = device.get_reader(process_input_n1);
    
    // Simulate pressing right face button (input 31)
    println!("=== Simulating right face button press (input 31, state=1) ===");
    let input = process_input_n1(31, 1)?;
    println!("Generated DeviceInput: {:?}", input);
    
    // Read actual events from device
    println!("\n=== Now waiting for real events ===");
    println!("Press right face button now...\n");
    
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < 5 {
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            reader.read(None)
        ).await {
            Ok(Ok(updates)) => {
                for update in updates {
                    match update {
                        DeviceStateUpdate::EncoderDown(enc) => {
                            println!("✅ ENCODER DOWN: encoder={}", enc);
                        }
                        DeviceStateUpdate::EncoderUp(enc) => {
                            println!("✅ ENCODER UP: encoder={}", enc);
                        }
                        DeviceStateUpdate::EncoderTwist(enc, val) => {
                            println!("✅ ENCODER TWIST: encoder={} value={}", enc, val);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    
    println!("\nTest complete.");
    Ok(())
}
