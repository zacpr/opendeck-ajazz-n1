/// Comprehensive dial/encoder debug tool for N1
/// Run with: cargo run --bin debug_dial
/// 
/// This tool will:
/// 1. Find all HID interfaces on the N1
/// 2. Connect to each one and show what inputs come through
/// 3. Help identify which interface carries dial/face button events

use async_hid::{HidBackend, AsyncHidRead, AsyncHidWrite};
use futures_lite::StreamExt;

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[derive(Debug, Clone)]
struct HidInterface {
    name: String,
    usage_page: u16,
    usage: u16,
    id: async_hid::DeviceId,
}

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

/// Parse raw HID report to extract input data
fn parse_report(buf: &[u8], len: usize) -> Option<(u8, u8, String)> {
    if len < 11 {
        return None;
    }
    
    let is_ack = buf[0] == 65 && buf[1] == 67 && buf[2] == 75;
    
    if !is_ack {
        return None;
    }
    
    let input = buf[9];
    let state = buf[10];
    
    let classification = match input {
        0 => "Header/Sync".to_string(),
        1..=15 => format!("Main button {}", input),
        16..=18 => format!("Top LCD {}", input),
        30 => "Left face button".to_string(),
        31 => "Right face button".to_string(),
        50 => "Dial rotate CCW (←)".to_string(),
        51 => "Dial rotate CW (→)".to_string(),
        _ => format!("Unknown input {}", input),
    };
    
    Some((input, state, classification))
}

/// Listen on a specific interface for a short time
async fn test_interface(interface: &HidInterface, backend: &HidBackend, duration_secs: u64) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut events = vec![];
    
    let mut device_iter = backend.query_devices(&interface.id).await?;
    let device = match device_iter.next() {
        Some(d) => d,
        None => return Ok(events),
    };
    
    let (mut reader, mut writer): (async_hid::DeviceReader, async_hid::DeviceWriter) = device.open().await?;
    
    // Try to set mode if this is the main interface (usage_page 65440)
    if interface.usage_page == 65440 {
        let _ = set_software_mode(&mut writer).await;
    }
    
    let mut buf = vec![0u8; 512];
    let start = std::time::Instant::now();
    
    while start.elapsed().as_secs() < duration_secs {
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            reader.read_input_report(&mut buf)
        ).await {
            Ok(Ok(n)) => {
                let n = n as usize;
                if n > 0 {
                    if let Some((input, state, classification)) = parse_report(&buf, n) {
                        events.push(format!(
                            "input={:3} state={} | {}",
                            input, state, classification
                        ));
                    }
                }
            }
            _ => {}
        }
    }
    
    Ok(events)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  N1 Dial & Face Buttons Debug Tool");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let backend = HidBackend::default();
    let devices: Vec<_> = backend.enumerate().await?.collect().await;
    
    // Find all N1 interfaces
    let mut interfaces = vec![];
    for dev in &devices {
        if dev.vendor_id == AJAZZ_VID && dev.product_id == N1_PID {
            interfaces.push(HidInterface {
                name: dev.name.clone(),
                usage_page: dev.usage_page,
                usage: dev.usage_id,
                id: dev.id.clone(),
            });
        }
    }
    
    if interfaces.is_empty() {
        println!("❌ N1 not found! Make sure it's connected.");
        return Ok(());
    }
    
    println!("Found {} HID interface(s) on N1:\n", interfaces.len());
    for (i, iface) in interfaces.iter().enumerate() {
        println!("  [{}] {}", i, iface.name);
        println!("      Usage Page: {}, Usage: {}", iface.usage_page, iface.usage);
    }
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Testing each interface...");
    println!("  Please press the LEFT FACE BUTTON when prompted");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    for (i, iface) in interfaces.iter().enumerate() {
        println!("\n[Interface {}] {}", i, iface.name);
        println!("  Usage Page: {}, Usage: {}", iface.usage_page, iface.usage);
        println!("  Listening for 3 seconds... (press LEFT face button now)");
        
        match test_interface(iface, &backend, 3).await {
            Ok(events) => {
                if events.is_empty() {
                    println!("  No events received.");
                } else {
                    println!("  Events received:");
                    for event in events {
                        println!("    → {}", event);
                    }
                }
            }
            Err(e) => {
                println!("  Error: {}", e);
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("  Continuous monitoring of main interface");
    println!("═══════════════════════════════════════════════════════════════");
    println!("\nFinding main interface (usage_page=65440)...");
    
    // Find the main interface
    let main_iface = interfaces.iter()
        .find(|i| i.usage_page == 65440)
        .cloned()
        .or_else(|| interfaces.first().cloned())
        .expect("No interface found");
    
    println!("Using: {} (usage_page={})", main_iface.name, main_iface.usage_page);
    println!("\nNow continuously monitoring. Try:");
    println!("  • Left face button (input 30)");
    println!("  • Right face button (input 31)");
    println!("  • Dial press (unknown input)");
    println!("  • Dial rotate left/right (inputs 50/51)");
    println!("  • Hold dial + rotate (may be firmware-handled)");
    println!("\nPress Ctrl+C to exit\n");
    
    let mut device_iter = backend.query_devices(&main_iface.id).await?;
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
    println!("✅ Software mode set, listening...\n");
    
    let mut buf = vec![0u8; 512];
    let mut last_input = 0u8;
    let mut last_state = 255u8;  // Use 255 to force first display
    let mut event_count = 0;
    
    loop {
        match reader.read_input_report(&mut buf).await {
            Ok(n) => {
                let n = n as usize;
                if n == 0 {
                    continue;
                }
                
                // Show raw bytes for debugging
                if let Some((input, state, classification)) = parse_report(&buf, n) {
                    // Only show if different from last
                    if input != last_input || state != last_state {
                        event_count += 1;
                        print!("[{:3}] ", event_count);
                        
                        // Show hex bytes
                        for i in 0..n.min(12) {
                            print!("{:02x} ", buf[i]);
                        }
                        
                        let state_str = if state == 1 {
                            "▼ PRESS  "
                        } else if state == 0 && input != 0 {
                            "▲ RELEASE"
                        } else {
                            "  ─ SYNC  "
                        };
                        
                        println!("| {} | input={:3} ({:2}) | {}", 
                            state_str, input, classification, 
                            if input >= 30 && input <= 51 { "★ SPECIAL ★" } else { "" }
                        );
                        
                        last_input = input;
                        last_state = state;
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
