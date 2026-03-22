#![allow(unexpected_cfgs, deprecated)]

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DisplayInfo {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
    pub is_mirror: bool,
    pub is_virtual: bool,
    pub model_number: u32,
}

#[cfg(target_os = "macos")]
pub fn enumerate_displays() -> Vec<DisplayInfo> {
    use core_graphics::display::{
        CGDisplay, CGGetActiveDisplayList,
    };

    let mut display_ids = [0u32; 16];
    let mut display_count: u32 = 0;

    unsafe {
        CGGetActiveDisplayList(16, display_ids.as_mut_ptr(), &mut display_count);
    }

    let main_display_id = CGDisplay::main().id;

    (0..display_count as usize)
        .map(|i| {
            let id = display_ids[i];
            let display = CGDisplay::new(id);
            let bounds = display.bounds();

            DisplayInfo {
                id,
                width: bounds.size.width as u32,
                height: bounds.size.height as u32,
                is_primary: id == main_display_id,
                is_mirror: display.is_in_mirror_set(),
                is_virtual: display.model_number() == 0,
                model_number: display.model_number(),
            }
        })
        .collect()
}

#[cfg(target_os = "macos")]
pub fn get_secondary_display_origin() -> Option<(f64, f64)> {
    let displays = enumerate_displays();
    displays
        .iter()
        .find(|d| !d.is_primary)
        .map(|_| {
            use core_graphics::display::CGDisplay;
            // Find non-primary display bounds
            let mut display_ids = [0u32; 16];
            let mut count: u32 = 0;
            unsafe {
                core_graphics::display::CGGetActiveDisplayList(16, display_ids.as_mut_ptr(), &mut count);
            }
            let main_id = CGDisplay::main().id;
            for i in 0..count as usize {
                if display_ids[i] != main_id {
                    let display = CGDisplay::new(display_ids[i]);
                    let bounds = display.bounds();
                    return (bounds.origin.x, bounds.origin.y);
                }
            }
            (0.0, 0.0)
        })
}

#[cfg(not(target_os = "macos"))]
pub fn enumerate_displays() -> Vec<DisplayInfo> {
    Vec::new()
}

#[cfg(not(target_os = "macos"))]
pub fn get_secondary_display_origin() -> Option<(f64, f64)> {
    None
}
