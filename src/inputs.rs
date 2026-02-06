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
const N1_FACE_BUTTON_LEFT: u8 = 30;
const N1_FACE_BUTTON_RIGHT: u8 = 31;
const N1_DIAL_PRESS: u8 = 35;
const N1_DIAL_ROTATE_CCW: u8 = 50;
const N1_DIAL_ROTATE_CW: u8 = 51;

/// Track current encoder state
/// [encoder0_pressed] - the dial
static ENCODER_STATES: Mutex<[bool; 1]> = Mutex::new([false]);

/// Process raw input from N1 device (18 keys: 15 buttons + 3 LCDs, plus dial/face buttons)
/// Device inputs 16-18 (top LCDs) map to OpenDeck keys 0-2
/// Device inputs 1-15 (main grid) map to OpenDeck keys 3-17
/// Device inputs 30, 31 (face buttons) are ignored for now (no display)
/// Device input 35 (dial press) maps to encoder 0 press
/// Device inputs 50, 51 (dial rotation) map to encoder 0 twist
pub fn process_input_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    log::info!("Processing N1 input: input={}, state={}", input, state);

    let result = match input {
        // Main grid and top LCDs (inputs 1-18)
        1..=18 => read_button_press_n1(input, state),
        
        // Left face button - treat as regular button (not encoder)
        N1_FACE_BUTTON_LEFT => {
            log::debug!("Left face button (input 30) pressed, state={}", state);
            Ok(DeviceInput::ButtonStateChange(vec![false; N1_KEY_COUNT]))
        }
        
        // Right face button - treat as regular button (not encoder)
        N1_FACE_BUTTON_RIGHT => {
            log::debug!("Right face button (input 31) pressed, state={}", state);
            Ok(DeviceInput::ButtonStateChange(vec![false; N1_KEY_COUNT]))
        }
        
        // Dial press - mapped to encoder 0
        N1_DIAL_PRESS => {
            let mut states = ENCODER_STATES.lock().unwrap();
            states[0] = state != 0;
            Ok(DeviceInput::EncoderStateChange(vec![
                states[0],  // Encoder 0: dial press
            ]))
        }
        
        // Dial rotation - mapped to encoder 0
        N1_DIAL_ROTATE_CCW => {
            // Counter-clockwise rotation on encoder 0
            Ok(DeviceInput::EncoderTwist(vec![-1]))
        }
        N1_DIAL_ROTATE_CW => {
            // Clockwise rotation on encoder 0
            Ok(DeviceInput::EncoderTwist(vec![1]))
        }
        
        _ => {
            log::warn!("Unknown N1 input {}", input);
            Err(MirajazzError::BadData)
        }
    };
    
    if let Ok(ref device_input) = result {
        match device_input {
            DeviceInput::EncoderStateChange(states) => {
                log::info!("→ Generated EncoderStateChange: {:?}", states);
            }
            DeviceInput::EncoderTwist(twists) => {
                log::info!("→ Generated EncoderTwist: {:?}", twists);
            }
            _ => {}
        }
    }
    
    result
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
///
///   OpenDeck grid:     Intended device:    We send:
///   [0]  [1]  [2]      [16] [17] [18]      [15] [16] [17]  (offset)
///   [3]  [4]  [5]      [1]  [2]  [3]       [0]  [1]  [2]   (offset)
///   [6]  [7]  [8]      [4]  [5]  [6]       [3]  [4]  [5]
///   [9]  [10] [11]     [7]  [8]  [9]       [6]  [7]  [8]
///   [12] [13] [14]     [10] [11] [12]      [9]  [10] [11]
///   [15] [16] [17]     [13] [14] [15]      [12] [13] [14]
pub fn opendeck_to_device(key: u8) -> u8 {
    match key {
        // Top row LCDs: send position-1 to compensate for +1 offset
        0 => 15,  // Want 16, send 15 (15+1=16)
        1 => 16,  // Want 17, send 16 (16+1=17)
        2 => 17,  // Want 18, send 17 (17+1=18)
        // Main grid (3-17): send (key-3) to compensate for +1 offset
        // key 3 → send 0 → lands at 1
        // key 4 → send 1 → lands at 2
        // ...
        // key 17 → send 14 → lands at 15
        3..=17 => key - 3,
        _ => key,  // Fallback
    }
}

/// Converts N1 device key index to opendeck key index
/// Maps device inputs to 6×3 grid layout (direct mapping):
/// Physical layout (device inputs):    OpenDeck grid:
///   [16] [17] [18]  (top LCDs)          [0]  [1]  [2]   row 0
///   [1]  [2]  [3]   (row 1)             [3]  [4]  [5]   row 1
///   [4]  [5]  [6]   (row 2)             [6]  [7]  [8]   row 2
///   [7]  [8]  [9]   (row 3)             [9]  [10] [11]  row 3
///   [10] [11] [12]  (row 4)             [12] [13] [14]  row 4
///   [13] [14] [15]  (row 5)             [15] [16] [17]  row 5
fn device_to_opendeck_n1(key: usize) -> usize {
    match key {
        // Top LCDs: direct mapping
        16 => 0,  // Top LCD left → OpenDeck left (col 0)
        17 => 1,  // Top LCD middle → OpenDeck middle (col 1)
        18 => 2,  // Top LCD right → OpenDeck right (col 2)
        // Main grid (1-15): direct mapping to OpenDeck 3-17
        1..=15 => key + 2,  // Device 1→OD 3, Device 2→OD 4, ..., Device 15→OD 17
        _ => key.saturating_sub(1),  // Fallback
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
