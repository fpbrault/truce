//! macOS native menu bar for the standalone host.
//!
//! Builds an `NSMenu` with two top-level items: an autopopulated
//! App menu (Quit / Hide / About) and a Plugin menu carrying:
//!
//! - **Mic Input** (toggle, ⌘I, checkmark when on)
//! - **Input Device** submenu — lists cpal-visible inputs, click to switch
//! - **Output Device** submenu — same for outputs
//!
//! Installed via `NSApp.setMainMenu(...)`.
//!
//! Action wiring uses a custom `TruceMenuTarget` Objective-C
//! class declared at runtime. The class has one ivar — a raw
//! pointer to a `MenuState` heap-allocated by Rust — and three
//! action selectors (`toggleInputAction:`, `selectInputDeviceAction:`,
//! `selectOutputDeviceAction:`) that dereference the pointer and
//! route the click to the matching `InputController` /
//! `OutputController` method.
//!
//! Menu state (the mic checkmark + the active-device checkmark in
//! each device submenu) is refreshed on `menuWillOpen:`. Device
//! submenus are also *repopulated* on each open from cpal's live
//! device list, so hot-plug is reflected without restarting.

#![cfg(all(target_os = "macos", feature = "gui"))]

use std::ffi::c_void;
use std::sync::Once;

use objc::declare::ClassDecl;
use objc::runtime::{Class, Object, Sel, BOOL, NO, YES};
use objc::{class, msg_send, sel, sel_impl};

use crate::audio::{self, InputController, OutputController};

/// Heap-allocated state the Objective-C class points at via ivar.
struct MenuState {
    input: InputController,
    output: OutputController,
    /// Mic-toggle item — checkmark refreshed on Plugin-menu open.
    mic_item: *mut Object,
    /// Input device submenu — repopulated on open from cpal.
    input_device_menu: *mut Object,
    /// Output device submenu — repopulated on open from cpal.
    output_device_menu: *mut Object,
    /// Pointer to the action-target object itself, for re-targeting
    /// the device items repopulated each open.
    target: *mut Object,
}

pub fn install(app_name: &str, input: InputController, output: OutputController) {
    unsafe {
        let app: *mut Object = msg_send![class!(NSApplication), sharedApplication];

        // App menu — "About <App>" / "Hide <App>" / "Quit <App>".
        let app_menu_item = make_menu_item(app_name);
        let app_menu = make_menu(app_name);
        add_app_menu_items(app_menu, app_name);
        let _: () = msg_send![app_menu_item, setSubmenu: app_menu];

        // Plugin menu and its action target.
        let plugin_menu_item = make_menu_item("Plugin");
        let plugin_menu = make_menu("Plugin");
        let target = make_menu_target(input.clone(), output.clone());

        // Mic toggle (⌘I).
        let mic_item = make_toggle_item("Mic Input", "i", target);
        let _: () = msg_send![plugin_menu, addItem: mic_item];

        // Separator before device pickers.
        let sep: *mut Object = msg_send![class!(NSMenuItem), separatorItem];
        let _: () = msg_send![plugin_menu, addItem: sep];

        // Input device submenu — empty at install; repopulated on open.
        let input_dev_item = make_menu_item("Input Device");
        let input_dev_menu = make_menu("Input Device");
        let _: () = msg_send![input_dev_item, setSubmenu: input_dev_menu];
        let _: () = msg_send![plugin_menu, addItem: input_dev_item];

        // Output device submenu — same.
        let output_dev_item = make_menu_item("Output Device");
        let output_dev_menu = make_menu("Output Device");
        let _: () = msg_send![output_dev_item, setSubmenu: output_dev_menu];
        let _: () = msg_send![plugin_menu, addItem: output_dev_item];

        // Stash pointers in MenuState so menu-open delegates can
        // address the right submenu.
        update_menu_state(target, mic_item, input_dev_menu, output_dev_menu, target);

        // Wire menuWillOpen on the Plugin menu (mic checkmark) and
        // both device submenus (repopulate + checkmark).
        let _: () = msg_send![plugin_menu, setDelegate: target];
        let _: () = msg_send![input_dev_menu, setDelegate: target];
        let _: () = msg_send![output_dev_menu, setDelegate: target];

        let _: () = msg_send![plugin_menu_item, setSubmenu: plugin_menu];

        // Main menu — the one NSApp draws.
        let main_menu = make_menu("");
        let _: () = msg_send![main_menu, addItem: app_menu_item];
        let _: () = msg_send![main_menu, addItem: plugin_menu_item];
        let _: () = msg_send![app, setMainMenu: main_menu];
    }
}

unsafe fn make_menu(title: &str) -> *mut Object {
    let title = ns_string(title);
    let menu: *mut Object = msg_send![class!(NSMenu), alloc];
    let menu: *mut Object = msg_send![menu, initWithTitle: title];
    menu
}

