// Copyright 2024 Serial CLI Contributors
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use tauri::AppHandle;

pub fn create_system_tray(_app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    // System tray is now configured in tauri.conf.json
    // We just need to set up the event handler here

    // The tray menu items will be handled by Tauri's built-in system
    Ok(())
}
