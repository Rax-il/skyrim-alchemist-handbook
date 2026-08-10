// Прячет консольное окно на Windows (аналог windows_subsystem = "windows"
// в исходной egui-версии) — без этого атрибута рядом с окном приложения
// на Windows всплывало бы чёрное консольное окно.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    alchemist_lib::run();
}
