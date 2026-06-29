/**
 * AU v2 Cocoa UI view factory.
 *
 * Defines the `AUCocoaUIBase` class the host instantiates after
 * reading our `kAudioUnitProperty_CocoaUI`. The class is compiled
 * into every truce plugin dylib so it appears in `__objc_classlist`,
 * which `[NSBundle classNamed:]`-based hosts (REAPER) require.
 *
 * The class name MUST be unique per plugin. AppKit/AudioUnit hosts
 * load every installed `.component` into one process; if two plugins
 * publish a class with the same name, `libobjc` keeps the first one
 * and `[NSBundle classNamed:name]` on the loser's bundle returns nil
 * - the host then thinks the plugin has no GUI. Uniqueness comes
 * from the `TRUCE_AU_PLUGIN_ID` env var that `cargo-truce` sets at
 * build time; the build.rs sanitises and passes it as a `-D` define.
 * Without that env (plain `cargo build` for unit tests), the class
 * falls back to a default name - fine for isolated tests, not for
 * multi-plugin hosting.
 */

@import AppKit;
@import AudioToolbox;
#import <AudioUnit/AUCocoaUIView.h>
#import <dispatch/dispatch.h>

#include "au_shim_types.h"

// Private properties exposed by `au_v2_shim.c`:
//   64000: AuPlugin context pointer (rustCtx)
//   64001: pointer to the AU's AuCallbacks table (g_callbacks of the
//          dylib that owns this AudioUnit). Reading both via the AU
//          dispatch table keeps the methods plugin-agnostic - per-
//          dylib globals reached are always the right ones.
#define kTrucePrivateProperty_RustContext  64000
#define kTrucePrivateProperty_AuCallbacks  64001

#ifndef TRUCE_AU_VIEW_FACTORY_NAME
// Default name when `TRUCE_AU_PLUGIN_ID` is unset - keeps `cargo build`
// of the workspace cdylibs working for unit tests.
#define TRUCE_AU_VIEW_FACTORY_NAME TruceAUCocoaViewProxy
#endif
#ifndef TRUCE_AU_FIXED_CONTAINER_NAME
#define TRUCE_AU_FIXED_CONTAINER_NAME TruceAuFixedContainer
#endif
#ifndef TRUCE_AU_RESIZE_HANDLE_NAME
#define TRUCE_AU_RESIZE_HANDLE_NAME TruceAuResizeHandle
#endif

static const CGFloat kTruceResizeHandleSize = 18.0;
static const CGFloat kTruceResizeHandleInset = 3.0;

@class TRUCE_AU_FIXED_CONTAINER_NAME;

@interface TRUCE_AU_RESIZE_HANDLE_NAME : NSView
@property(nonatomic, weak) TRUCE_AU_FIXED_CONTAINER_NAME *container;
@property(nonatomic, assign) NSPoint dragStartPoint;
@property(nonatomic, assign) NSSize dragStartSize;
@end

/// Container the host parents the editor into. AU v2 has no
/// standardised host-driven resize protocol, so resizable editors use an
/// AppKit corner handle that changes this NSView's frame. Hosts observe
/// the frame change through normal Cocoa layout notifications.
@interface TRUCE_AU_FIXED_CONTAINER_NAME : NSView
@property(nonatomic, assign) void *rustCtx;
@property(nonatomic, assign) const AuCallbacks *callbacks;
@property(nonatomic, strong) TRUCE_AU_RESIZE_HANDLE_NAME *resizeHandle;
- (void)syncResizeHandleVisibility;
- (void)bringResizeHandleToFront;
- (void)resizeFromHandleWithDelta:(NSSize)delta;
@end

@implementation TRUCE_AU_RESIZE_HANDLE_NAME

- (BOOL)isFlipped {
    return YES;
}

- (void)drawRect:(NSRect)dirtyRect {
    [super drawRect:dirtyRect];

    [[NSColor colorWithCalibratedWhite:0.72 alpha:0.75] setStroke];
    NSBezierPath *path = [NSBezierPath bezierPath];
    [path setLineWidth:1.0];

    CGFloat maxX = NSMaxX(self.bounds) - kTruceResizeHandleInset;
    CGFloat maxY = NSMaxY(self.bounds) - kTruceResizeHandleInset;
    for (NSInteger i = 0; i < 3; i++) {
        CGFloat offset = 5.0 + (CGFloat)i * 5.0;
        [path moveToPoint:NSMakePoint(maxX - offset, maxY)];
        [path lineToPoint:NSMakePoint(maxX, maxY - offset)];
    }
    [path stroke];
}

- (void)resetCursorRects {
    [self addCursorRect:self.bounds cursor:[NSCursor arrowCursor]];
}

- (void)mouseDown:(NSEvent *)event {
    self.dragStartPoint = [self.window convertPointToScreen:event.locationInWindow];
    self.dragStartSize = self.container.frame.size;
}

