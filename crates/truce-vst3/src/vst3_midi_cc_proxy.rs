const PROXY_BASE: u32 = 0x6d63_6d00;
const PROXY_CHANNELS: u32 = 16;
const PROXY_CCS: u32 = 128;
const PROXY_COUNT: u32 = PROXY_CHANNELS * PROXY_CCS;

pub fn is_enabled(info: &truce_core::info::PluginInfo) -> bool {
    info.accepts_midi_in
}

pub fn is_proxy_id(id: u32) -> bool {
    id >= PROXY_BASE && id < PROXY_BASE + PROXY_COUNT
}

pub fn to_param_id(channel: u8, cc: u8) -> Option<u32> {
    if channel < 16 && cc < 128 {
        Some(PROXY_BASE + u32::from(channel) * PROXY_CCS + u32::from(cc))
    } else {
        None
    }
}

pub fn from_param_id(id: u32) -> Option<(u8, u8)> {
    let offset = id.checked_sub(PROXY_BASE)?;
    if offset >= PROXY_COUNT {
        return None;
    }
    Some(((offset / PROXY_CCS) as u8, (offset % PROXY_CCS) as u8))
}

pub fn normalized_to_cc(normalized: f64) -> u8 {
    (normalized.clamp(0.0, 1.0) * 127.0).round() as u8
}
