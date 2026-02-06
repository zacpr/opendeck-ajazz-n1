use mirajazz::{error::MirajazzError, types::DeviceInput};

/// AKP153 key count (3x6 = 18)
const AKP153_KEY_COUNT: usize = 18;
/// N1 key count (6x3 = 18: 15 buttons + 3 top LCDs)
const N1_KEY_COUNT: usize = 18;
/// Maximum key count we support (for bounds checking)
const MAX_KEY_COUNT: usize = 18;

/// Process raw input from N1 device (18 keys: 15 buttons + 3 LCDs)
/// Device inputs 16-18 (top LCDs) map to OpenDeck keys 0-2
/// Device inputs 1-15 (main grid) map to OpenDeck keys 3-17
/// Device inputs 30, 31 (normal buttons) are ignored for now
pub fn process_input_n1(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    log::info!("Processing N1 input: {}, {}", input, state);

    // Handle normal buttons (inputs 30, 31) - they work but have no display
    // For now, we silently ignore them. TODO: Map to touchpoints or actions
    if input == 30 || input == 31 {
        log::debug!("N1 normal button pressed: input={}, ignoring", input);
        return Ok(DeviceInput::ButtonStateChange(vec![false; N1_KEY_COUNT]));
    }

    match input as usize {
        1..=18 => read_button_press_n1(input, state),
        _ => {
            log::warn!("Unknown N1 input {}", input);
            Err(MirajazzError::BadData)
        }
    }
}

/// Process raw input from AKP153 device (18 buttons, remapped)
pub fn process_input_akp153(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    log::info!("Processing AKP153 input: {}, {}", input, state);

    match input as usize {
        0..=MAX_KEY_COUNT => read_button_press_akp153(input, state),
        _ => {
            log::warn!("Unknown AKP153 input {}", input);
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

/// Converts opendeck key index to device key index
/// For AKP153: Uses specific mapping for 3x6 layout
/// For N1: Maps 6×3 grid to device inputs.
/// Both top LCDs (16-18) and main grid (1-15) have a +1 offset.
///
///   OpenDeck grid:     Intended device:    We send:
///   [0]  [1]  [2]      [16] [17] [18]      [15] [16] [17]  (offset)
///   [3]  [4]  [5]      [1]  [2]  [3]       [0]  [1]  [2]   (offset)
///   [6]  [7]  [8]      [4]  [5]  [6]       [3]  [4]  [5]
///   [9]  [10] [11]     [7]  [8]  [9]       [6]  [7]  [8]
///   [12] [13] [14]     [10] [11] [12]      [9]  [10] [11]
///   [15] [16] [17]     [13] [14] [15]      [12] [13] [14]
pub fn opendeck_to_device(key: u8, is_n1: bool) -> u8 {
    if is_n1 {
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
    } else {
        // AKP153: specific key mapping
        if (key as usize) < AKP153_KEY_COUNT {
            [12, 9, 6, 3, 0, 15, 13, 10, 7, 4, 1, 16, 14, 11, 8, 5, 2, 17][key as usize]
        } else {
            key
        }
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

/// Converts AKP153 device key index to opendeck key index (specific mapping)
fn device_to_opendeck_akp153(key: usize) -> usize {
    let key = key.saturating_sub(1);

    if key < AKP153_KEY_COUNT {
        [4, 10, 16, 3, 9, 15, 2, 8, 14, 1, 7, 13, 0, 6, 12, 5, 11, 17][key]
    } else {
        key
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

fn read_button_press_akp153(input: u8, state: u8) -> Result<DeviceInput, MirajazzError> {
    let mut button_states = vec![0x01];
    button_states.extend(vec![0u8; MAX_KEY_COUNT + 1]);

    if input == 0 {
        return Ok(DeviceInput::ButtonStateChange(read_button_states(
            &button_states,
            MAX_KEY_COUNT,
        )));
    }

    let pressed_index: usize = device_to_opendeck_akp153(input as usize);

    if pressed_index < MAX_KEY_COUNT {
        button_states[pressed_index + 1] = state;
    }

    Ok(DeviceInput::ButtonStateChange(read_button_states(
        &button_states,
        MAX_KEY_COUNT,
    )))
}

