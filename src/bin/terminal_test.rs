/// Terminal-based visual test for N1
/// Shows a live updating display in the terminal
/// Run with: cargo run --bin terminal_test

use mirajazz::{
    device::{list_devices, Device, DeviceQuery},
    state::DeviceStateUpdate,
    error::MirajazzError,
};
use std::io::{self, Write};
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

fn clear_screen() {
    print!("\x1B[2J\x1B[H");
    io::stdout().flush().unwrap();
}

fn draw_ui(face_left: bool, face_right: bool, dial_pressed: bool, rotation: i32) {
    clear_screen();
    
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                 N1 Terminal Visual Test                      ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║                                                              ║");
    
    // Face buttons
    let left_status = if face_left { "● PRESSED " } else { "○ released" };
    let right_status = if face_right { "● PRESSED " } else { "○ released" };
    
    println!("║   ┌─────────────┐                                            ║");
    println!("║   │ Face Buttons│                                            ║");
    println!("║   ├─────────────┤                                            ║");
    println!("║   │ Left:  {} │ Right: {}  ║", left_status, right_status);
    println!("║   └─────────────┘                                            ║");
    println!("║                                                              ║");
    
    // Dial
    let dial_status = if dial_pressed { "● PRESSED " } else { "○ released" };
    let rotation_bar = match rotation {
        r if r > 0 => format!("►►► {} ►►►", r),
        r if r < 0 => format!("◄◄◄ {} ◄◄◄", r.abs()),
        _ => "─────────".to_string(),
    };
    
    println!("║   ┌───────────────────────────┐                              ║");
    println!("║   │           DIAL            │                              ║");
    println!("║   ├───────────────────────────┤                              ║");
    println!("║   │  Press: {}              ║", dial_status);
    println!("║   │  Rotation: {:^20}  ║", rotation_bar);
    println!("║   └───────────────────────────┘                              ║");
    println!("║                                                              ║");
    
    // Instructions
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Press face buttons or rotate the dial to see changes       ║");
    println!("║  Press Ctrl+C to exit                                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    clear_screen();
    
    let devices = list_devices(&[DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID)]).await?;
    
    if devices.is_empty() {
        println!("❌ No N1 device found!");
        return Ok(());
    }
    
    let dev_info = devices.into_iter().next().unwrap();
    
    let device = Device::connect(&dev_info, 3, 18, 1).await?;
    device.set_mode(3).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    let reader = device.get_reader(process_input_n1);
    
    let mut face_left = false;
    let mut face_right = false;
    let mut dial_pressed = false;
    let mut rotation = 0i32;
    let mut last_activity = std::time::Instant::now();
    
    draw_ui(face_left, face_right, dial_pressed, rotation);
    
    loop {
        match tokio::time::timeout(
            tokio::time::Duration::from_millis(100),
            reader.read(None)
        ).await {
            Ok(Ok(updates)) => {
                let mut changed = false;
                
                for update in updates {
                    match update {
                        DeviceStateUpdate::EncoderDown(enc) => {
                            println!("\n🔘 Encoder {} pressed", enc);
                            // We can't distinguish which button from the event alone
                            // But we know one of them was pressed
                            face_left = true;
                            dial_pressed = true;
                            changed = true;
                        }
                        DeviceStateUpdate::EncoderUp(enc) => {
                            println!("\n🔘 Encoder {} released", enc);
                            face_left = false;
                            face_right = false;
                            dial_pressed = false;
                            changed = true;
                        }
                        DeviceStateUpdate::EncoderTwist(enc, val) => {
                            println!("\n🔄 Encoder {} twist: {}", enc, val);
                            rotation += val as i32;
                            last_activity = std::time::Instant::now();
                            changed = true;
                        }
                        _ => {}
                    }
                }
                
                if changed {
                    draw_ui(face_left, face_right, dial_pressed, rotation);
                }
            }
            _ => {}
        }
        
        // Reset rotation display after inactivity
        if rotation != 0 && last_activity.elapsed().as_millis() > 500 {
            rotation = 0;
            draw_ui(face_left, face_right, dial_pressed, rotation);
        }
    }
}
