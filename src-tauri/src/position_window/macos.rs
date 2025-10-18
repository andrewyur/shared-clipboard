use anyhow::anyhow;
use objc2_application_services::{
    kAXTrustedCheckOptionPrompt, AXError, AXIsProcessTrustedWithOptions, AXUIElement, AXValue,
    AXValueType,
};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFIndex, CFRange, CFType, CGRect};
use objc2_foundation::NSString;
use std::{ffi::c_void, ptr::NonNull};
use tauri::{LogicalPosition, LogicalRect, LogicalSize, WebviewWindow};

// translated from https://github.com/p0deje/Maccy/blob/3358537421cdb29613c19c6fc6f2b0c17fc412f0/Maccy/Maccy.swift
// this usually only works with apps developed with apple using native text inputs... but its better than nothing i guess
pub fn get_caret(_window: &WebviewWindow) -> anyhow::Result<LogicalRect<i32, u32>> {
    let options = CFDictionary::from_slices(
        &[unsafe { kAXTrustedCheckOptionPrompt }],
        &[CFBoolean::new(true)],
    );

    if !unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) } {
        return Err(anyhow!("Process is not trusted"));
    }

    let systemwide_element = unsafe { AXUIElement::new_system_wide() };

    let mut focused_element: *const CFType = std::ptr::null();

    let ax_error = unsafe {
        AXUIElement::copy_attribute_value(
            &systemwide_element,
            NSString::from_str("AXFocusedUIElement").as_ref(),
            NonNull::new(&mut focused_element).unwrap(),
        )
    };

    if ax_error != AXError::Success {
        if ax_error == AXError::NoValue {
            return Err(anyhow!("No currently focused UI Element"));
        } else {
            return Err(anyhow!(
                "Could not fetch currently focused UI Element: {:?}",
                ax_error
            ));
        }
    }

    if focused_element.is_null() {
        return Err(anyhow!("focused_element is null"));
    }

    let mut selected_range_value: *const CFType = std::ptr::null();

    let ax_error = unsafe {
        AXUIElement::copy_attribute_value(
            (focused_element as *const AXUIElement).as_ref().unwrap(),
            NSString::from_str("AXSelectedTextRange").as_ref(),
            NonNull::new(&mut selected_range_value as *mut *const CFType).unwrap(),
        )
    };

    if ax_error != AXError::Success {
        if ax_error == AXError::NoValue {
            return Err(anyhow!("No currently selected text range"));
        } else {
            return Err(anyhow!(
                "Could not fetch currently selected text range: {:?}",
                ax_error
            ));
        }
    }

    if selected_range_value.is_null() {
        return Err(anyhow!("selected_range_value is null"));
    }

    let mut selected_range = CFRange::new(CFIndex::default(), CFIndex::default());

    if !unsafe {
        AXValue::value(
            (selected_range_value as *const AXValue).as_ref().unwrap(),
            AXValueType::CFRange,
            NonNull::new(&mut selected_range as *mut _ as *mut c_void).unwrap(),
        )
    } {
        log::warn!("Getting selected range value returned false");
    }

    let mut select_bounds = std::ptr::null();
    let ax_error = unsafe {
        AXUIElement::copy_parameterized_attribute_value(
            (focused_element as *const AXUIElement).as_ref().unwrap(),
            NSString::from_str("AXBoundsForRange").as_ref(),
            selected_range_value.as_ref().unwrap(),
            NonNull::new(&mut select_bounds).unwrap(),
        )
    };

    if ax_error != AXError::Success {
        return Err(anyhow!(
            "Could not fetch screen bounds for text range: {:?}",
            ax_error
        ));
    }

    let mut select_rect = CGRect::default();

    if !unsafe {
        AXValue::value(
            (select_bounds as *const AXValue).as_ref().unwrap(),
            AXValueType::CGRect,
            NonNull::new(&mut select_rect as *mut _ as *mut c_void).unwrap(),
        )
    } {
        log::warn!("Getting select bounds rect value returned false");
    }

    return Ok(LogicalRect {
        position: LogicalPosition {
            x: select_rect.origin.x as i32,
            y: (select_rect.origin.y + select_rect.size.height) as i32,
        },
        size: LogicalSize {
            width: select_rect.size.width as u32,
            height: select_rect.size.height as u32,
        },
    });
}

// if you ever want to fall back to fixed position within display of focused element, this is how you do it

// let mut caret_display_id = CGDirectDisplayID::default();
// let mut matched_displays = 0;

// let cg_error = unsafe { CGGetDisplaysWithPoint(select_rect.origin, 1, &mut caret_display_id, &mut matched_displays) };
// if cg_error != CGError::Success {
//     return Err(anyhow!( "Could get display id for caret position: {:?}", cg_error ));
// }
