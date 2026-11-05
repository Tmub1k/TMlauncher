// Prevents an additional console window on Windows in both debug and release builds.
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![windows_subsystem = "windows"]

fn main() {
    omnilauncher_lib::run();
}
