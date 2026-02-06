use mirajazz::{error::MirajazzError, types::DeviceInput};
use std::sync::Mutex;

/// N1 key count (6x3 = 18: 15 buttons + 3 top LCDs)
const N1_KEY_COUNT: usize = 18;

/// N1 encoder/dial input IDs
/// Input 30: Left face button (above the dial)
/// Input 31: Right face button (above the dial)  
/// Input 35: Dial press (push down on the dial)
/// Input 50: Dial rotation counter-clockwise (left)
/// Input 51: Dial rotation clockwise (right)

/// Track current encoder state [dial_pressed]
static DIAL_PRESSED: Mutex<bool> = Mutex::new(false);

/// Process raw input from N1 device (18 keys: 15 buttons + 3 LCDs, plus dial/face buttons)
/// Device inputs 16-18 (top LCDs) map to OpenDeck keys 0-2
/// Device inputs 1-15 (main grid) map to OpenDeck keys 3-17
/// Device inputs 30, 31 (face buttons) are ignored (no display, no action)
/// Device input 35 (dial press) maps to encoder 0
/// Device inputs 50, 51 (dial rotation) map to encoder 0 twist
pub fn process_input_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    log::info!("Processing N1 input: input={}, state={}", input, state);

    // Handle face buttons (inputs 30, 31) - EXPERIMENTAL: currently ignored
    if input == 30 || input == 31 {
        log::info!("N1 face button pressed: input={}, ignoring (experimental)", input);
        return Ok(DeviceInput::ButtonStateChange(vec![false; N1_KEY_COUNT]));
    }

    // Handle dial press (input 35)
    if input == 35 {
        let is_pressed = state != 0;
        log::info!("N1 dial press: is_pressed={}", is_pressed);
        
        let mut dial_state = DIAL_PRESSED.lock().unwrap();
        *dial_state = is_pressed;
        
        log::info!("→ Sending EncoderStateChange([{}])", is_pressed);
        return Ok(DeviceInput::EncoderStateChange(vec![is_pressed]));
    }

    // Handle dial rotation
    if input == 50 {
        log::info!("N1 dial CCW rotation → EncoderTwist([-1])");
        return Ok(DeviceInput::EncoderTwist(vec![-1]));
    }
    if input == 51 {
        log::info!("N1 dial CW rotation → EncoderTwist([1])");
        return Ok(DeviceInput::EncoderTwist(vec![1]));
    }

    // Handle main buttons (inputs 1-18)
    match input {
        1..=18 => read_button_press_n1(input, state),
        _ => {
            log::warn!("Unknown N1 input {}", input);
            Err(MirajazzError::BadData)
        }
    }
}

fn read_button_states(states: &[u8], key_count: usize) -> Vec<bool> {
    let mut bools = vec![];
    for i in 0..key_count {
        bools.push(states.get(i + 1).copied().unwrap_or(0) != 0);
    }
    bools
}

/// Converts opendeck key index to device key index for N1
/// Maps 6×3 grid to device inputs.
/// Both top LCDs (16-18) and main grid (1-15) have a +1 offset.
pub fn opendeck_to_device(key: u8) -> u8 {
    match key {
        // Top row LCDs: send position-1 to compensate for +1 offset
        0 => 15,  // Want 16, send 15 (15+1=16)
        1 => 16,  // Want 17, send 16 (16+1=17)
        2 => 17,  // Want 18, send 17 (17+1=18)
        // Main grid (3-17): send (key-3) to compensate for +1 offset
        3..=17 => key - 3,
        _ => key,
    }
}

/// Converts N1 device key index to opendeck key index
fn device_to_opendeck_n1(key: usize) -> usize {
    match key {
        // Top LCDs: direct mapping
        16 => 0,
        17 => 1,
        18 => 2,
        // Main grid (1-15): direct mapping to OpenDeck 3-17
        1..=15 => key + 2,
        _ => key.saturating_sub(1),
    }
}

fn read_button_press_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; N1_KEY_COUNT + 1]);

    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(
            &button_states,
            N1_KEY_COUNT,
        )));
    }

    let pressed_index: usize = device_to_opendeck_n1(input as usize);

    if pressed_index < N1_KEY_COUNT {
        button_states[pressed_index + 1] = state;
    }

    Ok(DeviceInput::ButtonStateChange(read_button_states(
        &button_states,
        N1_KEY_COUNT,
    )))
}
