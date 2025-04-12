use core_foundation::base::{CFAllocatorRef, CFIndex, CFRelease};
use core_foundation::runloop::CFRunLoopSourceRef;
use core_foundation::string::CFStringRef;
use std::os::raw::{c_longlong, c_void};
use std::ptr;
use std::collections::HashMap;

// Import necessary items from other modules
use crate::accessibility::*;
use crate::cf_utils::{cf_string_ref, cfstring_to_string};
use crate::utils::get_app_name_from_pid;
use crate::network::check_nettop_for_pid;

// Re-export AXUIElementRef for use within this module if needed
pub use crate::accessibility::AXUIElementRef;

// Type Aliases & Structs for C Types
pub type CGEventTapProxy = *mut c_void; // Opaque pointer
pub type CGEventType = u32;
pub type CGEventRef = *mut c_void; // Opaque pointer
pub type CFMachPortRef = *mut c_void; // Opaque pointer (actually __CFMachPort*)
pub type CGEventTapLocation = u32;
pub type CGEventTapPlacement = u32;
pub type CGEventTapOptions = u32;
pub type CGEventMask = u64;
pub type CGEventField = u32;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CGPoint {
    pub x: f64,
    pub y: f64,
}

// Type for the event tap callback
pub type CGEventTapCallBack = unsafe extern "C" fn(
    proxy: CGEventTapProxy,
    type_: CGEventType,
    event: CGEventRef,
    userInfo: *mut c_void,
) -> CGEventRef;

// Constants
// CGEventTapLocation
pub const K_CG_HID_EVENT_TAP: CGEventTapLocation = 0;
// CGEventTapPlacement
pub const K_CG_HEAD_INSERT_EVENT_TAP: CGEventTapPlacement = 0;
// CGEventTapOptions
pub const K_CG_EVENT_TAP_DEFAULT: CGEventTapOptions = 0x00000000;
// CGEventType
#[allow(dead_code)]
pub const K_CG_EVENT_NULL: CGEventType = 0; // Internal use
pub const K_CG_EVENT_LEFT_MOUSE_DOWN: CGEventType = 1;
pub const K_CG_EVENT_KEY_DOWN: CGEventType = 10;
pub const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: CGEventType = 0xFFFFFFFE;
pub const K_CG_EVENT_TAP_DISABLED_BY_USER_INPUT: CGEventType = 0xFFFFFFFF;

// CGEventField
pub const K_CG_EVENT_TARGET_UNIX_PROCESS_ID: CGEventField = 8; // kCGEventTargetUnixProcessID
pub const K_CG_KEYBOARD_EVENT_KEYCODE: CGEventField = 9; // kCGKeyboardEventKeycode

#[link(name = "CoreGraphics", kind = "framework")]
#[allow(non_snake_case)] // To allow function names like CGEventTapCreate
unsafe extern "C" {
    pub fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        eventsOfInterest: CGEventMask,
        callback: CGEventTapCallBack,
        userInfo: *mut c_void,
    ) -> CFMachPortRef;

    pub fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);

    pub fn CGEventGetLocation(event: CGEventRef) -> CGPoint;
    pub fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> c_longlong; // Note: Returns int64_t

    pub fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef, // Usually kCFAllocatorDefault or null
        tap: CFMachPortRef,
        order: CFIndex, // Usually 0
    ) -> CFRunLoopSourceRef;
}

// Global mutable cache for network stats (requires unsafe access)
static mut EVENT_CALLBACK_CACHE: Option<HashMap<String, (u64, u64)>> = None;

