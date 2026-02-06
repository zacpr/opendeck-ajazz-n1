/// Test different device modes to see which one enables dial/face button events
/// Run with: cargo run --bin debug_modes

use async_hid::{HidBackend, AsyncHidRead, AsyncHidWrite};
use futures_lite::StreamExt;

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

async fn set_mode(writer: &mut async_hid::DeviceWriter, mode: u8) -> Result<(), Box<dyn std::error::Error>> {
    let mode_packet = vec![
        0x00, 0x43, 0x52, 0x54, 0x00, 0x00, 
        0x4D, 0x4F, 0x44, 0x00, 0x00, 
        0x30 + mode  // '0', '1', '2', '3', etc.
    ];
    writer.write_output_report(&mode_packet).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    Ok(())
}

async fn test_mode(backend: &HidBackend, dev_info: &async_hid::DeviceInfo, mode: u8, duration_secs: u64) -> Vec<String> {
    let mut events = vec![];
    
    let Ok(mut device_iter) = backend.query_devices(&dev_info.id).await else {
        return events;
    };
    
    let Some(device) = device_iter.next() else {
        return events;
    };
    
    let Ok((mut reader, mut writer)) = device.open().await else {
        return events;
    };
    
    // Set the mode
    let _ = set_mode(&mut writer, mode).await;
    
    let mut buf = vec![0u8; 512];
    let start = std::time::Instant::now();
    
    while start.elapsed().as_secs() < duration_secs {
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(50),
            reader.read_input_report(&mut buf)
        ).await {
            Ok(Ok(n)) => {
                let n = n as usize;
                if n >= 11 && buf[0] == 65 && buf[1] == 67 && buf[2] == 75 {
                    let input = buf[9];
                    let state = buf[10];
                    
                    let desc = match input {
                        0 => "SYNC".to_string(),
                        1..=18 => format!("Button {}", input),
                        30 => "FaceBtnL".to_string(),
                        31 => "FaceBtnR".to_string(),
                        35 => "DialPress".to_string(),
                        50 => "DialCCW".to_string(),
                        51 => "DialCW".to_string(),
                        _ => format!("Input{}", input),
                    };
                    
                    events.push(format!("{} state={}", desc, state));
                }
            }
            _ => {}
        }
    }
    
    events
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  N1 Mode Testing Tool");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let backend = HidBackend::default();
    let devices: Vec<_> = backend.enumerate().await?.collect().await;
    
    // Find N1 main interface (usage_page 65440)
    let mut n1_info = None;
    for dev in &devices {
        if dev.vendor_id == AJAZZ_VID && dev.product_id == N1_PID && dev.usage_page == 65440 {
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
    
    println!("Found N1: {}", n1_info.name);
    println!("Usage Page: {}, Usage: {}\n", n1_info.usage_page, n1_info.usage_id);
    
    // Test different modes
    for mode in [0u8, 1, 2, 3, 4, 5] {
        println!("═══════════════════════════════════════════════════════════════");
        println!("  Testing Mode {}", mode);
        println!("═══════════════════════════════════════════════════════════════");
        println!("\n👉 Please do these actions NOW:");
        println!("   1. Press LEFT face button");
        println!("   2. Press RIGHT face button");
        println!("   3. Rotate dial left");
        println!("   4. Rotate dial right");
        println!("   5. Press dial down");
        println!();
        
        let events = test_mode(&backend, &n1_info, mode, 5).await;
        
        if events.is_empty() {
            println!("❌ No events received in mode {}\n", mode);
        } else {
            println!("✅ Events received:");
            for event in events {
                println!("   • {}", event);
            }
            println!();
        }
        
        // Small delay between modes
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Testing Complete");
    println!("═══════════════════════════════════════════════════════════════");
    println!();
    println!("If you didn't see 'FaceBtnL', 'FaceBtnR', 'DialCCW', or 'DialCW'");
    println!("events in any mode, the device firmware may be handling these");
    println!("inputs internally and not sending them over HID.");
    println!();
    println!("The hold+rotate gesture is likely handled entirely by firmware.");
    
    Ok(())
}
