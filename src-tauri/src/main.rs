// Prevent console window on Windows (including startup from debug executable).
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

fn main() {
    snappaste_lib::run()
}
