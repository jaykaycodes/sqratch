// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Our main entry point for the application
fn main() {
    sqratch_lib::run()
}
