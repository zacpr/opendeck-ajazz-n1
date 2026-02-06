/// X11 Input Injection Test for N1
/// Injects fake keypresses/mouse events when you use the N1 dial/buttons
/// This lets you test with xev, xinput, or any X11 tool
/// Run with: cargo run --bin x11_inject
///
/// THEN in another terminal, run: xev
/// and watch the events appear when you press N1 buttons!

use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    error::MirajazzError,
};
use std::process::Command;
use mirajazz::types::DeviceInput;

const N1_KEY_COUNT: usize = 18;
const N1_FACE_BUTTON_LEFT: u8 = 30;
const N1_FACE_BUTTON_RIGHT: u8 = 31;
const N1_DIAL_PRESS: u8 = 35;
const N1_DIAL_ROTATE_CCW: u8 = 50;
const N1_DIAL_ROTATE_CW: u8 = 51;

fn device_to_opendeck_n1(key: usize) -> usize {
    match key {
        16 => 0, 17 => 1, 18 => 2,
        1..=15 => key + 2,
        _ => key.saturating_sub(1),
    }
}

fn read_button_states(states: &[u8], key_count: usize) -> Vec<bool> {
    let mut bools = vec![];
    for i in 0..key_count {
        bools.push(states.get(i + 1).copied().unwrap_or(0) != 0);
    }
    bools
}

fn read_button_press_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; N1_KEY_COUNT + 1]);
    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(&button_states, N1_KEY_COUNT)));
    }
    let pressed_index: usize = device_to_opendeck_n1(input as usize);
    if pressed_index < N1_KEY_COUNT {
        button_states[pressed_index + 1] = state;
    }
    Ok(DeviceInput::ButtonStateChange(read_button_states(&button_states, N1_KEY_COUNT)))
}

pub fn process_input_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    match input {
        1..=18 => read_button_press_n1(input, state),
        N1_FACE_BUTTON_LEFT | N1_FACE_BUTTON_RIGHT => {
            Ok(DeviceInput::EncoderStateChange(vec![state != 0]))
        }
        N1_DIAL_PRESS => {
            Ok(DeviceInput::EncoderStateChange(vec![state != 0]))
        }
        N1_DIAL_ROTATE_CCW => Ok(DeviceInput::EncoderTwist(vec![-1])),
        N1_DIAL_ROTATE_CW => Ok(DeviceInput::EncoderTwist(vec![1])),
        _ => Err(MirajazzError::BadData),
    }
}

const AJAZZ_VID: u16 = 0x0300;
const N1_PID: u16 = 0x3007;

/// Send a key press using xdotool
fn send_key(key: &str) {
    let _ = Command::new("xdotool")
        .args(["key", key])
        .spawn();
}

/// Send volume up/down
fn send_volume(direction: &str) {
    let key = match direction {
        "up" => "XF86AudioRaiseVolume",
        "down" => "XF86AudioLowerVolume",
        _ => return,
    };
    let _ = Command::new("xdotool")
        .args(["key", key])
        .spawn();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════════════");
    println!("  N1 X11 Input Injection Test");
    println!("═══════════════════════════════════════════════════════════════\n");
    
    // Check for xdotool
    if Command::new("which").arg("xdotool").output()?.status.success() == false {
        println!("❌ xdotool not found! Install it with:");
        println!("   sudo apt install xdotool   # Debian/Ubuntu");
        println!("   sudo dnf install xdotool   # Fedora");
        println!("   sudo pacman -S xdotool     # Arch");
        return Ok(());
    }
    
    println!("✅ xdotool found!");
    println!();
    println!("This tool will inject X11 input events when you use the N1.");
    println!();
    println!("SETUP:");
    println!("  1. Open another terminal");
    println!("  2. Run: xev");
    println!("  3. Make sure the xev window has focus");
    println!("  4. Come back here and connect the N1");
    println!();
    println!("Then try:");
    println!("  • Face buttons → inject 'a' and 'b' keys");
    println!("  • Dial rotate → inject volume up/down keys");
    println!();
    
    let devices = list_devices(&[DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID)]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    println!("Found: {}", dev_info.name);
    
    let device = Device::connect(&dev_info, 3, 18, 1).await?;
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("✅ Connected!");
    println!("\n🎯 Now press buttons on your N1 and watch xev!\n");
    
    let reader = device.get_reader(process_input_n1);
    
    loop {
        match reader.read(None).await {
            Ok(updates) => {
                for update in updates {
                    match update {
                        DeviceStateUpdate::EncoderDown(enc) => {
                            println!("🔘 Encoder {} pressed → sending key 'a'", enc);
                            send_key("a");
                        }
                        DeviceStateUpdate::EncoderUp(_) => {}
                        DeviceStateUpdate::EncoderTwist(enc, val) => {
                            let dir = if val > 0 { "up" } else { "down" };
                            let arrow = if val > 0 { "→" } else { "←" };
                            println!("🔄 Encoder {} twist {} → volume {}", enc, arrow, dir);
                            send_volume(dir);
                        }
                        _ => {}
                    }
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