// The actual event callback function
pub unsafe extern "C" fn event_callback(
    _proxy: CGEventTapProxy,
    type_: CGEventType,
    event: CGEventRef,
    user_info: *mut c_void,
) -> CGEventRef {
    
    // Get userInfo (system_wide element)
    if user_info.is_null() {
        log::error!("userInfo (system_wide element) is null in callback!");
        return event; // Cannot proceed
    }
    let system_wide_element = user_info as AXUIElementRef;

    // Handle tap disable events
    if type_ == K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT || type_ == K_CG_EVENT_TAP_DISABLED_BY_USER_INPUT {
         println!("DEBUG: Event tap disabled (type: {})", type_);
         log::warn!("Event Tap disabled (type: {})! Input monitoring stopped.", type_);
         // We might need to re-enable the tap if desired.
         // unsafe { CGEventTapEnable(proxy as CFMachPortRef, true) }; // Needs unsafe block if uncommented
         return event; // Return the event directly
    }

    // We are interested in left mouse down OR key down events
    if type_ != K_CG_EVENT_LEFT_MOUSE_DOWN && type_ != K_CG_EVENT_KEY_DOWN {
        return event;
    }
    
    // Get PID common to both event types we handle   
    let pid = unsafe { CGEventGetIntegerValueField(event, K_CG_EVENT_TARGET_UNIX_PROCESS_ID) } as i32;
    let app_name = get_app_name_from_pid(pid); // Use imported function

    // --- Handle Left Mouse Down --- 
    if type_ == K_CG_EVENT_LEFT_MOUSE_DOWN {
         let location = unsafe { CGEventGetLocation(event) };
         log::debug!(
             "LeftMouseDown detected. App='{}' (PID={}), Pos=({:.1}, {:.1}), SysWideRef={:p}",
             app_name, pid, location.x, location.y, system_wide_element
         );
          
         // Get the AXUIElementRef for the element at the click location
         let mut element_ref: AXUIElementRef = ptr::null_mut();
         let result = unsafe { ax_ui_element_copy_element_at_position(
             system_wide_element, // Use the system-wide ref here!
             location.x as f32, 
             location.y as f32, 
             &mut element_ref
         )};
         
         if result != 0 || element_ref.is_null() { // kAXErrorSuccess is 0
             log::debug!("Could not get element at position ({:.1}, {:.1}) for PID {}. AXError={}", location.x, location.y, pid, result);
             return event; // Can't proceed
         }
         log::debug!("Got element_ref ({:p}) at position ({:.1}, {:.1})", element_ref, location.x, location.y);

         // --- Get Actual PID from Element ---
         let mut actual_pid: i32 = -1; // Default to -1 if error
         let pid_result = unsafe { ax_ui_element_get_pid(element_ref, &mut actual_pid) };
         if pid_result != 0 { // kAXErrorSuccess is 0
             log::debug!("Failed to get PID from element_ref ({:p}). AXError={}", element_ref, pid_result);
             actual_pid = pid; // Fallback to event PID if needed, though likely still 0
         } else {
              log::debug!("Got actual PID {} from element_ref ({:p})", actual_pid, element_ref);
         }
         let actual_app_name = get_app_name_from_pid(actual_pid); // Use imported function
         // --- End Get Actual PID ---

         // --- Call nettop check ---
         let pid_str = actual_pid.to_string();
         // Access and initialize the cache if needed (unsafe block required)
         unsafe {
             if EVENT_CALLBACK_CACHE.is_none() {
                 EVENT_CALLBACK_CACHE = Some(HashMap::new());
             }
             if let Some(cache) = &mut EVENT_CALLBACK_CACHE {
                  log::debug!("Calling check_nettop_for_pid for PID {} (App: {})", actual_pid, actual_app_name);
                  check_nettop_for_pid(&pid_str, cache);
             }
         }
         // --- End nettop check ---

         // --- Get Identifier (Try this for all elements) ---
         let mut identifier_ref: *mut c_void = ptr::null_mut();
         let identifier_attr = unsafe { cf_string_ref(K_AX_IDENTIFIER_ATTRIBUTE) }; // Use imported cf_utils
         let identifier_result = unsafe { ax_ui_element_copy_attribute_value(
             element_ref, 
             identifier_attr, 
             &mut identifier_ref
         )};

         let identifier_str = if identifier_result == 0 && !identifier_ref.is_null() {
             unsafe { cfstring_to_string(identifier_ref as CFStringRef) } // Use imported cf_utils
         } else {
             None
         };
         log::debug!("Element identifier is \"{:?}\" (AXError={})", identifier_str, identifier_result);

         // Clean up identifier attribute string and value ref
         unsafe { CFRelease(identifier_attr as *const c_void); }
         if !identifier_ref.is_null() { unsafe { CFRelease(identifier_ref); } }
         // --- End Get Identifier ---

         // Get the role of the element
         let mut role_ref: *mut c_void = ptr::null_mut();
         let role_attr = unsafe { cf_string_ref(K_AX_ROLE_ATTRIBUTE) }; // Use imported cf_utils
         let role_result = unsafe { ax_ui_element_copy_attribute_value(
             element_ref, 
             role_attr, 
             &mut role_ref
         )};
         
         let role_str = if role_result == 0 && !role_ref.is_null() {
             let converted_role = unsafe { cfstring_to_string(role_ref as CFStringRef) }; // Use imported cf_utils
             log::debug!("Element role is \"{:?}\" (AXError={})", converted_role, role_result);
             converted_role
         } else {
             log::debug!("Failed to get element role (AXError={})", role_result);
             None
         };

         // Clean up role attribute string and potentially role value ref
         unsafe { CFRelease(role_attr as *const c_void); }
         if !role_ref.is_null() { unsafe { CFRelease(role_ref); } }

         // Check if it's a button
         if let Some(role) = role_str {
             if role == K_AX_BUTTON_ROLE { // Use imported constant
                 // --- Get Description Directly ---
                 log::debug!("Attempting to get description for button element {:p}", element_ref);
                 let mut desc_ref: *mut c_void = ptr::null_mut();
                 let desc_attr = unsafe { cf_string_ref(K_AX_DESCRIPTION_ATTRIBUTE) }; // Use imported cf_utils
                 let desc_result = unsafe { ax_ui_element_copy_attribute_value(
                     element_ref, 
                     desc_attr, 
                     &mut desc_ref
                 )};
                 let description_str = if desc_result == 0 && !desc_ref.is_null() {
                     let converted_desc = unsafe { cfstring_to_string(desc_ref as CFStringRef) }; // Use imported cf_utils
                     log::debug!("Got element description \"{:?}\" (AXError={})", converted_desc, desc_result);
                     converted_desc
                 } else {
                     log::debug!("Failed to get element description (AXError={})", desc_result);
                     None
                 };
                 // Clean up description attribute and value refs
                 unsafe { CFRelease(desc_attr as *const c_void); }
                 if !desc_ref.is_null() { unsafe { CFRelease(desc_ref); } }
                 // --- End Get Description ---

                 // Get the optional identifier string we fetched earlier
                 let id_str = identifier_str.clone().unwrap_or_else(|| "<No ID>".to_string());
                  
                 // Log the button click
                 let button_label = description_str.unwrap_or_else(|| "<No Label>".to_string());
                 // Use actual_app_name and actual_pid
                 log::info!(
                     "Button Clicked: App='{}' (PID={}), ID='{}', Label='{}', Pos=({:.1}, {:.1})",
                     actual_app_name, 
                     actual_pid, 
                     id_str,
                     button_label, // Use the final label (might be from Title or Description)
                     location.x,
                     location.y
                 );
             } else {
                 // Log if it's not a button but has an identifier
                 if let Some(id) = identifier_str.clone() {
                     let role_name = role.clone(); // Already unwrapped Some(role)
                     log::info!(
                         "Element Clicked: App='{}' (PID={}), ID='{}', Role='{}', Pos=({:.1}, {:.1})",
                         actual_app_name,
                         actual_pid,
                         id,
                         role_name,
                         location.x,
                         location.y
                     );
                 }
             }
         }

         // Clean up element 
         unsafe { CFRelease(element_ref as *const c_void); }

    // --- Handle Key Down ---    
    } else if type_ == K_CG_EVENT_KEY_DOWN {
        let keycode = unsafe { CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) };
        log::info!(
            "Key Down: App='{}' (PID={}), KeyCode={}",
            app_name, 
            pid, 
            keycode
        );
    }
    
    event // Pass the event along
} 