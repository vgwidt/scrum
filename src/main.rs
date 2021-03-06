mod client;
mod db;
mod app;
mod ui;
mod ticket;
mod theme;

use app::*;
use ticket::*;

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    //Initialize AppState
    let mut app = AppState::default();
    //Initialize DB
    update_ticket_count(&mut app);
    //Initialize theme
    let theme = theme::Theme::default();
    //Run the app
    run(&mut app).unwrap();

    Ok(())
}