- (void)mouseDragged:(NSEvent *)event {
    NSPoint currentPoint = [self.window convertPointToScreen:event.locationInWindow];
    NSSize delta = NSMakeSize(
        currentPoint.x - self.dragStartPoint.x,
        self.dragStartPoint.y - currentPoint.y);
    [self.container resizeFromHandleWithDelta:delta];
}

@end

@implementation TRUCE_AU_FIXED_CONTAINER_NAME

- (instancetype)initWithFrame:(NSRect)frameRect {
    self = [super initWithFrame:frameRect];
    if (self) {
        self.autoresizesSubviews = YES;
        _resizeHandle = [[TRUCE_AU_RESIZE_HANDLE_NAME alloc]
            initWithFrame:NSMakeRect(NSWidth(frameRect) - kTruceResizeHandleSize,
                          0,
                          kTruceResizeHandleSize,
                          kTruceResizeHandleSize)];
        _resizeHandle.container = self;
        _resizeHandle.autoresizingMask = NSViewMinXMargin | NSViewMaxYMargin;
        _resizeHandle.hidden = YES;
        [self addSubview:_resizeHandle];
    }
    return self;
}

- (BOOL)isResizable {
    return self.rustCtx != NULL && self.callbacks != NULL &&
        self.callbacks->gui_can_resize(self.rustCtx) != 0;
}

- (NSSize)currentEditorSize {
    uint32_t w = 0, h = 0;
    if (self.rustCtx != NULL && self.callbacks != NULL) {
        self.callbacks->gui_get_size(self.rustCtx, &w, &h);
    }
    if (w > 0 && h > 0) {
        return NSMakeSize((CGFloat)w, (CGFloat)h);
    }
    return self.frame.size;
}

- (void)syncResizeHandleVisibility {
    self.resizeHandle.hidden = ![self isResizable];
    [self bringResizeHandleToFront];
}

- (void)bringResizeHandleToFront {
    if (self.resizeHandle.superview == self && !self.resizeHandle.hidden) {
        [self addSubview:self.resizeHandle positioned:NSWindowAbove relativeTo:nil];
        [self.resizeHandle setNeedsDisplay:YES];
    }
}

- (NSSize)applyEditorSize:(NSSize)requestedSize {
    if (self.rustCtx == NULL || self.callbacks == NULL) {
        return requestedSize;
    }

    if (![self isResizable]) {
        return [self currentEditorSize];
    }

    uint32_t requestedW = (uint32_t)MAX(1.0, requestedSize.width);
    uint32_t requestedH = (uint32_t)MAX(1.0, requestedSize.height);
    self.callbacks->gui_set_size(self.rustCtx, requestedW, requestedH);
    return [self currentEditorSize];
}

- (void)setFrameSize:(NSSize)newSize {
    NSSize acceptedSize = [self applyEditorSize:newSize];
    [super setFrameSize:acceptedSize];
    [self syncResizeHandleVisibility];
}

- (void)resizeFromHandleWithDelta:(NSSize)delta {
    NSSize requestedSize = NSMakeSize(
        self.resizeHandle.dragStartSize.width + delta.width,
        self.resizeHandle.dragStartSize.height + delta.height);
    NSSize acceptedSize = [self applyEditorSize:requestedSize];

    NSRect frame = self.frame;
    frame.origin.y -= acceptedSize.height - frame.size.height;
    frame.size = acceptedSize;
    [super setFrame:frame];
    [self syncResizeHandleVisibility];
    [self bringResizeHandleToFront];
}
@end

@interface TRUCE_AU_VIEW_FACTORY_NAME : NSObject <AUCocoaUIBase>
@end

@implementation TRUCE_AU_VIEW_FACTORY_NAME

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
    (void)preferredSize;

    NSRect frame = NSMakeRect(0, 0, w, h);
    TRUCE_AU_FIXED_CONTAINER_NAME *container =
        [[TRUCE_AU_FIXED_CONTAINER_NAME alloc] initWithFrame:frame];
    container.rustCtx = ctx;
    container.callbacks = cb;
    [container syncResizeHandleVisibility];
    // PATCH (cosmo): defer gui_open until the host has attached our
    // container to its window. The Cosmo editor validates that the
    // parent NSView already belongs to a window before embedding the
    // WKWebView child, and AUv2 hosts often call this factory method
    // before the view hierarchy is live.
    dispatch_async(dispatch_get_main_queue(), ^{
        cb->gui_open(ctx, (__bridge void *)container);
        [container syncResizeHandleVisibility];
        [container bringResizeHandleToFront];
    });
    return container;
}

@end

// Stringify the class name for the v2 shim's `kAudioUnitProperty_CocoaUI`
// response. Two-step macro so the argument is expanded before stringification.
#define _TRUCE_STRINGIFY(x) #x
#define TRUCE_STRINGIFY(x) _TRUCE_STRINGIFY(x)

const char *truce_au_view_factory_class_name(void) {
    return TRUCE_STRINGIFY(TRUCE_AU_VIEW_FACTORY_NAME);
}
