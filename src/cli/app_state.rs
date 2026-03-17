// Copyright (c) 2026 bazelik-null

#[derive(Default)]
pub struct AppState {
    pub is_debug: bool,
}

impl AppState {
    pub fn toggle_debug(&mut self) {
        self.is_debug = !self.is_debug;
        println!("[INFO]: Debug mode: {}", self.is_debug);
    }
}
