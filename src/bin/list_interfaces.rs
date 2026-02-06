/// List all HID interfaces for N1
/// This helps identify if the dial/face buttons are on a different interface

use async_hid::HidBackend;
use futures_lite::StreamExt;

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  N1 HID Interface Lister");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    let backend = HidBackend::default();
    let devices: Vec<_> = backend.enumerate().await?.collect().await;
    
    println!("All HID devices found:\n");
    
    let mut n1_count = 0;
    for dev in &devices {
        let is_n1 = dev.vendor_id == AJAZZ_VID && dev.product_id == N1_PID;
        let prefix = if is_n1 { "🎯 [N1]" } else { "       " };
        
        if is_n1 {
            n1_count += 1;
        }
        
        // Usage page meanings
        let usage_desc = match (dev.usage_page, dev.usage_id) {
            (1, 6) => "Generic Desktop / Keyboard".to_string(),
            (1, 2) => "Generic Desktop / Mouse".to_string(),
            (1, 4) => "Generic Desktop / Joystick".to_string(),
            (1, 5) => "Generic Desktop / Gamepad".to_string(),
            (12, 1) => "Consumer / Consumer Control (media keys)".to_string(),
            (65440, _) => format!("Vendor Specific (0x{:04x})", dev.usage_id),
            (up, uid) => format!("Other (page={}, id={})", up, uid),
        };
        
        println!("{} {}", prefix, dev.name);
        println!("       VID: {:04x}, PID: {:04x}", dev.vendor_id, dev.product_id);
        println!("       Usage Page: {:5}, Usage: {:3} | {}", 
            dev.usage_page, dev.usage_id, usage_desc);
        println!();
    }
    
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Summary: Found {} N1 interface(s)", n1_count);
    println!("═══════════════════════════════════════════════════════════════\n");
    
    if n1_count == 0 {
        println!("❌ No N1 interfaces found!");
        println!("   Make sure your N1 is connected.");
    } else {
        println!("Notes:");
        println!("  • Consumer Control (Usage Page 12) often handles media keys, dials");
        println!("  • Vendor Specific (Usage Page 65440) is the main control interface");
        println!("  • Generic Desktop (Usage Page 1) might handle standard HID inputs");
        println!();
        println!("If the dial/face buttons don't work:");
        println!("  1. Check if they're on Consumer Control interface");
        println!("  2. The device firmware may handle them internally");
        println!("  3. A different initialization command may be needed");
    }
    
    Ok(())
}