unsafe fn make_menu_item(title: &str) -> *mut Object {
    let title = ns_string(title);
    let empty = ns_string("");
    let item: *mut Object = msg_send![class!(NSMenuItem), alloc];
    let item: *mut Object = msg_send![
        item,
        initWithTitle: title
        action: sel!(noopAction:)
        keyEquivalent: empty
    ];
    item
}

unsafe fn make_toggle_item(title: &str, key_equiv: &str, target: *mut Object) -> *mut Object {
    let title = ns_string(title);
    let key = ns_string(key_equiv);
    let item: *mut Object = msg_send![class!(NSMenuItem), alloc];
    let item: *mut Object = msg_send![
        item,
        initWithTitle: title
        action: sel!(toggleInputAction:)
        keyEquivalent: key
    ];
    let _: () = msg_send![item, setTarget: target];
    item
}

/// Add the standard App-menu items. macOS does NOT auto-fill the
/// app name here — we have to spell out "Quit <App>" ourselves.
unsafe fn add_app_menu_items(menu: *mut Object, app_name: &str) {
    let title = ns_string(&format!("Quit {app_name}"));
    let key = ns_string("q");
    let quit_item: *mut Object = msg_send![class!(NSMenuItem), alloc];
    let quit_item: *mut Object = msg_send![
        quit_item,
        initWithTitle: title
        action: sel!(terminate:)
        keyEquivalent: key
    ];
    let _: () = msg_send![menu, addItem: quit_item];
}

/// Replace the contents of `menu` with a fresh device list. Items
/// fire `action` on `target`; the chosen item gets a checkmark
/// when its title matches `current`.
unsafe fn populate_device_menu(
    menu: *mut Object,
    target: *mut Object,
    devices: &[String],
    current: Option<&str>,
    action: Sel,
) {
    let _: () = msg_send![menu, removeAllItems];

    if devices.is_empty() {
        let title = ns_string("(no devices)");
        let empty = ns_string("");
        let item: *mut Object = msg_send![class!(NSMenuItem), alloc];
        let item: *mut Object = msg_send![
            item,
            initWithTitle: title
            action: sel!(noopAction:)
            keyEquivalent: empty
        ];
        let _: () = msg_send![item, setEnabled: NO];
        let _: () = msg_send![menu, addItem: item];
        return;
    }

    for name in devices {
        let title = ns_string(name);
        let empty = ns_string("");
        let item: *mut Object = msg_send![class!(NSMenuItem), alloc];
        let item: *mut Object = msg_send![
            item,
            initWithTitle: title
            action: action
            keyEquivalent: empty
        ];
        let _: () = msg_send![item, setTarget: target];
        let is_current = current.map(|c| c == name.as_str()).unwrap_or(false);
        let mark: BOOL = if is_current { YES } else { NO };
        let _: () = msg_send![item, setState: mark as i64];
        let _: () = msg_send![menu, addItem: item];
    }
}

unsafe fn ns_string(s: &str) -> *mut Object {
    let bytes = s.as_bytes();
    let cls = class!(NSString);
    let nsstr: *mut Object = msg_send![cls, alloc];
    let nsstr: *mut Object = msg_send![
        nsstr,
        initWithBytes: bytes.as_ptr() as *const c_void
        length: bytes.len()
        encoding: 4_usize // NSUTF8StringEncoding
    ];
    nsstr
}

/// Read an NSMenuItem's title back as a Rust String.
unsafe fn item_title(item: *mut Object) -> Option<String> {
    if item.is_null() {
        return None;
    }
    let nsstr: *mut Object = msg_send![item, title];
    if nsstr.is_null() {
        return None;
    }
    let cstr: *const std::os::raw::c_char = msg_send![nsstr, UTF8String];
    if cstr.is_null() {
        return None;
    }
    Some(
        std::ffi::CStr::from_ptr(cstr)
            .to_string_lossy()
            .into_owned(),
    )
}

// ---------------------------------------------------------------------------
// Custom TruceMenuTarget class
// ---------------------------------------------------------------------------

static REGISTER_CLASS: Once = Once::new();

const STATE_IVAR: &str = "_truce_menu_state";

