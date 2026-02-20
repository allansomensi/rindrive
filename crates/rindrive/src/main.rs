#![windows_subsystem = "windows"]

fn main() {
    rindrive_i18n::localize();
    rindrive_gui::run().unwrap();
}
