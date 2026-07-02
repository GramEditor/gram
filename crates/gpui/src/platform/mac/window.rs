use super::{BoolExt, MacDisplay, NSRange, NSStringExt, ns_string};
use crate::{
    AnyWindowHandle, Bounds, Capslock, DevicePixels, DisplayLink, ExternalPaths, FileDropEvent,
    ForegroundExecutor, KeyDownEvent, Keystroke, Modifiers, MouseMoveEvent, Pixels, PlatformAtlas,
    PlatformDisplay, PlatformInput, PlatformWindow, Point, PromptButton, PromptLevel,
    RequestFrameOptions, SharedString, Size, SystemWindowTab, Timer, WindowAppearance,
    WindowBackgroundAppearance, WindowBounds, WindowControlArea, WindowKind, WindowParams,
    dispatch_get_main_queue,
    dispatch_sys::dispatch_async_f,
    platform::{
        PlatformInputHandler,
        mac::events::ESCAPE_KEY,
        wgpu::{WgpuContext, WgpuRenderer, WgpuSurfaceConfig},
    },
    point, px, size,
};
use block2::RcBlock;
use cocoa::{
    appkit::{
        NSAppKitVersionNumber, NSAppKitVersionNumber12_0, NSApplication, NSBackingStoreBuffered,
        NSColor, NSEvent, NSEventModifierFlags, NSFilenamesPboardType, NSPasteboard, NSScreen,
        NSView, NSWindow, NSWindowButton, NSWindowCollectionBehavior, NSWindowOcclusionState,
        NSWindowOrderingMode, NSWindowStyleMask, NSWindowTitleVisibility,
    },
    base::{id, nil},
    foundation::{
        NSArray, NSAutoreleasePool, NSDictionary, NSFastEnumeration, NSInteger,
        NSOperatingSystemVersion, NSPoint, NSProcessInfo, NSRect, NSSize, NSString, NSUInteger,
        NSUserDefaults,
    },
};

use core_graphics::display::{CGDirectDisplayID, CGPoint, CGRect};
use ctor::ctor;
use futures::channel::oneshot;
use objc::{
    class,
    declare::ClassDecl,
    msg_send,
    runtime::{BOOL, Class, NO, Object, Sel, YES},
    sel, sel_impl,
};
use objc2::rc::Retained;
use objc2_app_kit::{NSAlert, NSAlertStyle, NSButton as Objc2NSButton, NSWindow as Objc2NSWindow};
use objc2_foundation::MainThreadMarker;
use parking_lot::Mutex;
use raw_window_handle as rwh;
use smallvec::SmallVec;
use std::{
    cell::Cell,
    ffi::{CStr, c_void},
    mem,
    path::PathBuf,
    ptr::{self, NonNull},
    rc::Rc,
    sync::{Arc, Weak},
    time::Duration,
};
use util::ResultExt;
use wgpu::PresentMode;

const WINDOW_STATE_IVAR: &str = "windowState";

static mut WINDOW_CLASS: *const Class = ptr::null();
static mut PANEL_CLASS: *const Class = ptr::null();

#[allow(non_upper_case_globals)]
const NSWindowStyleMaskNonactivatingPanel: NSWindowStyleMask =
    NSWindowStyleMask::from_bits_retain(1 << 7);
#[allow(non_upper_case_globals)]
const NSNormalWindowLevel: NSInteger = 0;
#[allow(non_upper_case_globals)]
const NSPopUpWindowLevel: NSInteger = 101;
#[allow(non_upper_case_globals)]
const NSTrackingMouseEnteredAndExited: NSUInteger = 0x01;
#[allow(non_upper_case_globals)]
const NSTrackingMouseMoved: NSUInteger = 0x02;
#[allow(non_upper_case_globals)]
const NSTrackingActiveAlways: NSUInteger = 0x80;
#[allow(non_upper_case_globals)]
const NSTrackingInVisibleRect: NSUInteger = 0x200;
#[allow(non_upper_case_globals)]
const NSWindowAnimationBehaviorUtilityWindow: NSInteger = 4;
// https://developer.apple.com/documentation/appkit/nsdragoperation
type NSDragOperation = NSUInteger;
#[allow(non_upper_case_globals)]
const NSDragOperationNone: NSDragOperation = 0;
#[allow(non_upper_case_globals)]
const NSDragOperationCopy: NSDragOperation = 1;
#[derive(PartialEq)]
pub enum UserTabbingPreference {
    Never,
    Always,
    InFullScreen,
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    // Widely used private APIs; Apple uses them for their Terminal.app.
    fn CGSMainConnectionID() -> id;
    fn CGSSetWindowBackgroundBlurRadius(
        connection_id: id,
        window_id: NSInteger,
        radius: i64,
    ) -> i32;
}

// declare_class!(
//     #[unsafe(super = NSWindow)]
//     pub struct GPUIWindow;
// );

// declare_class!(
//     #[unsafe(super = NSPanel)]
//     pub struct GPUIPanel;
// );

#[ctor]
unsafe fn build_classes() {
    unsafe {
        WINDOW_CLASS = build_window_class("GPUIWindow", class!(NSWindow));
        PANEL_CLASS = build_window_class("GPUIPanel", class!(NSPanel));
    }
}

pub(crate) fn convert_mouse_position(position: NSPoint, window_height: Pixels) -> Point<Pixels> {
    point(
        px(position.x as f32),
        // macOS screen coordinates are relative to bottom left
        window_height - px(position.y as f32),
    )
}

unsafe fn build_window_class(name: &'static str, superclass: &Class) -> *const Class {
    unsafe {
        let mut decl = ClassDecl::new(name, superclass).unwrap();
        decl.add_ivar::<*mut c_void>(WINDOW_STATE_IVAR);
        decl.add_method(sel!(dealloc), dealloc_window as extern "C" fn(&Object, Sel));

        decl.add_method(
            sel!(canBecomeMainWindow),
            yes as extern "C" fn(&Object, Sel) -> BOOL,
        );
        decl.add_method(
            sel!(canBecomeKeyWindow),
            yes as extern "C" fn(&Object, Sel) -> BOOL,
        );
        decl.add_method(
            sel!(windowDidResize:),
            window_did_resize as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(windowDidChangeOcclusionState:),
            window_did_change_occlusion_state as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(windowWillEnterFullScreen:),
            window_will_enter_fullscreen as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(windowWillExitFullScreen:),
            window_will_exit_fullscreen as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(windowDidMove:),
            window_did_move as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(windowDidChangeScreen:),
            window_did_change_screen as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(windowDidBecomeKey:),
            window_did_change_key_status as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(windowDidResignKey:),
            window_did_change_key_status as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(windowShouldClose:),
            window_should_close as extern "C" fn(&Object, Sel, id) -> BOOL,
        );

        decl.add_method(sel!(close), close_window as extern "C" fn(&Object, Sel));

        decl.add_method(
            sel!(draggingEntered:),
            dragging_entered as extern "C" fn(&Object, Sel, id) -> NSDragOperation,
        );
        decl.add_method(
            sel!(draggingUpdated:),
            dragging_updated as extern "C" fn(&Object, Sel, id) -> NSDragOperation,
        );
        decl.add_method(
            sel!(draggingExited:),
            dragging_exited as extern "C" fn(&Object, Sel, id),
        );
        decl.add_method(
            sel!(performDragOperation:),
            perform_drag_operation as extern "C" fn(&Object, Sel, id) -> BOOL,
        );
        decl.add_method(
            sel!(concludeDragOperation:),
            conclude_drag_operation as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(addTitlebarAccessoryViewController:),
            add_titlebar_accessory_view_controller as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(moveTabToNewWindow:),
            move_tab_to_new_window as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(mergeAllWindows:),
            merge_all_windows as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(selectNextTab:),
            select_next_tab as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(selectPreviousTab:),
            select_previous_tab as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(toggleTabBar:),
            toggle_tab_bar as extern "C" fn(&Object, Sel, id),
        );

        decl.register()
    }
}

