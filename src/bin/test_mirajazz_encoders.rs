/// Test mirajazz encoder handling with full tracing
use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    error::MirajazzError,
    types::DeviceInput,
};
use std::sync::Mutex;

const N1_KEY_COUNT: usize = 18;

// Track encoder states like the main plugin does
static ENCODER_STATES: Mutex<[bool; 3]> = Mutex::new([false, false, false]);

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
        30 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            states[0] = state != 0;
            println!("  [INPUT 30] Left face btn state={} → enc_states={:?}", state, *states);
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        31 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            states[1] = state != 0;
            println!("  [INPUT 31] Right face btn state={} → enc_states={:?}", state, *states);
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        35 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            states[2] = state != 0;
            println!("  [INPUT 35] Dial press state={} → enc_states={:?}", state, *states);
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        50 => {
            println!("  [INPUT 50] Dial rotate CCW");
            Ok(DeviceInput::EncoderTwist(vec![0, 0, -1]))
        }
        51 => {
            println!("  [INPUT 51] Dial rotate CW");
            Ok(DeviceInput::EncoderTwist(vec![0, 0, 1]))
        }
        _ => Err(MirajazzError::BadData),
    };
    
    if let Err(ref e) = result {
        println!("  [ERROR] input={} error={:?}", input, e);
    }
    
    result
}

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Mirajazz Encoder Test with Full Tracing");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let devices = list_devices(&[DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID)]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("Found: {}\n", dev_info.name);
    
    // Connect with 3 encoders
    println!("Connecting with protocol=3, keys=18, encoders=3...");
    let device = Device::connect(&dev_info, 3, 18, 3).await?;
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!("✅ Connected!\n");
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  TEST ALL BUTTONS NOW:");
    println!("═══════════════════════════════════════════════════════════════");
    println!("\n👉 Press each button and watch for events below:\n");
    
    let reader = device.get_reader(process_input_n1);
    
    loop {
        match reader.read(None).await {
            Ok(updates) => {
                if !updates.is_empty() {
                    println!("\n📤 Generated {} update(s):", updates.len());
                    for (i, update) in updates.iter().enumerate() {
                        match update {
                            DeviceStateUpdate::EncoderDown(enc) => {
                                println!("   [{}] ✅ EncoderDown(encoder={})", i, enc);
                            }
                            DeviceStateUpdate::EncoderUp(enc) => {
                                println!("   [{}] ✅ EncoderUp(encoder={})", i, enc);
                            }
                            DeviceStateUpdate::EncoderTwist(enc, val) => {
                                let dir = if *val > 0 { "CW" } else { "CCW" };
                                println!("   [{}] ✅ EncoderTwist(encoder={}, val={}, dir={})", i, enc, val, dir);
                            }
                            _ => {
                                println!("   [{}] {:?}", i, update);
                            }
                        }
                    }
                    println!();
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
