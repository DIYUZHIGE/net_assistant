#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[tokio::main]
async fn main() {
    net_assistant_lib::run()
}