fn ensure_class() -> &'static Class {
    REGISTER_CLASS.call_once(|| unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("TruceMenuTarget", superclass)
            .expect("TruceMenuTarget already registered (this should be unreachable — Once gate)");

        decl.add_ivar::<*mut c_void>(STATE_IVAR);

        // Mic toggle.
        extern "C" fn toggle_input_action(this: &Object, _: Sel, _sender: *mut Object) {
            unsafe {
                let Some(state) = state_from(this) else {
                    return;
                };
                let want = !state.input.is_enabled();
                state.input.set_enabled(want);
                eprintln!(
                    "[truce-standalone] mic: {} (request, via menu)",
                    if want { "ON" } else { "OFF" }
                );
                let new_state: BOOL = if want { YES } else { NO };
                let _: () = msg_send![_sender, setState: new_state as i64];
            }
        }
        decl.add_method(
            sel!(toggleInputAction:),
            toggle_input_action as extern "C" fn(&Object, Sel, *mut Object),
        );

        // Input device chosen.
        extern "C" fn select_input_device_action(this: &Object, _: Sel, sender: *mut Object) {
            unsafe {
                let Some(state) = state_from(this) else {
                    return;
                };
                if let Some(name) = item_title(sender) {
                    eprintln!("[truce-standalone] input device: {name}");
                    state.input.set_device(Some(name));
                }
            }
        }
        decl.add_method(
            sel!(selectInputDeviceAction:),
            select_input_device_action as extern "C" fn(&Object, Sel, *mut Object),
        );

        // Output device chosen.
        extern "C" fn select_output_device_action(this: &Object, _: Sel, sender: *mut Object) {
            unsafe {
                let Some(state) = state_from(this) else {
                    return;
                };
                if let Some(name) = item_title(sender) {
                    eprintln!("[truce-standalone] output device: {name}");
                    state.output.set_device(Some(name));
                }
            }
        }
        decl.add_method(
            sel!(selectOutputDeviceAction:),
            select_output_device_action as extern "C" fn(&Object, Sel, *mut Object),
        );

        // -(void) menuWillOpen:(NSMenu *)menu — refresh state for
        // the about-to-open menu. Dispatch by pointer comparison so
        // we know whether to refresh the mic checkmark or
        // repopulate a device submenu.
        extern "C" fn menu_will_open(this: &Object, _: Sel, menu: *mut Object) {
            unsafe {
                let Some(state) = state_from(this) else {
                    return;
                };

                if menu == state.input_device_menu {
                    let (_, names) = audio::list_input_devices();
                    let current = state.input.current_name();
                    populate_device_menu(
                        state.input_device_menu,
                        state.target,
                        &names,
                        current.as_deref(),
                        sel!(selectInputDeviceAction:),
                    );
                    return;
                }

                if menu == state.output_device_menu {
                    let (_, names) = audio::list_output_devices();
                    let current = state.output.current_name();
                    populate_device_menu(
                        state.output_device_menu,
                        state.target,
                        &names,
                        current.as_deref(),
                        sel!(selectOutputDeviceAction:),
                    );
                    return;
                }

                // Plugin menu (any other we delegate) — refresh
                // mic checkmark.
                if !state.mic_item.is_null() {
                    let on = state.input.is_enabled();
                    let new_state: BOOL = if on { YES } else { NO };
                    let _: () = msg_send![state.mic_item, setState: new_state as i64];
                }
            }
        }
        decl.add_method(
            sel!(menuWillOpen:),
            menu_will_open as extern "C" fn(&Object, Sel, *mut Object),
        );

        // Placeholder selector used by non-action items
        // (separators, disabled "(no devices)" rows).
        extern "C" fn noop_action(_: &Object, _: Sel, _sender: *mut Object) {}
        decl.add_method(
            sel!(noopAction:),
            noop_action as extern "C" fn(&Object, Sel, *mut Object),
        );

        decl.register();
    });
    Class::get("TruceMenuTarget").unwrap()
}

unsafe fn state_from<'a>(this: &Object) -> Option<&'a MenuState> {
    let state_ptr: *mut c_void = *this.get_ivar(STATE_IVAR);
    if state_ptr.is_null() {
        None
    } else {
        Some(&*(state_ptr as *const MenuState))
    }
}

unsafe fn make_menu_target(input: InputController, output: OutputController) -> *mut Object {
    let cls = ensure_class();
    let target: *mut Object = msg_send![cls, alloc];
    let target: *mut Object = msg_send![target, init];
    let state = Box::into_raw(Box::new(MenuState {
        input,
        output,
        mic_item: std::ptr::null_mut(),
        input_device_menu: std::ptr::null_mut(),
        output_device_menu: std::ptr::null_mut(),
        target: std::ptr::null_mut(),
    }));
    (*target).set_ivar::<*mut c_void>(STATE_IVAR, state as *mut c_void);
    target
}

unsafe fn update_menu_state(
    target: *mut Object,
    mic_item: *mut Object,
    input_device_menu: *mut Object,
    output_device_menu: *mut Object,
    target_self: *mut Object,
) {
    let state_ptr: *mut c_void = *(*target).get_ivar(STATE_IVAR);
    if state_ptr.is_null() {
        return;
    }
    let state = &mut *(state_ptr as *mut MenuState);
    state.mic_item = mic_item;
    state.input_device_menu = input_device_menu;
    state.output_device_menu = output_device_menu;
    state.target = target_self;
}
