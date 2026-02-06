/// Trace encoder state changes step by step
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
    let result = match input {
        1..=18 => read_button_press_n1(input, state),
        30 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            let old = states[0];
            states[0] = state != 0;
            println!("  [INPUT 30] Encoder 0: {} -> {}, All states: {:?}", old, states[0], *states);
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        31 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            let old = states[1];
            states[1] = state != 0;
            println!("  [INPUT 31] Encoder 1: {} -> {}, All states: {:?}", old, states[1], *states);
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        35 => {
            let mut states = ENCODER_STATES.lock().unwrap();
            let old = states[2];
            states[2] = state != 0;
            println!("  [INPUT 35] Encoder 2: {} -> {}, All states: {:?}", old, states[2], *states);
            Ok(DeviceInput::EncoderStateChange(vec![states[0], states[1], states[2]]))
        }
        50 => {
            println!("  [INPUT 50] Dial CCW");
            Ok(DeviceInput::EncoderTwist(vec![0, 0, -1]))
        }
        51 => {
            println!("  [INPUT 51] Dial CW");
            Ok(DeviceInput::EncoderTwist(vec![0, 0, 1]))
        }
        _ => Err(MirajazzError::BadData),
    };
    result
}

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Encoder State Tracing Test");
    println!("═══════════════════════════════════════════════════════════════\n");
    println!("This test shows the encoder state transitions.");
    println!("The mirajazz state machine compares new state with old state.\n");
    
    let devices = list_devices(&[DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID)]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("Found: {}\n", dev_info.name);
    
    let device = Device::connect(&dev_info, 3, 18, 3).await?;
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!("✅ Connected!\n");
    
    let reader = device.get_reader(process_input_n1);
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  DO THIS EXACT SEQUENCE:");
    println!("═══════════════════════════════════════════════════════════════");
    println!("1. Press LEFT face button (encoder 0)");
    println!("2. Press RIGHT face button (encoder 1) - DON'T release left");
    println!("3. Release LEFT face button");
    println!("4. Release RIGHT face button\n");
    
    loop {
        match reader.read(None).await {
            Ok(updates) => {
                if !updates.is_empty() {
                    println!("\n📤 Generated events:");
                    for update in &updates {
                        match update {
                            DeviceStateUpdate::EncoderDown(enc) => println!("   ✅ EncoderDown({})", enc),
                            DeviceStateUpdate::EncoderUp(enc) => println!("   ✅ EncoderUp({})", enc),
                            _ => {}
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