pub(crate) struct MacWindowState {
    handle: AnyWindowHandle,
    pub(crate) executor: ForegroundExecutor,
    pub(crate) native_window: id,
    native_view: NonNull<Object>,
    blurred_view: Option<id>,
    background_appearance: WindowBackgroundAppearance,
    display_link: Option<DisplayLink>,
    pub(crate) renderer: WgpuRenderer,
    pub(crate) request_frame_callback: Option<Box<dyn FnMut(RequestFrameOptions)>>,
    pub(crate) event_callback: Option<Box<dyn FnMut(PlatformInput) -> crate::DispatchEventResult>>,
    activate_callback: Option<Box<dyn FnMut(bool)>>,
    pub(crate) resize_callback: Option<Box<dyn FnMut(Size<Pixels>, f32)>>,
    moved_callback: Option<Box<dyn FnMut()>>,
    should_close_callback: Option<Box<dyn FnMut() -> bool>>,
    close_callback: Option<Box<dyn FnOnce()>>,
    pub(crate) appearance_changed_callback: Option<Box<dyn FnMut()>>,
    pub(crate) input_handler: Option<PlatformInputHandler>,
    pub(crate) last_key_equivalent: Option<KeyDownEvent>,
    pub(crate) synthetic_drag_counter: usize,
    traffic_light_position: Option<Point<Pixels>>,
    transparent_titlebar: bool,
    pub(crate) previous_modifiers_changed_event: Option<PlatformInput>,
    pub(crate) keystroke_for_do_command: Option<Keystroke>,
    pub(crate) do_command_handled: Option<bool>,
    pub(crate) external_files_dragged: bool,
    // Whether the next left-mouse click is also the focusing click.
    pub(crate) first_mouse: bool,
    fullscreen_restore_bounds: Bounds<Pixels>,
    move_tab_to_new_window_callback: Option<Box<dyn FnMut()>>,
    merge_all_windows_callback: Option<Box<dyn FnMut()>>,
    select_next_tab_callback: Option<Box<dyn FnMut()>>,
    select_previous_tab_callback: Option<Box<dyn FnMut()>>,
    toggle_tab_bar_callback: Option<Box<dyn FnMut()>>,
    activated_least_once: bool,
}

impl MacWindowState {
    pub(crate) fn move_traffic_light(&self) {
        if let Some(traffic_light_position) = self.traffic_light_position {
            if self.is_fullscreen() {
                // Moving traffic lights while fullscreen doesn't work,
                // see https://github.com/zed-industries/zed/issues/4712
                return;
            }

            let titlebar_height = self.titlebar_height();

            unsafe {
                let close_button: id = msg_send![
                    self.native_window,
                    standardWindowButton: NSWindowButton::NSWindowCloseButton
                ];
                let min_button: id = msg_send![
                    self.native_window,
                    standardWindowButton: NSWindowButton::NSWindowMiniaturizeButton
                ];
                let zoom_button: id = msg_send![
                    self.native_window,
                    standardWindowButton: NSWindowButton::NSWindowZoomButton
                ];

                let mut close_button_frame: CGRect = msg_send![close_button, frame];
                let mut min_button_frame: CGRect = msg_send![min_button, frame];
                let mut zoom_button_frame: CGRect = msg_send![zoom_button, frame];
                let mut origin = point(
                    traffic_light_position.x,
                    titlebar_height
                        - traffic_light_position.y
                        - px(close_button_frame.size.height as f32),
                );
                let button_spacing =
                    px((min_button_frame.origin.x - close_button_frame.origin.x) as f32);

                close_button_frame.origin = CGPoint::new(origin.x.into(), origin.y.into());
                let _: () = msg_send![close_button, setFrame: close_button_frame];
                origin.x += button_spacing;

                min_button_frame.origin = CGPoint::new(origin.x.into(), origin.y.into());
                let _: () = msg_send![min_button, setFrame: min_button_frame];
                origin.x += button_spacing;

                zoom_button_frame.origin = CGPoint::new(origin.x.into(), origin.y.into());
                let _: () = msg_send![zoom_button, setFrame: zoom_button_frame];
                origin.x += button_spacing;
            }
        }
    }

    pub fn start_display_link(&mut self) {
        self.stop_display_link();
        unsafe {
            if !self
                .native_window
                .occlusionState()
                .contains(NSWindowOcclusionState::NSWindowOcclusionStateVisible)
            {
                return;
            }
        }
        let display_id = unsafe { display_id_for_screen(self.native_window.screen()) };
        if let Some(mut display_link) =
            DisplayLink::new(display_id, self.native_view.as_ptr() as *mut c_void, step).log_err()
        {
            display_link.start().log_err();
            self.display_link = Some(display_link);
        }
    }

    pub fn stop_display_link(&mut self) {
        self.display_link = None;
    }

    fn is_maximized(&self) -> bool {
        unsafe {
            let bounds = self.bounds();
            let screen_size = self.native_window.screen().visibleFrame().into();
            bounds.size == screen_size
        }
    }

    fn is_fullscreen(&self) -> bool {
        unsafe {
            let style_mask = self.native_window.styleMask();
            style_mask.contains(NSWindowStyleMask::NSFullScreenWindowMask)
        }
    }

    fn bounds(&self) -> Bounds<Pixels> {
        let mut window_frame = unsafe { NSWindow::frame(self.native_window) };
        let screen = unsafe { NSWindow::screen(self.native_window) };
        if screen == nil {
            return Bounds::new(point(px(0.), px(0.)), crate::DEFAULT_WINDOW_SIZE);
        }
        let screen_frame = unsafe { NSScreen::frame(screen) };

        // Flip the y coordinate to be top-left origin
        window_frame.origin.y =
            screen_frame.size.height - window_frame.origin.y - window_frame.size.height;

        Bounds::new(
            point(
                px((window_frame.origin.x - screen_frame.origin.x) as f32),
                px((window_frame.origin.y + screen_frame.origin.y) as f32),
            ),
            size(
                px(window_frame.size.width as f32),
                px(window_frame.size.height as f32),
            ),
        )
    }

    pub fn content_size(&self) -> Size<Pixels> {
        let NSSize { width, height, .. } =
            unsafe { NSView::frame(self.native_window.contentView()) }.size;
        size(px(width as f32), px(height as f32))
    }

    pub fn scale_factor(&self) -> f32 {
        get_scale_factor(self.native_window)
    }

    fn titlebar_height(&self) -> Pixels {
        unsafe {
            let frame = NSWindow::frame(self.native_window);
            let content_layout_rect: CGRect = msg_send![self.native_window, contentLayoutRect];
            px((frame.size.height - content_layout_rect.size.height) as f32)
        }
    }

    fn window_bounds(&self) -> WindowBounds {
        if self.is_fullscreen() {
            WindowBounds::Fullscreen(self.fullscreen_restore_bounds)
        } else {
            WindowBounds::Windowed(self.bounds())
        }
    }
}

unsafe impl Send for MacWindowState {}

pub(crate) struct MacWindow(Arc<Mutex<MacWindowState>>);

struct RawWindow {
    view: id,
}

unsafe impl Send for RawWindow {}
unsafe impl Sync for RawWindow {}

impl rwh::HasWindowHandle for RawWindow {
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        let view = NonNull::<c_void>::new(self.view as *mut c_void).unwrap();
        let handle = rwh::AppKitWindowHandle::new(view);
        Ok(unsafe { rwh::WindowHandle::borrow_raw(handle.into()) })
    }
}

impl rwh::HasDisplayHandle for RawWindow {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        let handle = rwh::RawDisplayHandle::AppKit(rwh::AppKitDisplayHandle::new());
        Ok(unsafe { rwh::DisplayHandle::borrow_raw(handle) })
    }
}

