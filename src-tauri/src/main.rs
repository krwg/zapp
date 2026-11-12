// ╔════════════════════════════════════════════════════╗
// ║                       main                       ║
// ╚════════════════════════════════════════════════════╝

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if std::env::args().any(|a| a == zapp_lib::helper::HELPER_FLAG) {
        
        zapp_lib::helper::run_helper();
        return;
    }
    zapp_lib::run();
}
