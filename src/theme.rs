//Not implemented

use tui::style::Color;

pub struct Theme {
    pub background: Color,
    pub text: Color,
    pub selection: Color,
    pub selection_text: Color,
}

impl Theme {
    pub fn default() -> Self {
        Theme {
            background: Color::Rgb(0x0B, 0x0E, 0x14),
            text: Color::Rgb(0xBF, 0xBD, 0xB6),
            selection: Color::Rgb(0xE6, 0xB4, 0x50),
            selection_text: Color::Rgb(0x0B, 0x0E, 0x14),
        }
    }
    pub fn gruvbox() -> Self {
        Theme {
            background: Color::Rgb(0x1d, 0x20, 0x21),
            text: Color::Rgb(0xF8, 0xF2, 0xD9),
            selection: Color::Rgb(0xE6, 0xB4, 0x50),
            selection_text: Color::Rgb(0x28, 0x28, 0x28),
        }
    }

}