impl MacWindow {
    pub fn open(
        handle: AnyWindowHandle,
        WindowParams {
            bounds,
            titlebar,
            kind,
            is_movable,
            is_resizable,
            is_minimizable,
            focus,
            show,
            display_id,
            window_min_size,
            tabbing_identifier,
        }: WindowParams,
        executor: ForegroundExecutor,
        renderer_context: &WgpuContext,
    ) -> Self {
        unsafe {
            let pool = NSAutoreleasePool::new(nil);

            let allows_automatic_window_tabbing = tabbing_identifier.is_some();
            if allows_automatic_window_tabbing {
                let () = msg_send![class!(NSWindow), setAllowsAutomaticWindowTabbing: YES];
            } else {
                let () = msg_send![class!(NSWindow), setAllowsAutomaticWindowTabbing: NO];
            }

            let mut style_mask;
            if let Some(titlebar) = titlebar.as_ref() {
                style_mask =
                    NSWindowStyleMask::NSClosableWindowMask | NSWindowStyleMask::NSTitledWindowMask;

                if is_resizable {
                    style_mask |= NSWindowStyleMask::NSResizableWindowMask;
                }

                if is_minimizable {
                    style_mask |= NSWindowStyleMask::NSMiniaturizableWindowMask;
                }

                if titlebar.appears_transparent {
                    style_mask |= NSWindowStyleMask::NSFullSizeContentViewWindowMask;
                }
            } else {
                style_mask = NSWindowStyleMask::NSTitledWindowMask
                    | NSWindowStyleMask::NSFullSizeContentViewWindowMask;
            }

            let native_window: id = match kind {
                WindowKind::Normal | WindowKind::Floating => msg_send![WINDOW_CLASS, alloc],
                WindowKind::PopUp => {
                    style_mask |= NSWindowStyleMaskNonactivatingPanel;
                    msg_send![PANEL_CLASS, alloc]
                }
            };

            let display = display_id
                .and_then(MacDisplay::find_by_id)
                .unwrap_or_else(MacDisplay::primary);

            let mut target_screen = nil;
            let mut screen_frame = None;

            let screens = NSScreen::screens(nil);
            let count: u64 = cocoa::foundation::NSArray::count(screens);
            for i in 0..count {
                let screen = cocoa::foundation::NSArray::objectAtIndex(screens, i);
                let frame = NSScreen::frame(screen);
                let display_id = display_id_for_screen(screen);
                if display_id == display.0 {
                    screen_frame = Some(frame);
                    target_screen = screen;
                }
            }

            let screen_frame = screen_frame.unwrap_or_else(|| {
                let screen = NSScreen::mainScreen(nil);
                target_screen = screen;
                NSScreen::frame(screen)
            });

            let window_rect = NSRect::new(
                NSPoint::new(
                    screen_frame.origin.x + bounds.origin.x.0 as f64,
                    screen_frame.origin.y
                        + (display.bounds().size.height - bounds.origin.y).0 as f64,
                ),
                NSSize::new(bounds.size.width.0 as f64, bounds.size.height.0 as f64),
            );

            let native_window = native_window.initWithContentRect_styleMask_backing_defer_screen_(
                window_rect,
                style_mask,
                NSBackingStoreBuffered,
                NO,
                target_screen,
            );
            assert!(!native_window.is_null());
            let () = msg_send![
                native_window,
                registerForDraggedTypes:
                    NSArray::arrayWithObject(nil, NSFilenamesPboardType)
            ];
            let () = msg_send![
                native_window,
                setReleasedWhenClosed: NO
            ];

            let content_view = native_window.contentView();
            let native_view = {
                let mtm = MainThreadMarker::new().expect("Must be called from the main thread");
                let bounds = NSView::bounds(content_view);
                crate::platform::mac::gpui_view::GPUIView::new(mtm, to_objc2_rect(bounds))
            };

            let renderer = {
                let raw_window = RawWindow {
                    view: crate::platform::mac::gpui_view::GPUIView::as_obj_ptr(
                        native_view.clone(),
                    ),
                };
                let surface_config = WgpuSurfaceConfig {
                    size: Size {
                        width: DevicePixels(bounds.size.width.0 as i32),
                        height: DevicePixels(bounds.size.height.0 as i32),
                    },
                    transparent: true,
                    preferred_present_mode: Some(PresentMode::Immediate),
                };

                WgpuRenderer::new(renderer_context, &raw_window, surface_config).unwrap()
            };

            let mut window = Self(Arc::new(Mutex::new(MacWindowState {
                handle,
                executor,
                native_window,
                native_view: NonNull::new_unchecked(
                    crate::platform::mac::gpui_view::GPUIView::as_obj_ptr(native_view.clone()),
                ),
                blurred_view: None,
                background_appearance: WindowBackgroundAppearance::Opaque,
                display_link: None,
                renderer,
                request_frame_callback: None,
                event_callback: None,
                activate_callback: None,
                resize_callback: None,
                moved_callback: None,
                should_close_callback: None,
                close_callback: None,
                appearance_changed_callback: None,
                input_handler: None,
                last_key_equivalent: None,
                synthetic_drag_counter: 0,
                traffic_light_position: titlebar
                    .as_ref()
                    .and_then(|titlebar| titlebar.traffic_light_position),
                transparent_titlebar: titlebar
                    .as_ref()
                    .is_none_or(|titlebar| titlebar.appears_transparent),
                previous_modifiers_changed_event: None,
                keystroke_for_do_command: None,
                do_command_handled: None,
                external_files_dragged: false,
                first_mouse: false,
                fullscreen_restore_bounds: Bounds::default(),
                move_tab_to_new_window_callback: None,
                merge_all_windows_callback: None,
                select_next_tab_callback: None,
                select_previous_tab_callback: None,
                toggle_tab_bar_callback: None,
                activated_least_once: false,
            })));

            (*native_window).set_ivar(
                WINDOW_STATE_IVAR,
                Arc::into_raw(window.0.clone()) as *const c_void,
            );
            native_window.setDelegate_(native_window);
            native_view.set_window_state(window.0.clone());

            if let Some(title) = titlebar
                .as_ref()
                .and_then(|t| t.title.as_ref().map(AsRef::as_ref))
            {
                window.set_title(title);
            }

            native_window.setMovable_(is_movable as BOOL);

            if let Some(window_min_size) = window_min_size {
                native_window.setContentMinSize_(NSSize {
                    width: window_min_size.width.to_f64(),
                    height: window_min_size.height.to_f64(),
                });
            }

            if titlebar.is_none_or(|titlebar| titlebar.appears_transparent) {
                native_window.setTitlebarAppearsTransparent_(YES);
                native_window.setTitleVisibility_(NSWindowTitleVisibility::NSWindowTitleHidden);
            }

            native_view.setAutoresizingMask(
                objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                    | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
            );
            #[allow(deprecated)]
            native_view.setWantsBestResolutionOpenGLSurface(true);

            // From winit crate: On Mojave, views automatically become layer-backed shortly after
            // being added to a native_window. Changing the layer-backedness of a view breaks the
            // association between the view and its associated OpenGL context. To work around this,
            // on we explicitly make the view layer-backed up front so that AppKit doesn't do it
            // itself and break the association with its context.
            native_view.setWantsLayer(true);
            native_view.setLayerContentsRedrawPolicy(
                objc2_app_kit::NSViewLayerContentsRedrawPolicy::DuringViewResize,
            );

            let native_view_ptr =
                crate::platform::mac::gpui_view::GPUIView::as_obj_ptr(native_view.clone());
            content_view.addSubview_(native_view_ptr);
            native_window.makeFirstResponder_(native_view_ptr);

            match kind {
                WindowKind::Normal | WindowKind::Floating => {
                    native_window.setLevel_(NSNormalWindowLevel);
                    native_window.setAcceptsMouseMovedEvents_(YES);

                    if let Some(tabbing_identifier) = tabbing_identifier {
                        let tabbing_id = ns_string(tabbing_identifier.as_str());
                        let _: () = msg_send![native_window, setTabbingIdentifier: tabbing_id];
                    } else {
                        let _: () = msg_send![native_window, setTabbingIdentifier:nil];
                    }
                }
                WindowKind::PopUp => {
                    // Use a tracking area to allow receiving MouseMoved events even when
                    // the window or application aren't active, which is often the case
                    // e.g. for notification windows.
                    let tracking_area: id = msg_send![class!(NSTrackingArea), alloc];
                    let _: () = msg_send![
                        tracking_area,
                        initWithRect: NSRect::new(NSPoint::new(0., 0.), NSSize::new(0., 0.))
                        options: NSTrackingMouseEnteredAndExited | NSTrackingMouseMoved | NSTrackingActiveAlways | NSTrackingInVisibleRect
                        owner: native_view_ptr
                        userInfo: nil
                    ];
                    native_view.addTrackingArea(
                        &*(tracking_area.autorelease() as *mut objc2::runtime::AnyObject
                            as *const objc2_app_kit::NSTrackingArea),
                    );

                    native_window.setLevel_(NSPopUpWindowLevel);
                    let _: () = msg_send![
                        native_window,
                        setAnimationBehavior: NSWindowAnimationBehaviorUtilityWindow
                    ];
                    native_window.setCollectionBehavior_(
                        NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces |
                        NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
                    );
                }
            }

            let app = NSApplication::sharedApplication(nil);
            let main_window: id = msg_send![app, mainWindow];
            if allows_automatic_window_tabbing
                && !main_window.is_null()
                && main_window != native_window
            {
                let main_window_is_fullscreen = main_window
                    .styleMask()
                    .contains(NSWindowStyleMask::NSFullScreenWindowMask);
                let user_tabbing_preference = Self::get_user_tabbing_preference()
                    .unwrap_or(UserTabbingPreference::InFullScreen);
                let should_add_as_tab = user_tabbing_preference == UserTabbingPreference::Always
                    || user_tabbing_preference == UserTabbingPreference::InFullScreen
                        && main_window_is_fullscreen;

                if should_add_as_tab {
                    let main_window_can_tab: BOOL =
                        msg_send![main_window, respondsToSelector: sel!(addTabbedWindow:ordered:)];
                    let main_window_visible: BOOL = msg_send![main_window, isVisible];

                    if main_window_can_tab == YES && main_window_visible == YES {
                        let _: () = msg_send![main_window, addTabbedWindow: native_window ordered: NSWindowOrderingMode::NSWindowAbove];

                        // Ensure the window is visible immediately after adding the tab, since the tab bar is updated with a new entry at this point.
                        // Note: Calling orderFront here can break fullscreen mode (makes fullscreen windows exit fullscreen), so only do this if the main window is not fullscreen.
                        if !main_window_is_fullscreen {
                            let _: () = msg_send![native_window, orderFront: nil];
                        }
                    }
                }
            }

            if focus && show {
                native_window.makeKeyAndOrderFront_(nil);
            } else if show {
                native_window.orderFront_(nil);
            }

            // Set the initial position of the window to the specified origin.
            // Although we already specified the position using `initWithContentRect_styleMask_backing_defer_screen_`,
            // the window position might be incorrect if the main screen (the screen that contains the window that has focus)
            //  is different from the primary screen.
            NSWindow::setFrameTopLeftPoint_(native_window, window_rect.origin);
            window.0.lock().move_traffic_light();

            pool.drain();

            window
        }
    }

