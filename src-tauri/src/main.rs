// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod __allow_unused {
    extern crate specta;
    extern crate tauri_plugin_log;
    extern crate tauri_plugin_opener;
}

fn main() {
    colorist_lib::run();
}
