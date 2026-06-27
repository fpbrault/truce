use std::ffi::CString;
use std::os::raw::c_char;

use crate::ffi::Vst3ParamDescriptor;

pub const PROXY_BASE: u32 = 0x6d63_6d00;
const PROXY_CHANNELS: u32 = 16;
const PROXY_CCS: u32 = 128;
const PROXY_COUNT: u32 = PROXY_CHANNELS * PROXY_CCS;

pub const COUNT: u32 = PROXY_COUNT;

fn leak_cstr(s: String) -> *const c_char {
    CString::new(s).unwrap_or_default().into_raw()
}

fn leak_empty_cstr() -> *const c_char {
    CString::new("").unwrap_or_default().into_raw()
}

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

pub fn normalized_to_plain(normalized: f64) -> f64 {
    normalized.clamp(0.0, 1.0) * 127.0
}

pub fn plain_to_normalized(plain: f64) -> f64 {
    (plain / 127.0).clamp(0.0, 1.0)
}

pub fn plain_to_cc(plain: f64) -> u8 {
    plain.clamp(0.0, 127.0).round() as u8
}

pub fn descriptors() -> Vec<Vst3ParamDescriptor> {
    let mut descs = Vec::with_capacity(PROXY_COUNT as usize);

    for channel in 0..PROXY_CHANNELS {
        for cc in 0..PROXY_CCS {
            let channel = channel as u8;
            let cc = cc as u8;
            let id = to_param_id(channel, cc).expect("validated proxy param range");

            descs.push(Vst3ParamDescriptor {
                id,
                name: leak_cstr(format!("MIDI CC Ch {} CC {}", channel + 1, cc)),
                short_name: leak_cstr(format!("CC {}:{}", channel + 1, cc)),
                units: leak_empty_cstr(),
                min: 0.0,
                max: 127.0,
                default_normalized: 0.0,
                step_count: 0,
                flags: 0,
                group: leak_empty_cstr(),
            });
        }
    }

    descs
}