    pub fn active_window() -> Option<AnyWindowHandle> {
        unsafe {
            let app = NSApplication::sharedApplication(nil);
            let main_window: id = msg_send![app, mainWindow];
            if main_window.is_null() {
                return None;
            }

            if msg_send![main_window, isKindOfClass: WINDOW_CLASS] {
                let handle = get_window_state(&*main_window).lock().handle;
                Some(handle)
            } else {
                None
            }
        }
    }

    pub fn ordered_windows() -> Vec<AnyWindowHandle> {
        unsafe {
            let app = NSApplication::sharedApplication(nil);
            let windows: id = msg_send![app, orderedWindows];
            let count: NSUInteger = msg_send![windows, count];

            let mut window_handles = Vec::new();
            for i in 0..count {
                let window: id = msg_send![windows, objectAtIndex:i];
                if msg_send![window, isKindOfClass: WINDOW_CLASS] {
                    let handle = get_window_state(&*window).lock().handle;
                    window_handles.push(handle);
                }
            }

            window_handles
        }
    }

    pub fn get_user_tabbing_preference() -> Option<UserTabbingPreference> {
        unsafe {
            let defaults: id = NSUserDefaults::standardUserDefaults();
            let domain = ns_string("NSGlobalDomain");
            let key = ns_string("AppleWindowTabbingMode");

            let dict: id = msg_send![defaults, persistentDomainForName: domain];
            let value: id = if !dict.is_null() {
                msg_send![dict, objectForKey: key]
            } else {
                nil
            };

            let value_str = if !value.is_null() {
                CStr::from_ptr(NSString::UTF8String(value)).to_string_lossy()
            } else {
                "".into()
            };

            match value_str.as_ref() {
                "manual" => Some(UserTabbingPreference::Never),
                "always" => Some(UserTabbingPreference::Always),
                _ => Some(UserTabbingPreference::InFullScreen),
            }
        }
    }
}

impl Drop for MacWindow {
    fn drop(&mut self) {
        let mut this = self.0.lock();
        this.renderer.destroy();
        let window = this.native_window;
        this.display_link.take();
        unsafe {
            this.native_window.setDelegate_(nil);
        }
        this.input_handler.take();
        this.executor
            .spawn(async move {
                unsafe {
                    window.close();
                    window.autorelease();
                }
            })
            .detach();
    }
}

impl PlatformWindow for MacWindow {
    fn bounds(&self) -> Bounds<Pixels> {
        self.0.as_ref().lock().bounds()
    }

    fn window_bounds(&self) -> WindowBounds {
        self.0.as_ref().lock().window_bounds()
    }

    fn is_maximized(&self) -> bool {
        self.0.as_ref().lock().is_maximized()
    }

    fn content_size(&self) -> Size<Pixels> {
        self.0.as_ref().lock().content_size()
    }

    fn resize(&mut self, size: Size<Pixels>) {
        let this = self.0.lock();
        let window = this.native_window;
        this.executor
            .spawn(async move {
                unsafe {
                    window.setContentSize_(NSSize {
                        width: size.width.0 as f64,
                        height: size.height.0 as f64,
                    });
                }
            })
            .detach();
    }

    fn merge_all_windows(&self) {
        let native_window = self.0.lock().native_window;
        unsafe extern "C" fn merge_windows_async(context: *mut std::ffi::c_void) {
            let native_window = context as id;
            let _: () = msg_send![native_window, mergeAllWindows:nil];
        }

        unsafe {
            dispatch_async_f(
                dispatch_get_main_queue(),
                native_window as *mut std::ffi::c_void,
                Some(merge_windows_async),
            );
        }
    }

    fn move_tab_to_new_window(&self) {
        let native_window = self.0.lock().native_window;
        unsafe extern "C" fn move_tab_async(context: *mut std::ffi::c_void) {
            let native_window = context as id;
            let _: () = msg_send![native_window, moveTabToNewWindow:nil];
            let _: () = msg_send![native_window, makeKeyAndOrderFront: nil];
        }

        unsafe {
            dispatch_async_f(
                dispatch_get_main_queue(),
                native_window as *mut std::ffi::c_void,
                Some(move_tab_async),
            );
        }
    }

    fn toggle_window_tab_overview(&self) {
        let native_window = self.0.lock().native_window;
        unsafe {
            let _: () = msg_send![native_window, toggleTabOverview:nil];
        }
    }

    fn set_tabbing_identifier(&self, tabbing_identifier: Option<String>) {
        let native_window = self.0.lock().native_window;
        unsafe {
            let allows_automatic_window_tabbing = tabbing_identifier.is_some();
            if allows_automatic_window_tabbing {
                let () = msg_send![class!(NSWindow), setAllowsAutomaticWindowTabbing: YES];
            } else {
                let () = msg_send![class!(NSWindow), setAllowsAutomaticWindowTabbing: NO];
            }

            if let Some(tabbing_identifier) = tabbing_identifier {
                let tabbing_id = ns_string(tabbing_identifier.as_str());
                let _: () = msg_send![native_window, setTabbingIdentifier: tabbing_id];
            } else {
                let _: () = msg_send![native_window, setTabbingIdentifier:nil];
            }
        }
    }

    fn scale_factor(&self) -> f32 {
        self.0.as_ref().lock().scale_factor()
    }

    fn appearance(&self) -> WindowAppearance {
        unsafe {
            let appearance: id = msg_send![self.0.lock().native_window, effectiveAppearance];
            let obj: &objc2::runtime::AnyObject = (appearance as *mut objc2::runtime::AnyObject)
                .as_ref()
                .unwrap();
            let appearance_ref = obj.downcast_ref::<objc2_app_kit::NSAppearance>().unwrap();
            WindowAppearance::from_native(appearance_ref)
        }
    }

    fn display(&self) -> Option<Rc<dyn PlatformDisplay>> {
        unsafe {
            let screen = self.0.lock().native_window.screen();
            if screen.is_null() {
                return None;
            }
            let device_description: id = msg_send![screen, deviceDescription];
            let screen_number: id =
                NSDictionary::valueForKey_(device_description, ns_string("NSScreenNumber"));

            let screen_number: u32 = msg_send![screen_number, unsignedIntValue];

            Some(Rc::new(MacDisplay(screen_number)))
        }
    }

    fn mouse_position(&self) -> Point<Pixels> {
        let position = unsafe {
            self.0
                .lock()
                .native_window
                .mouseLocationOutsideOfEventStream()
        };
        convert_mouse_position(position, self.content_size().height)
    }

    fn modifiers(&self) -> Modifiers {
        unsafe {
            let modifiers: NSEventModifierFlags = msg_send![class!(NSEvent), modifierFlags];

            let control = modifiers.contains(NSEventModifierFlags::NSControlKeyMask);
            let alt = modifiers.contains(NSEventModifierFlags::NSAlternateKeyMask);
            let shift = modifiers.contains(NSEventModifierFlags::NSShiftKeyMask);
            let command = modifiers.contains(NSEventModifierFlags::NSCommandKeyMask);
            let function = modifiers.contains(NSEventModifierFlags::NSFunctionKeyMask);

            Modifiers {
                control,
                alt,
                shift,
                platform: command,
                function,
            }
        }
    }

