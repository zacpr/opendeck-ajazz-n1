/// Debug tool to read raw inputs from N1 device
/// Run with: cargo run --bin debug_inputs

use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    types::DeviceInput,
    error::MirajazzError,
};

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

/// Process raw input - capture ALL inputs
fn process_input_debug(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    println!("[RAW_INPUT] input={} state={} (0x{:02x}/0x{:02x})", input, state, input, state);
    
    // Classify based on what we've learned
    let classification = match input {
        0 => "Header/Sync",
        1..=18 => "Button/LCD (expected range)",
        30 => "Encoder press?",
        31 => "Other encoder?",
        50 => "Encoder twist LEFT/CCW",
        51 => "Encoder twist RIGHT/CW",
        _ => "Unknown",
    };
    println!("  → Classification: {}", classification);
    
    // Return empty state
    let button_states = vec![false; 18];
    Ok(DeviceInput::ButtonStateChange(button_states))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Looking for Ajazz N1 device...\n");
    
    // Try different usage pages - buttons might be on a different interface
    let queries = [
        DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID),
        DeviceQuery::new(65440, 0, AJAZZ_VID, N1_PID),
        DeviceQuery::new(1, 1, AJAZZ_VID, N1_PID),
        DeviceQuery::new(0, 0, AJAZZ_VID, N1_PID),
    ];
    
    for (i, query) in queries.iter().enumerate() {
        let devices = list_devices(&[query.clone()]).await?;
        if !devices.is_empty() {
            println!("Query {}: Found {} device(s)", i, devices.len());
            for dev in &devices {
                println!("  - {} (usage_page={}, usage_id={})", 
                    dev.name, dev.usage_page, dev.usage_id);
            }
        }
    }
    
    // Use the standard query
    let devices = list_devices(&[queries[0].clone()]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("\n✅ Using: {} (Serial: {:?})", dev_info.name, dev_info.serial_number);
    
    // Try connecting with different key counts to see if buttons appear
    for key_count in [18usize, 21, 32, 64] {
        println!("\n📡 Testing with key_count={}...", key_count);
        
        match Device::connect(&dev_info, 3, key_count, 1).await {
            Ok(device) => {
                println!("   Connected!");
                
                // Set software mode
                if let Err(e) = device.set_mode(3).await {
                    println!("   Failed to set mode: {:?}", e);
                    continue;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                
                // Clear button states
                device.clear_all_button_images().await.ok();
                
                println!("   Please press any button NOW (5 second window)...");
                
                let reader = device.get_reader(process_input_debug);
                
                // Read for 5 seconds
                let timeout = tokio::time::Duration::from_secs(5);
                let start = std::time::Instant::now();
                
                while start.elapsed() < timeout {
                    match tokio::time::timeout(
                        tokio::time::Duration::from_millis(100),
                        reader.read(None)
                    ).await {
                        Ok(Ok(updates)) => {
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
                        Ok(Err(_)) => {}
                        Err(_) => {} // Timeout
                    }
                }
                
                println!("   Window closed.\n");
                device.shutdown().await.ok();
                
                // Ask if we should continue
                println!("Continue testing? (y/n): ");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                if input.trim().to_lowercase() != "y" {
                    break;
                }
            }
            Err(e) => {
                println!("   Failed to connect: {:?}", e);
            }
        }
    }
    
    println!("\nDone!");
    Ok(())
}
