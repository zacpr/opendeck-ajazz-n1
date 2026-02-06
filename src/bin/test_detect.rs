/// Quick test binary to check if the N1 device is detected
/// Run with: cargo run --bin test_detect

use mirajazz::device::{list_devices, DeviceQuery};

// Device IDs
const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

// All supported device queries (same as in mappings.rs)
const SUPPORTED_QUERIES: [(DeviceQuery, &str); 13] = [
    (DeviceQuery::new(65440, 1, 0x5548, 0x6670), "Mirabox HSV293S"),
    (DeviceQuery::new(65440, 1, 0x6603, 0x1014), "Mirabox HSV293SV3"),
    (DeviceQuery::new(65440, 1, 0x6603, 0x1005), "Mirabox HSV293SV3 (1005)"),
    (DeviceQuery::new(65440, 1, 0x5548, 0x6674), "Ajazz AKP153"),
    (DeviceQuery::new(65440, 1, 0x0300, 0x1010), "Ajazz AKP153E"),
    (DeviceQuery::new(65440, 1, 0x0300, 0x3010), "Ajazz AKP153E (rev. 2)"),
    (DeviceQuery::new(65440, 1, 0x0300, 0x1020), "Ajazz AKP153R"),
    (DeviceQuery::new(65440, 1, 0x0300, 0x3007), "Ajazz N1"),
    (DeviceQuery::new(65440, 1, 0x0b00, 0x1000), "Mars Gaming MSD-ONE"),
    (DeviceQuery::new(65440, 1, 0x0c00, 0x1000), "Mad Dog GK150K"),
    (DeviceQuery::new(65440, 1, 0x0a00, 0x1001), "Risemode Vision 01"),
    (DeviceQuery::new(65440, 1, 0x1500, 0x3003), "Soomfon Stream Controller"),
    (DeviceQuery::new(65440, 1, 0x0500, 0x1001), "TMICE Stream Controller"),
];

// N1 query specifically
const N1_QUERY: DeviceQuery = DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Scanning for supported devices...\n");
    
    // First try to find N1 specifically
    println!("=== Checking for Ajazz N1 (VID: {:04x}, PID: {:04x}) ===", AJAZZ_VID, N1_PID);
    
    match list_devices(&[N1_QUERY]).await {
        Ok(devices) => {
            if devices.is_empty() {
                println!("❌ Ajazz N1 not found with usage page 65440.");
                println!("   This could mean:");
                println!("   1. The device is not plugged in");
                println!("   2. The device uses a different HID usage page");
                println!("   3. You need to install udev rules");
            } else {
                println!("✅ Found {} Ajazz N1 device(s)!", devices.len());
                for dev in &devices {
                    println!("   Serial: {:?}", dev.serial_number);
                    println!("   Name: {}", dev.name);
                    println!("   VID: {:04x}, PID: {:04x}", dev.vendor_id, dev.product_id);
                }
                println!("\n🎉 N1 should work with the plugin!");
            }
        }
        Err(e) => {
            println!("❌ Error scanning for N1: {:?}", e);
        }
    }
    
    // Check all other Ajazz devices too (with different possible usage pages)
    println!("\n=== Checking for N1 with different usage pages ===");
    
    // Try some common usage pages
    let usage_pages = [65440u16, 1, 0xFF00, 0];
    let mut found_ajazz = false;
    
    for usage_page in usage_pages {
        let query = DeviceQuery::new(usage_page, 1, AJAZZ_VID, N1_PID);
        if let Ok(devices) = list_devices(&[query]).await {
            for dev in devices {
                found_ajazz = true;
                println!("   Found N1 with usage page {}: {} (serial: {:?})", 
                    usage_page, dev.name, dev.serial_number);
            }
        }
        
        // Also check usage_id 0
        let query = DeviceQuery::new(usage_page, 0, AJAZZ_VID, N1_PID);
        if let Ok(devices) = list_devices(&[query]).await {
            for dev in devices {
                found_ajazz = true;
                println!("   Found N1 with usage page {} / usage id 0: {} (serial: {:?})", 
                    usage_page, dev.name, dev.serial_number);
            }
        }
    }
    
    if !found_ajazz {
        println!("   No Ajazz N1 devices found with any usage page tested.");
    }
    
    // Check all supported devices
    println!("\n=== Checking all supported devices ===");
    let mut found_any = false;
    
    for (query, name) in SUPPORTED_QUERIES.iter() {
        if let Ok(devices) = list_devices(&[query.clone()]).await {
            if !devices.is_empty() {
                found_any = true;
                println!("✅ {} - found {} device(s)", name, devices.len());
            }
        }
    }
    
    if !found_any {
        println!("   (no supported devices found)");
        println!("\n📝 Troubleshooting tips:");
        println!("   1. Make sure your device is plugged in");
        println!("   2. Install udev rules:");
        println!("      sudo cp 40-opendeck-ajazz-n1.rules /etc/udev/rules.d/");
        println!("      sudo udevadm control --reload-rules");
        println!("   3. Unplug and replug the device");
        println!("   4. Check if your user is in the 'plugdev' group:");
        println!("      groups | grep plugdev");
    }
    
    // Also show raw lsusb for verification
    println!("\n=== Running lsusb for verification ===");
    let output = std::process::Command::new("lsusb")
        .output();
    
    if let Ok(output) = output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let relevant_lines: Vec<&str> = stdout.lines()
            .filter(|line| line.to_lowercase().contains("ajazz") || line.contains("0300:"))
            .collect();
        
        if relevant_lines.is_empty() {
            println!("   No Ajazz devices found in lsusb output");
        } else {
            for line in relevant_lines {
                println!("   {}", line);
            }
        }
    }
    
    println!("\nDone!");
    Ok(())
}
