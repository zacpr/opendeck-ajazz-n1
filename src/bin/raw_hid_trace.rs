/// Raw HID trace - shows exactly what bytes come from the device
/// Run with: cargo run --bin raw_hid_trace

use async_hid::{HidBackend, AsyncHidRead, AsyncHidWrite};
use futures_lite::StreamExt;

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

/// Send the set_mode command (mode 3 = software mode)
async fn set_software_mode(writer: &mut async_hid::DeviceWriter) -> Result<(), Box<dyn std::error::Error>> {
    let mode_packet = vec![
        0x00, 0x43, 0x52, 0x54, 0x00, 0x00, 
        0x4D, 0x4F, 0x44, 0x00, 0x00, 0x33
    ];
    writer.write_output_report(&mode_packet).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  N1 RAW HID TRACE");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let backend = HidBackend::default();
    let devices: Vec<_> = backend.enumerate().await?.collect().await;
    
    // Find N1 with usage_page 65440 (main interface)
    let mut n1_info = None;
    for dev in &devices {
        if dev.vendor_id == AJAZZ_VID && dev.product_id == N1_PID && dev.usage_page == 65440 {
            println!("Found N1 interface:");
            println!("  Name: {}", dev.name);
            println!("  Usage Page: {}, Usage: {}", dev.usage_page, dev.usage_id);
            n1_info = Some(dev.clone());
            break;
        }
    }
    
    let n1_info = match n1_info {
        Some(i) => i,
        None => {
            println!("❌ N1 not found with usage_page=65440!");
            return Ok(());
        }
    };
    
    println!("\n📡 Opening device...");
    let mut device_iter = backend.query_devices(&n1_info.id).await?;
    let device = match device_iter.next() {
        Some(d) => d,
        None => {
            println!("❌ Device disappeared");
            return Ok(());
        }
    };
    
    let (mut reader, mut writer): (async_hid::DeviceReader, async_hid::DeviceWriter) = device.open().await?;
    
    // Set software mode
    set_software_mode(&mut writer).await?;
    println!("✅ Software mode set\n");
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  RAW HID REPORTS");
    println!("  Looking for: input=byte[9], state=byte[10]");
    println!("  Expected inputs: 30, 31, 35, 50, 51");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    println!("👉 NOW TEST EACH BUTTON:");
    println!("   1. Press LEFT face button (expect input=30)");
    println!("   2. Press RIGHT face button (expect input=31)");
    println!("   3. Press DIAL DOWN (expect input=35)");
    println!("   4. Rotate dial LEFT (expect input=50)");
    println!("   5. Rotate dial RIGHT (expect input=51)");
    println!("\nPress Ctrl+C to exit\n");
    
    let mut buf = vec![0u8; 512];
    let mut seen_inputs = std::collections::HashSet::new();
    
    loop {
        match reader.read_input_report(&mut buf).await {
            Ok(n) => {
                let n = n as usize;
                if n == 0 {
                    continue;
                }
                
                // Check for ACK prefix (65 67 75 = "ACK")
                let is_ack = n >= 3 && buf[0] == 65 && buf[1] == 67 && buf[2] == 75;
                
                if is_ack && n > 10 {
                    let input = buf[9];
                    let state = buf[10];
                    
                    // Only show interesting inputs
                    let is_interesting = input == 30 || input == 31 || input == 35 || input == 50 || input == 51 || input == 0;
                    
                    if is_interesting || !seen_inputs.contains(&input) {
                        seen_inputs.insert(input);
                        
                        let label = match input {
                            0 => "SYNC",
                            30 => "LEFT FACE BTN ★",
                            31 => "RIGHT FACE BTN ★",
                            35 => "DIAL PRESS ★",
                            50 => "DIAL CCW ★",
                            51 => "DIAL CW ★",
                            1..=18 => &format!("Button {}", input),
                            _ => "OTHER",
                        };
                        
                        print!("[{:<3}] ", n);
                        for i in 0..n.min(12) {
                            print!("{:02x} ", buf[i]);
                        }
                        
                        let state_str = if state == 1 { "PRESS" } else if state == 0 && input != 0 { "RELEASE" } else { "SYNC" };
                        println!("| input={:3} state={:3} | {:15} | {}", input, state, state_str, label);
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
