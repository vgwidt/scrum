//Not implemented

use tui::style::Color;

pub struct Theme {
    pub background: Color,
    pub selection: Color,
    pub selection_text: Color,
    pub border: Color,
    pub border_focus: Color,
}

impl Theme {
    pub fn default() -> Self {
        Theme {
            background: Color::Rgb(0x00, 0x00, 0x00),
            selection: Color::Rgb(0x00, 0x00, 0x00),
            selection_text: Color::Rgb(0x00, 0x00, 0x00),
            border: Color::Rgb(0x00, 0x00, 0x00),
            border_focus: Color::Rgb(0x00, 0x00, 0x00),
        }
    }
}