    fn capslock(&self) -> Capslock {
        unsafe {
            let modifiers: NSEventModifierFlags = msg_send![class!(NSEvent), modifierFlags];

            Capslock {
                on: modifiers.contains(NSEventModifierFlags::NSAlphaShiftKeyMask),
            }
        }
    }

    fn set_input_handler(&mut self, input_handler: PlatformInputHandler) {
        self.0.as_ref().lock().input_handler = Some(input_handler);
    }

    fn take_input_handler(&mut self) -> Option<PlatformInputHandler> {
        self.0.as_ref().lock().input_handler.take()
    }

    fn prompt(
        &self,
        level: PromptLevel,
        msg: &str,
        detail: Option<&str>,
        answers: &[PromptButton],
    ) -> Option<oneshot::Receiver<usize>> {
        // macOs applies overrides to modal window buttons after they are added.
        // Two most important for this logic are:
        // * Buttons with "Cancel" title will be displayed as the last buttons in the modal
        // * Last button added to the modal via `addButtonWithTitle` stays focused
        // * Focused buttons react on "space"/" " keypresses
        // * Usage of `keyEquivalent`, `makeFirstResponder` or `setInitialFirstResponder` does not change the focus
        //
        // See also https://developer.apple.com/documentation/appkit/nsalert/1524532-addbuttonwithtitle#discussion
        // ```
        // By default, the first button has a key equivalent of Return,
        // any button with a title of “Cancel” has a key equivalent of Escape,
        // and any button with the title “Don’t Save” has a key equivalent of Command-D (but only if it’s not the first button).
        // ```
        //
        // To avoid situations when the last element added is "Cancel" and it gets the focus
        // (hence stealing both ESC and Space shortcuts), we find and add one non-Cancel button
        // last, so it gets focus and a Space shortcut.
        // This way, "Save this file? Yes/No/Cancel"-ish modals will get all three buttons mapped with a key.
        use objc2_foundation::{NSInteger, NSString};

        let initial_focus_ix = answers
            .iter()
            .enumerate()
            .rev()
            .find(|(_, label)| !label.is_cancel())
            .map(|(ix, _)| ix)
            .filter(|&ix| ix > 0);

        let marker = MainThreadMarker::new().expect("alert not on main thread");
        let alert = NSAlert::new(marker);
        alert.setAlertStyle(match level {
            PromptLevel::Critical => NSAlertStyle::Critical,
            PromptLevel::Warning => NSAlertStyle::Warning,
            PromptLevel::Info => NSAlertStyle::Informational,
        });
        let message = NSString::from_str(msg);
        alert.setMessageText(message.as_ref());

        if let Some(detail) = detail {
            let detail_text = NSString::from_str(detail);
            alert.setInformativeText(detail_text.as_ref());
        }

        let mut initial_focus_button: Option<Retained<Objc2NSButton>> = None;
        for (ix, answer) in answers.iter().enumerate() {
            let title = NSString::from_str(answer.label());
            let button = alert.addButtonWithTitle(&title);
            button.setTag(ix as NSInteger);

            if answer.is_cancel() {
                if let Some(key) = core::char::from_u32(ESCAPE_KEY) {
                    let key = NSString::from_str(&key.to_string());
                    button.setKeyEquivalent(&key);
                }
            } else if Some(ix) == initial_focus_ix {
                initial_focus_button = Some(button);
            }
        }

        if let Some(button) = initial_focus_button {
            alert.window().setInitialFirstResponder(Some(&button));
        }

        let (done_tx, done_rx) = oneshot::channel();
        let done_tx = Cell::new(Some(done_tx));

        let block = RcBlock::new(move |answer: NSInteger| {
            if let Some(done_tx) = done_tx.take() {
                let _ = done_tx.send(answer.try_into().unwrap());
            }
        });

        let lock = self.0.lock();
        let native_window = lock.native_window;
        let executor = lock.executor.clone();
        executor
            .spawn(async move {
                // SAFETY: `native_window` is an Objective-C `NSWindow` pointer
                // owned by the platform window; bridge it into objc2.
                let sheet_window: &Objc2NSWindow =
                    unsafe { &*(native_window as *const Objc2NSWindow) };
                alert.beginSheetModalForWindow_completionHandler(sheet_window, Some(&block));
            })
            .detach();
        Some(done_rx)
    }

    fn activate(&self) {
        let window = self.0.lock().native_window;
        let executor = self.0.lock().executor.clone();
        executor
            .spawn(async move {
                unsafe {
                    let _: () = msg_send![window, makeKeyAndOrderFront: nil];
                }
            })
            .detach();
    }

    fn is_active(&self) -> bool {
        unsafe { self.0.lock().native_window.isKeyWindow() == YES }
    }

    // is_hovered is unused on macOS. See Window::is_window_hovered.
    fn is_hovered(&self) -> bool {
        false
    }

    fn set_title(&mut self, title: &str) {
        unsafe {
            let app = NSApplication::sharedApplication(nil);
            let window = self.0.lock().native_window;
            let title = ns_string(title);
            let _: () = msg_send![app, changeWindowsItem:window title:title filename:false];
            let _: () = msg_send![window, setTitle: title];
            self.0.lock().move_traffic_light();
        }
    }

    fn get_title(&self) -> String {
        unsafe {
            let title: id = msg_send![self.0.lock().native_window, title];
            if title.is_null() {
                "".to_string()
            } else {
                title.to_str().to_string()
            }
        }
    }

    fn set_app_id(&mut self, _app_id: &str) {}

    fn set_background_appearance(&self, background_appearance: WindowBackgroundAppearance) {
        let mut this = self.0.as_ref().lock();
        this.background_appearance = background_appearance;

        let opaque = background_appearance == WindowBackgroundAppearance::Opaque;
        this.renderer.update_transparency(!opaque);

        unsafe {
            this.native_window.setOpaque_(opaque as BOOL);
            let background_color = if opaque {
                NSColor::colorWithSRGBRed_green_blue_alpha_(nil, 0f64, 0f64, 0f64, 1f64)
            } else {
                // Not using `+[NSColor clearColor]` to avoid broken shadow.
                NSColor::colorWithSRGBRed_green_blue_alpha_(nil, 0f64, 0f64, 0f64, 0.0001)
            };
            this.native_window.setBackgroundColor_(background_color);

            if NSAppKitVersionNumber < NSAppKitVersionNumber12_0 {
                // Whether `-[NSVisualEffectView respondsToSelector:@selector(_updateProxyLayer)]`.
                // On macOS Catalina/Big Sur `NSVisualEffectView` doesn’t own concrete sublayers
                // but uses a `CAProxyLayer`. Use the legacy WindowServer API.
                let blur_radius = if background_appearance == WindowBackgroundAppearance::Blurred {
                    80
                } else {
                    0
                };

                let window_number = this.native_window.windowNumber();
                CGSSetWindowBackgroundBlurRadius(CGSMainConnectionID(), window_number, blur_radius);
            } else {
                // On newer macOS `NSVisualEffectView` manages the effect layer directly. Using it
                // could have a better performance (it downsamples the backdrop) and more control
                // over the effect layer.
                if background_appearance != WindowBackgroundAppearance::Blurred {
                    if let Some(blur_view) = this.blurred_view {
                        NSView::removeFromSuperview(blur_view);
                        this.blurred_view = None;
                    }
                } else if this.blurred_view.is_none() {
                    let content_view = this.native_window.contentView();
                    let frame = NSView::bounds(content_view);
                    let mtm = MainThreadMarker::new().expect("Must run on the main thread");
                    let blur_view =
                        super::blurred_view::BlurredView::new(mtm, to_objc2_rect(frame));
                    blur_view.setAutoresizingMask(
                        objc2_app_kit::NSAutoresizingMaskOptions::ViewWidthSizable
                            | objc2_app_kit::NSAutoresizingMaskOptions::ViewHeightSizable,
                    );

                    let blur_view_ptr = Retained::into_raw(blur_view) as cocoa::base::id;

                    let _: () = msg_send![
                        content_view,
                        addSubview: blur_view_ptr
                        positioned: NSWindowOrderingMode::NSWindowBelow
                        relativeTo: nil
                    ];
                    this.blurred_view = Some(blur_view_ptr);
                }
            }
        }
    }

