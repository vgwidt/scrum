mod client;
mod db;
mod app;
mod ui;

use app::*;

enum Event<I> {
    Input(I),
    Tick,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    //Initialize AppState
    let mut app = AppState::default();
    //Initialize DB
    update_ticket_count(&mut app);
    //Run the app
    run(&mut app).unwrap();

    Ok(())
}

