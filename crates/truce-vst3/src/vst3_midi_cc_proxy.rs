use std::ffi::CString;
use std::os::raw::c_char;

use crate::ffi::Vst3ParamDescriptor;

const PROXY_BASE: u32 = 0x6d63_6d00;
const PROXY_CHANNELS: u32 = 16;
const PROXY_CCS: u32 = 128;
const PROXY_COUNT: u32 = PROXY_CHANNELS * PROXY_CCS;

fn leak_cstr(s: String) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

fn leak_static(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
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

pub fn normalized_to_cc(normalized: f64) -> u8 {
    (normalized.clamp(0.0, 1.0) * 127.0).round() as u8
}

pub fn iter_param_ids() -> impl Iterator<Item = u32> {
    (0..PROXY_COUNT).map(|offset| PROXY_BASE + offset)
}

pub fn descriptors() -> Vec<Vst3ParamDescriptor> {
    let mut descs = Vec::with_capacity(PROXY_COUNT as usize);
    for id in iter_param_ids() {
        let (ch, cc) = from_param_id(id).unwrap();
        // Display channel 1-based: hosts and users expect "Ch 1" not "Ch 0".
        let display_ch = ch + 1;
        descs.push(Vst3ParamDescriptor {
            id,
            name: leak_cstr(format!("MIDI CC {cc} Ch {display_ch}")),
            short_name: leak_cstr(format!("CC{cc}")),
            units: leak_static(""),
            min: 0.0,
            max: 1.0,
            default_normalized: 0.0,
            step_count: 127,
            flags: 1,
            group: leak_static("MIDI CC Proxy"),
        });
    }
    descs
}
