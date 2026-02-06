/// Raw HID debug tool - reads directly from USB HID
/// Run with: cargo run --bin debug_raw

use async_hid::{HidBackend, AsyncHidRead};
use futures_lite::StreamExt;

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Scanning for N1 via raw HID...\n");
    
    let backend = HidBackend::default();
    let devices: Vec<_> = backend.enumerate().await?.collect().await;
    
    println!("Found {} HID devices:", devices.len());
    let mut n1_device = None;
    
    for dev in &devices {
        let is_ajazz = dev.vendor_id == AJAZZ_VID;
        let is_n1 = dev.product_id == N1_PID;
        
        if is_ajazz {
            println!("  🎯 Ajazz device:");
            println!("     Name: {}", dev.name);
            println!("     VID: {:04x}, PID: {:04x}", dev.vendor_id, dev.product_id);
            println!("     Serial: {:?}", dev.serial_number);
            println!("     Usage Page: {}, Usage: {}", dev.usage_page, dev.usage_id);
            
            if is_n1 {
                n1_device = Some(dev.clone());
            }
        }
    }
    
    let n1_info = match n1_device {
        Some(d) => d,
        None => {
            println!("\n❌ N1 not found!");
            return Ok(());
        }
    };
    
    println!("\n📡 Opening N1 device...");
    let mut devices = backend.query_devices(&n1_info.id).await?;
    
    let device = match devices.next() {
        Some(d) => d,
        None => {
            println!("❌ Device disappeared!");
            return Ok(());
        }
    };
    
    let (mut reader, _writer): (async_hid::DeviceReader, _) = device.open().await?;
    
    println!("✅ Connected! Reading raw HID reports...");
    println!("   Press buttons on N1 to see raw bytes\n");
    
    // Read raw reports
    let mut buf = vec![0u8; 64];
    loop {
        match reader.read_input_report(&mut buf).await {
            Ok(n) => {
                let n_usize: usize = n as usize;
                if n_usize > 0 {
                    print!("[{} bytes] ", n_usize);
                    for i in 0..n_usize.min(16) {
                        print!("{:02x} ", buf[i]);
                    }
                    if n_usize > 16 {
                        print!("...");
                    }
                    println!();
                    
                    // Try to interpret as button data
                    if n_usize >= 3 && buf[0] == 0x01 {
                        println!("  → Possible button report: input={} state={}", buf[1], buf[2]);
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
