mod client;
mod db;
mod app;
mod ui;

use scrum_lib::*;
use db::*;
use app::*;
use tui::widgets::TableState;

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    let mut app = AppState {
        ticket_view_mode: TicketViewMode::Open,
        active_menu_item: MenuItem::Tickets,
        open_tickets: get_open_tickets(),
        closed_tickets: get_closed_tickets(),
        ticket_list_state: TableState::default(),
        edit_ticket: Tickets::default(),
        messages: Vec::new(),
        input: String::new(),
        prompt: "Enter Title".to_string(),
    };

    update_ticket_count(&mut app);

    run(&mut app).unwrap();

    Ok(())
}