    fn background_appearance(&self) -> WindowBackgroundAppearance {
        self.0.as_ref().lock().background_appearance
    }

    fn is_subpixel_rendering_supported(&self) -> bool {
        // TODO: we could ask wgpu here but we need access to WgpuContext
        false
    }

    fn set_edited(&mut self, edited: bool) {
        unsafe {
            let window = self.0.lock().native_window;
            msg_send![window, setDocumentEdited: edited as BOOL]
        }

        // Changing the document edited state resets the traffic light position,
        // so we have to move it again.
        self.0.lock().move_traffic_light();
    }

    fn show_character_palette(&self) {
        let this = self.0.lock();
        let window = this.native_window;
        this.executor
            .spawn(async move {
                unsafe {
                    let app = NSApplication::sharedApplication(nil);
                    let _: () = msg_send![app, orderFrontCharacterPalette: window];
                }
            })
            .detach();
    }

    fn minimize(&self) {
        let window = self.0.lock().native_window;
        unsafe {
            window.miniaturize_(nil);
        }
    }

    fn zoom(&self) {
        let this = self.0.lock();
        let window = this.native_window;
        this.executor
            .spawn(async move {
                unsafe {
                    window.zoom_(nil);
                }
            })
            .detach();
    }

    fn toggle_fullscreen(&self) {
        let this = self.0.lock();
        let window = this.native_window;
        this.executor
            .spawn(async move {
                unsafe {
                    window.toggleFullScreen_(nil);
                }
            })
            .detach();
    }

    fn is_fullscreen(&self) -> bool {
        let this = self.0.lock();
        let window = this.native_window;

        unsafe {
            window
                .styleMask()
                .contains(NSWindowStyleMask::NSFullScreenWindowMask)
        }
    }

    fn on_request_frame(&self, callback: Box<dyn FnMut(RequestFrameOptions)>) {
        self.0.as_ref().lock().request_frame_callback = Some(callback);
    }

    fn on_input(&self, callback: Box<dyn FnMut(PlatformInput) -> crate::DispatchEventResult>) {
        self.0.as_ref().lock().event_callback = Some(callback);
    }

    fn on_active_status_change(&self, callback: Box<dyn FnMut(bool)>) {
        self.0.as_ref().lock().activate_callback = Some(callback);
    }

    fn on_hover_status_change(&self, _: Box<dyn FnMut(bool)>) {}

    fn on_resize(&self, callback: Box<dyn FnMut(Size<Pixels>, f32)>) {
        self.0.as_ref().lock().resize_callback = Some(callback);
    }

    fn on_moved(&self, callback: Box<dyn FnMut()>) {
        self.0.as_ref().lock().moved_callback = Some(callback);
    }

    fn on_should_close(&self, callback: Box<dyn FnMut() -> bool>) {
        self.0.as_ref().lock().should_close_callback = Some(callback);
    }

    fn on_close(&self, callback: Box<dyn FnOnce()>) {
        self.0.as_ref().lock().close_callback = Some(callback);
    }

    fn on_hit_test_window_control(&self, _callback: Box<dyn FnMut() -> Option<WindowControlArea>>) {
    }

    fn on_appearance_changed(&self, callback: Box<dyn FnMut()>) {
        self.0.lock().appearance_changed_callback = Some(callback);
    }

    fn tabbed_windows(&self) -> Option<Vec<SystemWindowTab>> {
        unsafe {
            let windows: id = msg_send![self.0.lock().native_window, tabbedWindows];
            if windows.is_null() {
                return None;
            }

            let count: NSUInteger = msg_send![windows, count];
            let mut result = Vec::new();
            for i in 0..count {
                let window: id = msg_send![windows, objectAtIndex:i];
                if msg_send![window, isKindOfClass: WINDOW_CLASS] {
                    let handle = get_window_state(&*window).lock().handle;
                    let title: id = msg_send![window, title];
                    let title = SharedString::from(title.to_str().to_string());

                    result.push(SystemWindowTab::new(title, handle));
                }
            }

            Some(result)
        }
    }

    fn tab_bar_visible(&self) -> bool {
        unsafe {
            let tab_group: id = msg_send![self.0.lock().native_window, tabGroup];
            if tab_group.is_null() {
                false
            } else {
                let tab_bar_visible: BOOL = msg_send![tab_group, isTabBarVisible];
                tab_bar_visible == YES
            }
        }
    }

    fn on_move_tab_to_new_window(&self, callback: Box<dyn FnMut()>) {
        self.0.as_ref().lock().move_tab_to_new_window_callback = Some(callback);
    }

    fn on_merge_all_windows(&self, callback: Box<dyn FnMut()>) {
        self.0.as_ref().lock().merge_all_windows_callback = Some(callback);
    }

    fn on_select_next_tab(&self, callback: Box<dyn FnMut()>) {
        self.0.as_ref().lock().select_next_tab_callback = Some(callback);
    }

    fn on_select_previous_tab(&self, callback: Box<dyn FnMut()>) {
        self.0.as_ref().lock().select_previous_tab_callback = Some(callback);
    }

    fn on_toggle_tab_bar(&self, callback: Box<dyn FnMut()>) {
        self.0.as_ref().lock().toggle_tab_bar_callback = Some(callback);
    }

    fn draw(&self, scene: &crate::Scene) {
        let mut this = self.0.lock();
        this.renderer.draw(scene);
    }

    fn sprite_atlas(&self) -> Arc<dyn PlatformAtlas> {
        self.0.lock().renderer.sprite_atlas().clone()
    }

    fn gpu_specs(&self) -> Option<crate::GpuSpecs> {
        self.0.lock().renderer.gpu_specs().into()
    }

    fn update_ime_position(&self, _bounds: Bounds<Pixels>) {
        let executor = self.0.lock().executor.clone();
        executor
            .spawn(async move {
                unsafe {
                    let input_context: id =
                        msg_send![class!(NSTextInputContext), currentInputContext];
                    if input_context.is_null() {
                        return;
                    }
                    let _: () = msg_send![input_context, invalidateCharacterCoordinates];
                }
            })
            .detach()
    }

    fn titlebar_double_click(&self) {
        let this = self.0.lock();
        let window = this.native_window;
        this.executor
            .spawn(async move {
                unsafe {
                    let defaults: id = NSUserDefaults::standardUserDefaults();
                    let domain = ns_string("NSGlobalDomain");
                    let key = ns_string("AppleActionOnDoubleClick");

                    let dict: id = msg_send![defaults, persistentDomainForName: domain];
                    let action: id = if !dict.is_null() {
                        msg_send![dict, objectForKey: key]
                    } else {
                        nil
                    };

                    let action_str = if !action.is_null() {
                        CStr::from_ptr(NSString::UTF8String(action)).to_string_lossy()
                    } else {
                        "".into()
                    };

                    match action_str.as_ref() {
                        "None" => {
                            // "Do Nothing" selected, so do no action
                        }
                        "Minimize" => {
                            window.miniaturize_(nil);
                        }
                        "Maximize" => {
                            window.zoom_(nil);
                        }
                        "Fill" => {
                            // There is no documented API for "Fill" action, so we'll just zoom the window
                            window.zoom_(nil);
                        }
                        _ => {
                            window.zoom_(nil);
                        }
                    }
                }
            })
            .detach();
    }

    fn start_window_move(&self) {
        let this = self.0.lock();
        let window = this.native_window;

        unsafe {
            let app = NSApplication::sharedApplication(nil);
            let mut event: id = msg_send![app, currentEvent];
            let _: () = msg_send![window, performWindowDragWithEvent: event];
        }
    }
}

impl rwh::HasWindowHandle for MacWindow {
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        // SAFETY: The AppKitWindowHandle is a wrapper around a pointer to an NSView
        unsafe {
            Ok(rwh::WindowHandle::borrow_raw(rwh::RawWindowHandle::AppKit(
                rwh::AppKitWindowHandle::new(self.0.lock().native_view.cast()),
            )))
        }
    }
}

impl rwh::HasDisplayHandle for MacWindow {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        // SAFETY: This is a no-op on macOS
        unsafe {
            Ok(rwh::DisplayHandle::borrow_raw(
                rwh::AppKitDisplayHandle::new().into(),
            ))
        }
    }
}

fn get_scale_factor(native_window: id) -> f32 {
    let factor = unsafe {
        let screen: id = msg_send![native_window, screen];
        if screen.is_null() {
            return 2.0;
        }
        NSScreen::backingScaleFactor(screen) as f32
    };

    // We are not certain what triggers this, but it seems that sometimes
    // this method would return 0 (https://github.com/zed-industries/zed/issues/6412)
    // It seems most likely that this would happen if the window has no screen
    // (if it is off-screen), though we'd expect to see viewDidChangeBackingProperties before
    // it was rendered for real.
    // Regardless, attempt to avoid the issue here.
    if factor == 0.0 { 2. } else { factor }
}

