// Sin consola extra en Windows en la version final.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    karaoke_lib::run()
}
