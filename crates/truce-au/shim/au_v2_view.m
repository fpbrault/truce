/**
 * AU v2 Cocoa UI view factory.
 *
 * Defines `TruceAUCocoaViewProxy` — the `AUCocoaUIBase` class the
 * host instantiates after reading our `kAudioUnitProperty_CocoaUI`.
 * Compiled into every truce plugin dylib so the class appears in
 * `__objc_classlist`, which hosts like REAPER require for their
 * `[NSBundle classNamed:]`-based view lookup. Hosts that fall through
 * to `NSClassFromString` (Logic, auval) don't care, but the
 * statically-listed class works for them too.
 *
 * One stable class name across every plugin instead of a unique name
 * per dylib: Logic loads many .components into one process; if every
 * dylib publishes a different class name, each gets a unique
 * `__objc_classlist` entry too, but per-dylib unique names + static
 * registration would require recompiling this shim per plugin.
 * Keeping the name fixed means `libobjc` emits a duplicate-class
 * warning at load time and picks one dylib's class as "the winner".
 * The winner's method implementations dispatch through the
 * AudioUnit's private callbacks property (NOT through per-dylib
 * statics), so the AU instance always reaches its own plugin's
 * callbacks regardless of which dylib's copy of the class won.
 */

@import AppKit;
@import AudioToolbox;
#import <AudioUnit/AUCocoaUIView.h>

#include "au_shim_types.h"

// Private properties exposed by `au_v2_shim.c`:
//   64000: AuPlugin context pointer (rustCtx)
//   64001: pointer to the AU's AuCallbacks table (g_callbacks of the
//          dylib that owns this AudioUnit). Reading both via the AU
//          dispatch table makes this class plugin-agnostic — the
//          per-dylib globals reached are always the right ones.
#define kTrucePrivateProperty_RustContext  64000
#define kTrucePrivateProperty_AuCallbacks  64001

@interface TruceAUCocoaViewProxy : NSObject <AUCocoaUIBase>
@end

@implementation TruceAUCocoaViewProxy

- (unsigned)interfaceVersion {
    return 0;
}

- (NSView *)uiViewForAudioUnit:(AudioUnit)au withSize:(NSSize)preferredSize {
    void *ctx = NULL;
    UInt32 ctxSize = sizeof(ctx);
    if (AudioUnitGetProperty(au, kTrucePrivateProperty_RustContext,
            kAudioUnitScope_Global, 0, &ctx, &ctxSize) != noErr || !ctx) {
        return nil;
    }

    const AuCallbacks *cb = NULL;
    UInt32 cbSize = sizeof(cb);
    if (AudioUnitGetProperty(au, kTrucePrivateProperty_AuCallbacks,
            kAudioUnitScope_Global, 0, &cb, &cbSize) != noErr || !cb) {
        return nil;
    }

    if (!cb->gui_has_editor(ctx)) return nil;

    uint32_t w = 0, h = 0;
    cb->gui_get_size(ctx, &w, &h);
    if (w == 0 || h == 0) return nil;

    NSRect frame = NSMakeRect(0, 0, w, h);
    NSView *container = [[NSView alloc] initWithFrame:frame];
    cb->gui_open(ctx, (__bridge void *)container);
    return container;
}

@end

// Class-name lookup for the v2 shim's `kAudioUnitProperty_CocoaUI`
// response. Stable string — matches the `@interface` above.
const char *truce_au_view_factory_class_name(void) {
    return "TruceAUCocoaViewProxy";
}