unsafe fn get_window_state(object: &Object) -> Arc<Mutex<MacWindowState>> {
    unsafe {
        let raw: *mut c_void = *object.get_ivar(WINDOW_STATE_IVAR);
        let rc1 = Arc::from_raw(raw as *mut Mutex<MacWindowState>);
        let rc2 = rc1.clone();
        mem::forget(rc1);
        rc2
    }
}

unsafe fn drop_window_state(object: &Object) {
    unsafe {
        let raw: *mut c_void = *object.get_ivar(WINDOW_STATE_IVAR);
        Arc::from_raw(raw as *mut Mutex<MacWindowState>);
    }
}

extern "C" fn yes(_: &Object, _: Sel) -> BOOL {
    YES
}

extern "C" fn dealloc_window(this: &Object, _: Sel) {
    unsafe {
        drop_window_state(this);
        let _: () = msg_send![super(this, class!(NSWindow)), dealloc];
    }
}

extern "C" fn window_did_change_occlusion_state(this: &Object, _: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    let lock = &mut *window_state.lock();
    unsafe {
        if lock
            .native_window
            .occlusionState()
            .contains(NSWindowOcclusionState::NSWindowOcclusionStateVisible)
        {
            lock.move_traffic_light();
            lock.start_display_link();
        } else {
            lock.stop_display_link();
        }
    }
}

extern "C" fn window_did_resize(this: &Object, _: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    window_state.as_ref().lock().move_traffic_light();
}

extern "C" fn window_will_enter_fullscreen(this: &Object, _: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    let mut lock = window_state.as_ref().lock();
    lock.fullscreen_restore_bounds = lock.bounds();

    let min_version = NSOperatingSystemVersion::new(15, 3, 0);

    if is_macos_version_at_least(min_version) {
        unsafe {
            lock.native_window.setTitlebarAppearsTransparent_(NO);
        }
    }
}

extern "C" fn window_will_exit_fullscreen(this: &Object, _: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    let mut lock = window_state.as_ref().lock();

    let min_version = NSOperatingSystemVersion::new(15, 3, 0);

    if is_macos_version_at_least(min_version) && lock.transparent_titlebar {
        unsafe {
            lock.native_window.setTitlebarAppearsTransparent_(YES);
        }
    }
}

pub(crate) fn is_macos_version_at_least(version: NSOperatingSystemVersion) -> bool {
    unsafe { NSProcessInfo::processInfo(nil).isOperatingSystemAtLeastVersion(version) }
}

extern "C" fn window_did_move(this: &Object, _: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    let mut lock = window_state.as_ref().lock();
    if let Some(mut callback) = lock.moved_callback.take() {
        drop(lock);
        callback();
        window_state.lock().moved_callback = Some(callback);
    }
}

// Update the window scale factor and drawable size, and call the resize callback if any.
pub(crate) fn update_window_scale_factor(window_state: &Arc<Mutex<MacWindowState>>) {
    let mut lock = window_state.as_ref().lock();
    let scale_factor = lock.scale_factor();
    let size = lock.content_size();
    let drawable_size = size.to_device_pixels(scale_factor);
    // unsafe {
    //     let _: () = msg_send![
    //         lock.renderer.layer(),
    //         setContentsScale: scale_factor as f64
    //     ];
    // }
    unsafe {
        let layer: id = msg_send![lock.native_view.as_ptr(), layer];
        if !layer.is_null() {
            let _: () = msg_send![
                layer,
                setContentsScale: scale_factor as f64
            ];
        }
    }

    lock.renderer.update_drawable_size(drawable_size);

    if let Some(mut callback) = lock.resize_callback.take() {
        let content_size = lock.content_size();
        let scale_factor = lock.scale_factor();
        drop(lock);
        callback(content_size, scale_factor);
        window_state.as_ref().lock().resize_callback = Some(callback);
    };
}

extern "C" fn window_did_change_screen(this: &Object, _: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    let mut lock = window_state.as_ref().lock();
    lock.start_display_link();
    drop(lock);
    update_window_scale_factor(&window_state);
}

extern "C" fn window_did_change_key_status(this: &Object, selector: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    let mut lock = window_state.lock();
    let is_active = unsafe { lock.native_window.isKeyWindow() == YES };

    // When opening a pop-up while the application isn't active, Cocoa sends a spurious
    // `windowDidBecomeKey` message to the previous key window even though that window
    // isn't actually key. This causes a bug if the application is later activated while
    // the pop-up is still open, making it impossible to activate the previous key window
    // even if the pop-up gets closed. The only way to activate it again is to de-activate
    // the app and re-activate it, which is a pretty bad UX.
    // The following code detects the spurious event and invokes `resignKeyWindow`:
    // in theory, we're not supposed to invoke this method manually but it balances out
    // the spurious `becomeKeyWindow` event and helps us work around that bug.
    if selector == sel!(windowDidBecomeKey:) && !is_active {
        unsafe {
            let _: () = msg_send![lock.native_window, resignKeyWindow];
            return;
        }
    }

    let executor = lock.executor.clone();
    drop(lock);

    // When a window becomes active, trigger an immediate synchronous frame request to prevent
    // tab flicker when switching between windows in native tabs mode.
    //
    // This is only done on subsequent activations (not the first) to ensure the initial focus
    // path is properly established. Without this guard, the focus state would remain unset until
    // the first mouse click, causing keybindings to be non-functional.
    if selector == sel!(windowDidBecomeKey:) && is_active {
        let window_state = unsafe { get_window_state(this) };
        let mut lock = window_state.lock();

        if lock.activated_least_once {
            if let Some(mut callback) = lock.request_frame_callback.take() {
                // lock.renderer.set_presents_with_transaction(true);
                lock.stop_display_link();
                drop(lock);
                callback(Default::default());

                let mut lock = window_state.lock();
                lock.request_frame_callback = Some(callback);
                // lock.renderer.set_presents_with_transaction(false);
                lock.start_display_link();
            }
        } else {
            lock.activated_least_once = true;
        }
    }

    executor
        .spawn(async move {
            let mut lock = window_state.as_ref().lock();
            if is_active {
                lock.move_traffic_light();
            }

            if let Some(mut callback) = lock.activate_callback.take() {
                drop(lock);
                callback(is_active);
                window_state.lock().activate_callback = Some(callback);
            };
        })
        .detach();
}

extern "C" fn window_should_close(this: &Object, _: Sel, _: id) -> BOOL {
    let window_state = unsafe { get_window_state(this) };
    let mut lock = window_state.as_ref().lock();
    if let Some(mut callback) = lock.should_close_callback.take() {
        drop(lock);
        let should_close = callback();
        window_state.lock().should_close_callback = Some(callback);
        should_close as BOOL
    } else {
        YES
    }
}

extern "C" fn close_window(this: &Object, _: Sel) {
    unsafe {
        let close_callback = {
            let window_state = get_window_state(this);
            let mut lock = window_state.as_ref().lock();
            lock.close_callback.take()
        };

        if let Some(callback) = close_callback {
            callback();
        }

        let _: () = msg_send![super(this, class!(NSWindow)), close];
    }
}

unsafe extern "C" fn step(view: *mut c_void) {
    let view: &super::gpui_view::GPUIView = unsafe { &*(view as *mut super::gpui_view::GPUIView) };
    let window_state = view.window_state();
    let mut lock = window_state.lock();

    if let Some(mut callback) = lock.request_frame_callback.take() {
        drop(lock);
        callback(Default::default());
        window_state.lock().request_frame_callback = Some(callback);
    }
}

extern "C" fn dragging_entered(this: &Object, _: Sel, dragging_info: id) -> NSDragOperation {
    let window_state = unsafe { get_window_state(this) };
    let position = drag_event_position(&window_state, dragging_info);
    let paths = external_paths_from_event(dragging_info);
    if let Some(event) =
        paths.map(|paths| PlatformInput::FileDrop(FileDropEvent::Entered { position, paths }))
        && send_new_event(&window_state, event)
    {
        window_state.lock().external_files_dragged = true;
        return NSDragOperationCopy;
    }
    NSDragOperationNone
}

