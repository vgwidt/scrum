use tui::widgets::TableState;
use crate::Tickets;

pub struct AppState {
    pub ticket_view_mode: TicketViewMode,
    pub active_menu_item: MenuItem,
    pub open_tickets: Vec<Tickets>,
    pub closed_tickets: Vec<Tickets>,
    pub open_count: i32,
    pub closed_count: i32,
    pub ticket_list_state: TableState,
    pub edit_ticket: Tickets,
    pub messages: Vec<String>,
    pub input: String,
    pub prompt: String,
}


#[derive(PartialEq)]
pub enum TicketViewMode {
    Open,
    Closed,
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Tickets,
    EditForm,
    NoteForm,
    ConfirmForm,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Tickets => 0,
            MenuItem::EditForm => 1,
            MenuItem::NoteForm => 2,
            MenuItem::ConfirmForm => 3,
        }
    }
}