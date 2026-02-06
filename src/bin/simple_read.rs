/// Simple continuous reader - matches mirajazz N1 example
/// Run with: cargo run --bin simple_read

use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    types::DeviceInput,
};
use std::time::Duration;

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;
const N1_QUERY: DeviceQuery = DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID);

fn process_input(input: u8, state: u8) -> Result<DeviceInput, mirajazz::error::MirajazzError> {
    println!("Key {}: {}", input, state);
    Ok(DeviceInput::NoData)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Looking for N1...\n");
    
    let devices = list_devices(&[N1_QUERY]).await?;
    if devices.is_empty() {
        println!("❌ N1 not found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("✅ Found: {} (Serial: {:?})", dev_info.name, dev_info.serial_number);
    
    // IMPORTANT: Close OpenDeck first!
    println!("\n⚠️  Make sure OpenDeck is closed!");
    println!("Press ENTER to continue...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    
    // Connect with key_count=18, encoder_count=0 (matching mirajazz example)
    println!("\n📡 Connecting with key_count=18, encoder_count=0...");
    let device = Device::connect(&dev_info, 3, 18, 0).await?;
    
    println!("⚙️  Setting software mode...");
    device.set_mode(3).await?;
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    device.set_brightness(50).await?;
    device.clear_all_button_images().await?;
    
    println!("✅ Ready! Press any button.");
    println!("(Try the 2 top buttons and main grid buttons)\n");
    
    let reader = device.get_reader(process_input);
    
    loop {
        match reader.read(None).await {
            Ok(_) => {}, // NoData returns empty vec
            Err(e) => {
                println!("⚠️  Error: {:?}", e);
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