extern "C" fn dragging_updated(this: &Object, _: Sel, dragging_info: id) -> NSDragOperation {
    let window_state = unsafe { get_window_state(this) };
    let position = drag_event_position(&window_state, dragging_info);
    if send_new_event(
        &window_state,
        PlatformInput::FileDrop(FileDropEvent::Pending { position }),
    ) {
        NSDragOperationCopy
    } else {
        NSDragOperationNone
    }
}

extern "C" fn dragging_exited(this: &Object, _: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    send_new_event(
        &window_state,
        PlatformInput::FileDrop(FileDropEvent::Exited),
    );
    window_state.lock().external_files_dragged = false;
}

extern "C" fn perform_drag_operation(this: &Object, _: Sel, dragging_info: id) -> BOOL {
    let window_state = unsafe { get_window_state(this) };
    let position = drag_event_position(&window_state, dragging_info);
    send_new_event(
        &window_state,
        PlatformInput::FileDrop(FileDropEvent::Submit { position }),
    )
    .to_objc()
}

fn external_paths_from_event(dragging_info: *mut Object) -> Option<ExternalPaths> {
    let mut paths = SmallVec::new();
    let pasteboard: id = unsafe { msg_send![dragging_info, draggingPasteboard] };
    let filenames = unsafe { NSPasteboard::propertyListForType(pasteboard, NSFilenamesPboardType) };
    if filenames == nil {
        return None;
    }
    for file in unsafe { filenames.iter() } {
        let path = unsafe {
            let f = NSString::UTF8String(file);
            CStr::from_ptr(f).to_string_lossy().into_owned()
        };
        paths.push(PathBuf::from(path))
    }
    Some(ExternalPaths(paths))
}

extern "C" fn conclude_drag_operation(this: &Object, _: Sel, _: id) {
    let window_state = unsafe { get_window_state(this) };
    send_new_event(
        &window_state,
        PlatformInput::FileDrop(FileDropEvent::Exited),
    );
}

pub(crate) async fn synthetic_drag(
    window_state: Weak<Mutex<MacWindowState>>,
    drag_id: usize,
    event: MouseMoveEvent,
) {
    loop {
        Timer::after(Duration::from_millis(16)).await;
        if let Some(window_state) = window_state.upgrade() {
            let mut lock = window_state.lock();
            if lock.synthetic_drag_counter == drag_id {
                if let Some(mut callback) = lock.event_callback.take() {
                    drop(lock);
                    callback(PlatformInput::MouseMove(event.clone()));
                    window_state.lock().event_callback = Some(callback);
                }
            } else {
                break;
            }
        }
    }
}

fn send_new_event(window_state_lock: &Mutex<MacWindowState>, e: PlatformInput) -> bool {
    let window_state = window_state_lock.lock().event_callback.take();
    if let Some(mut callback) = window_state {
        callback(e);
        window_state_lock.lock().event_callback = Some(callback);
        true
    } else {
        false
    }
}

fn drag_event_position(window_state: &Mutex<MacWindowState>, dragging_info: id) -> Point<Pixels> {
    let drag_location: NSPoint = unsafe { msg_send![dragging_info, draggingLocation] };
    convert_mouse_position(drag_location, window_state.lock().content_size().height)
}

unsafe fn display_id_for_screen(screen: id) -> CGDirectDisplayID {
    unsafe {
        let device_description = NSScreen::deviceDescription(screen);
        let screen_number_key: id = ns_string("NSScreenNumber");
        let screen_number = device_description.objectForKey_(screen_number_key);
        let screen_number: NSUInteger = msg_send![screen_number, unsignedIntegerValue];
        screen_number as CGDirectDisplayID
    }
}

pub(crate) unsafe fn remove_layer_background(layer: id) {
    unsafe {
        let _: () = msg_send![layer, setBackgroundColor:nil];

        let class_name: id = msg_send![layer, className];
        if class_name.isEqualToString("CAChameleonLayer") {
            // Remove the desktop tinting effect.
            let _: () = msg_send![layer, setHidden: YES];
            return;
        }

        let filters: id = msg_send![layer, filters];
        if !filters.is_null() {
            // Remove the increased saturation.
            // The effect of a `CAFilter` or `CIFilter` is determined by its name, and the
            // `description` reflects its name and some parameters. Currently `NSVisualEffectView`
            // uses a `CAFilter` named "colorSaturate". If one day they switch to `CIFilter`, the
            // `description` will still contain "Saturat" ("... inputSaturation = ...").
            let test_string: id = ns_string("Saturat");
            let count = NSArray::count(filters);
            for i in 0..count {
                let description: id = msg_send![filters.objectAtIndex(i), description];
                let hit: BOOL = msg_send![description, containsString: test_string];
                if hit == NO {
                    continue;
                }

                let all_indices = NSRange {
                    location: 0,
                    length: count,
                };
                let indices: id = msg_send![class!(NSMutableIndexSet), indexSet];
                let _: () = msg_send![indices, addIndexesInRange: all_indices];
                let _: () = msg_send![indices, removeIndex:i];
                let filtered: id = msg_send![filters, objectsAtIndexes: indices];
                let _: () = msg_send![layer, setFilters: filtered];
                break;
            }
        }

        let sublayers: id = msg_send![layer, sublayers];
        if !sublayers.is_null() {
            let count = NSArray::count(sublayers);
            for i in 0..count {
                let sublayer = sublayers.objectAtIndex(i);
                remove_layer_background(sublayer);
            }
        }
    }
}

extern "C" fn add_titlebar_accessory_view_controller(this: &Object, _: Sel, view_controller: id) {
    unsafe {
        let _: () = msg_send![super(this, class!(NSWindow)), addTitlebarAccessoryViewController: view_controller];

        // Hide the native tab bar and set its height to 0, since we render our own.
        let accessory_view: id = msg_send![view_controller, view];
        let _: () = msg_send![accessory_view, setHidden: YES];
        let mut frame: NSRect = msg_send![accessory_view, frame];
        frame.size.height = 0.0;
        let _: () = msg_send![accessory_view, setFrame: frame];
    }
}

extern "C" fn move_tab_to_new_window(this: &Object, _: Sel, _: id) {
    unsafe {
        let _: () = msg_send![super(this, class!(NSWindow)), moveTabToNewWindow:nil];

        let window_state = get_window_state(this);
        let mut lock = window_state.as_ref().lock();
        if let Some(mut callback) = lock.move_tab_to_new_window_callback.take() {
            drop(lock);
            callback();
            window_state.lock().move_tab_to_new_window_callback = Some(callback);
        }
    }
}

extern "C" fn merge_all_windows(this: &Object, _: Sel, _: id) {
    unsafe {
        let _: () = msg_send![super(this, class!(NSWindow)), mergeAllWindows:nil];

        let window_state = get_window_state(this);
        let mut lock = window_state.as_ref().lock();
        if let Some(mut callback) = lock.merge_all_windows_callback.take() {
            drop(lock);
            callback();
            window_state.lock().merge_all_windows_callback = Some(callback);
        }
    }
}

extern "C" fn select_next_tab(this: &Object, _sel: Sel, _id: id) {
    let window_state = unsafe { get_window_state(this) };
    let mut lock = window_state.as_ref().lock();
    if let Some(mut callback) = lock.select_next_tab_callback.take() {
        drop(lock);
        callback();
        window_state.lock().select_next_tab_callback = Some(callback);
    }
}

extern "C" fn select_previous_tab(this: &Object, _sel: Sel, _id: id) {
    let window_state = unsafe { get_window_state(this) };
    let mut lock = window_state.as_ref().lock();
    if let Some(mut callback) = lock.select_previous_tab_callback.take() {
        drop(lock);
        callback();
        window_state.lock().select_previous_tab_callback = Some(callback);
    }
}

extern "C" fn toggle_tab_bar(this: &Object, _sel: Sel, _id: id) {
    unsafe {
        let _: () = msg_send![super(this, class!(NSWindow)), toggleTabBar:nil];

        let window_state = get_window_state(this);
        let mut lock = window_state.as_ref().lock();
        lock.move_traffic_light();

        if let Some(mut callback) = lock.toggle_tab_bar_callback.take() {
            drop(lock);
            callback();
            window_state.lock().toggle_tab_bar_callback = Some(callback);
        }
    }
}

fn to_objc2_rect(rect: NSRect) -> objc2_foundation::NSRect {
    objc2_foundation::NSRect::new(
        objc2_foundation::NSPoint {
            x: rect.origin.x,
            y: rect.origin.y,
        },
        objc2_foundation::NSSize {
            width: rect.size.width,
            height: rect.size.height,
        },
    )
}
