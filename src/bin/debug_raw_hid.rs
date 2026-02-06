/// Raw HID Debug for N1 - simple version
/// Run with: cargo run --bin debug_raw_hid

use async_hid::{HidBackend, AsyncHidRead};
use futures_lite::StreamExt;

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

fn format_bytes(bytes: &[u8], len: usize) -> String {
    let mut result = String::new();
    for i in 0..len.min(bytes.len()).min(32) {
        result.push_str(&format!("{:02x} ", bytes[i]));
    }
    result
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Raw HID Debug for N1\n");
    
    let backend = HidBackend::default();
    let devices: Vec<_> = backend.enumerate().await?.collect().await;
    
    // Find N1
    let mut n1_info = None;
    for dev in &devices {
        if dev.vendor_id == AJAZZ_VID && dev.product_id == N1_PID {
            println!("Found N1:");
            println!("  Name: {}", dev.name);
            println!("  VID: {:04x}, PID: {:04x}", dev.vendor_id, dev.product_id);
            println!("  Usage Page: {}, Usage: {}", dev.usage_page, dev.usage_id);
            n1_info = Some(dev.clone());
        }
    }
    
    let n1_info = match n1_info {
        Some(i) => i,
        None => {
            println!("❌ N1 not found");
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
    
    let (mut reader, mut _writer): (async_hid::DeviceReader, async_hid::DeviceWriter) = device.open().await?;
    
    println!("✅ Ready! Press buttons and twist encoder...");
    println!("   Looking for reports that contain input data\n");
    
    let mut buf = vec![0u8; 512];
    loop {
        match reader.read_input_report(&mut buf).await {
            Ok(n) => {
                let n = n as usize;
                if n == 0 {
                    continue;
                }
                
                // Filter to show interesting reports only
                let is_ack = n >= 3 && buf[0] == 65 && buf[1] == 67 && buf[2] == 75;
                let has_data = buf.iter().any(|&b| b != 0);
                
                if has_data {
                    print!("[{:<3}] {}", n, format_bytes(&buf, n));
                    
                    if is_ack {
                        print!(" | ACK");
                        if n > 9 {
                            print!(" input={}", buf[9]);
                        }
                        if n > 10 {
                            print!(" state={}", buf[10]);
                        }
                    }
                    
                    // Show bytes 8-12 which often contain input data
                    if n > 8 {
                        print!(" | bytes[8..12]={:02x} {:02x} {:02x} {:02x}",
                            buf[8], buf.get(9).unwrap_or(&0), buf.get(10).unwrap_or(&0), buf.get(11).unwrap_or(&0));
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
