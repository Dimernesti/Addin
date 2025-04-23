#![warn(clippy::pedantic)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::must_use_candidate,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::return_self_not_must_use
)]

mod addin;
mod git;

use std::{
    ffi::{c_int, c_long, c_void},
    sync::atomic::{AtomicI32, Ordering},
};

use addin::GitAddin;
use addin1c::{AttachType, create_component, destroy_component, name};
use log::{debug, info, LevelFilter};

pub static PLATFORM_CAPABILITIES: AtomicI32 = AtomicI32::new(-1);

/// # Safety
///
/// Component must be non-null.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetClassObject(name: *const u16, component: *mut *mut c_void) -> c_long {
    match unsafe { *name } as u8 {
        b'1' => {
            let _res = simple_logging::log_to_file(
                "D:\\users\\sdp\\Documents\\log\\git-addin.log",
                LevelFilter::Debug,
            );
            info!("creating addin");
            let addin = GitAddin::new();
            unsafe { create_component(component, addin) }
        },
        _ => 0,
    }
}

/// # Safety
///
/// Component must be returned from `GetClassObject`, the function must be called once for each component.
#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DestroyObject(component: *mut *mut c_void) -> c_long {
    unsafe {
        info!("destroing addin");
        destroy_component(component)
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn GetClassNames() -> *const u16 {
    // small strings for performance
    name!("1").as_ptr()
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn SetPlatformCapabilities(capabilities: c_int) -> c_int {
    debug!("platform capabilities set");
    PLATFORM_CAPABILITIES.store(capabilities, Ordering::Relaxed);
    3
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
pub extern "C" fn GetAttachType() -> AttachType {
    AttachType::Any
}
