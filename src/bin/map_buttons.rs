/// Simple button mapper - just logs all inputs
/// Run with: cargo run --bin map_buttons
/// Then press buttons and watch the output

use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    types::DeviceInput,
    error::MirajazzError,
};

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;
const N1_QUERY: DeviceQuery = DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID);

fn process_input(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    // Map input to what we think it is
    let name = match input {
        30 => "TOP_BTN_LEFT",
        31 => "TOP_BTN_RIGHT",
        1..=18 => "DISPLAY_BTN",
        50 => "ENCODER_CCW",
        51 => "ENCODER_CW",
        _ => "UNKNOWN",
    };
    
    if state == 1 {
        println!("[PRESS]  input={:<3} → {}", input, name);
    } else {
        println!("[RELEASE] input={:<3} → {}", input, name);
    }
    
    // Return empty
    Ok(DeviceInput::ButtonStateChange(vec![false; 64]))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Connecting to N1...\n");
    
    let devices = list_devices(&[N1_QUERY]).await?;
    if devices.is_empty() {
        println!("❌ N1 not found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("✅ Connected to: {}\n", dev_info.name);
    
    // Use key_count=64 to see all inputs
    let device = Device::connect(&dev_info, 3, 64, 1).await?;
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("🎮 N1 Button Mapper - Press any button\n");
    println!("Expected layout:");
    println!("  [BTN_A] [BTN_B] [ENCODER]");
    println!("  [LCD_0] [LCD_1] [LCD_2]");
    println!("  [Main: 5 rows × 3 cols]");
    println!();
    println!("Press buttons in order from top-left to bottom-right.");
    println!("Press Ctrl+C to exit.\n");
    
    let reader = device.get_reader(process_input);
    
    loop {
        match reader.read(None).await {
            Ok(updates) => {
                for update in updates {
                    match update {
                        DeviceStateUpdate::ButtonDown(key) => {
                            println!("  📥 OpenDeck reports: ButtonDown(key={})", key);
                        }
                        DeviceStateUpdate::ButtonUp(key) => {
                            println!("  📤 OpenDeck reports: ButtonUp(key={})", key);
                        }
                        DeviceStateUpdate::EncoderDown(enc) => {
                            println!("  🔘 OpenDeck reports: EncoderDown(encoder={})", enc);
                        }
                        DeviceStateUpdate::EncoderUp(enc) => {
                            println!("  🔘 OpenDeck reports: EncoderUp(encoder={})", enc);
                        }
                        DeviceStateUpdate::EncoderTwist(enc, val) => {
                            println!("  🔄 OpenDeck reports: EncoderTwist(encoder={} value={})", enc, val);
                        }
                    }
                }
            }
            Err(e) => {
                println!("⚠️  Error: {:?}", e);
            }
        }
    }
}
