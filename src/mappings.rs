use mirajazz::{
    device::DeviceQuery,
    types::{HidDeviceInfo, ImageFormat, ImageMirroring, ImageMode, ImageRotation},
};

// Must be unique between all the plugins, 2 characters long and match `DeviceNamespace` field in `manifest.json`
pub const DEVICE_NAMESPACE: &str = "N1";

pub const AJAZZ_VID: u16 = 0x0300;
pub const N1_PID: u16 = 0x3007;

pub const N1_QUERY: DeviceQuery = DeviceQuery::new(65440, 1, AJAZZ_VID, N1_PID);

pub const QUERIES: [DeviceQuery; 1] = [N1_QUERY];

/// Returns correct image format for device kind and key
pub fn get_image_format_for_key(_kind: &Kind, key: u8) -> ImageFormat {
    // N1 uses different format: no rotation, no mirroring
    // With 6×3 layout:
    // Keys 0, 1, 2 are top LCD screens (64×64)
    // Keys 3-17 are main buttons (96×96)
    let size = if key <= 2 { (64, 64) } else { (96, 96) };
    ImageFormat {
        mode: ImageMode::JPEG,
        size,
        rotation: ImageRotation::Rot0,
        mirror: ImageMirroring::None,
    }
}

#[derive(Debug, Clone)]
pub enum Kind {
    N1,
}

impl Kind {
    /// Matches devices VID+PID pairs to correct kinds
    pub fn from_vid_pid(vid: u16, pid: u16) -> Option<Self> {
        if vid == AJAZZ_VID && pid == N1_PID {
            Some(Kind::N1)
        } else {
            None
        }
    }

    /// Returns protocol version for device
    pub fn protocol_version(&self) -> usize {
        3 // N1 uses protocol v3
    }

    /// Returns (rows, cols) layout for this device type
    pub fn layout(&self) -> (usize, usize) {
        // N1: 6 rows × 3 cols = 18 keys
        // Arranged to match physical layout:
        // Row 0: [LCD_16] [LCD_17] [LCD_18]  <- 3 top LCDs (inputs 16, 17, 18)
        // Row 1: [KEY_1]  [KEY_2]  [KEY_3]   <- Main row 0 (inputs 1, 2, 3)
        // Row 2: [KEY_4]  [KEY_5]  [KEY_6]   <- Main row 1 (inputs 4, 5, 6)
        // Row 3: [KEY_7]  [KEY_8]  [KEY_9]   <- Main row 2 (inputs 7, 8, 9)
        // Row 4: [KEY_10] [KEY_11] [KEY_12]  <- Main row 3 (inputs 10, 11, 12)
        // Row 5: [KEY_13] [KEY_14] [KEY_15]  <- Main row 4 (inputs 13, 14, 15)
        // Note: The 2 top normal buttons (inputs 30, 31) are NOT shown in GUI
        // (They work but have no display, so we hide them to avoid confusion)
        (6, 3)
    }

    /// Returns number of display keys for this device
    pub fn key_count(&self) -> usize {
        // N1 has 18 display keys total (15 main + 3 top LCDs)
        // Note: 2 normal buttons (inputs 30, 31) are NOT counted as they have no display
        18
    }

    /// Returns number of encoders (dials/knobs) for this device
    /// N1 has 3 virtual encoders:
    /// - Encoder 0: Left face button
    /// - Encoder 1: Right face button
    /// - Encoder 2: Dial (press + rotate)
    pub fn encoder_count(&self) -> usize {
        3
    }

    /// Returns human-readable device name
    pub fn human_name(&self) -> String {
        "Ajazz N1".to_string()
    }


}

#[derive(Debug, Clone)]
pub struct CandidateDevice {
    pub id: String,
    pub dev: HidDeviceInfo,
    pub kind: Kind,
}
