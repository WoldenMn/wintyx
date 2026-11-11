#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(debug_assertions)]
    {
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--export-bindings") {
            wintyx_lib::export_bindings();
            return;
        }
    }
    wintyx_lib::run()
}
