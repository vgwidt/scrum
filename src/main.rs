mod client;
mod db;
mod form;
mod app;

use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen}, execute,
};
use scrum_lib::*;
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs, TableState,
    },
    Terminal
};
use db::*;
use form::*;
use crate::app::*;

const TICKRATE: u64 = 60000;

enum Event<I> {
    Input(I),
    Tick,
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    enable_raw_mode().expect("raw mode");

    let mut app = AppState {
        ticket_view_mode: TicketViewMode::Open,
        active_menu_item: MenuItem::Tickets,
        open_tickets: get_open_tickets(),
        closed_tickets: get_closed_tickets(),
        open_count: 0,
        closed_count: 0,
        ticket_list_state: TableState::default(),
    };

    update_ticket_count(&mut app);


    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(TICKRATE);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("event poll") {
                if let CEvent::Key(key) = event::read().expect("event read") {
                    tx.send(Event::Input(key)).expect("send");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let ticket_menu_titles = vec!["Tickets", "Add", "Edit", "Delete", "1: Opened Tickets", "2: Closed Tickets", "Quit"];
    let edit_menu_titles = vec!["Save", "Cancel", "Quit"]; //Convert to const?
    let mut menu_titles = &ticket_menu_titles;


    app.ticket_list_state.select(Some(0));


    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            match app.active_menu_item {
                MenuItem::Tickets => {
                    menu_titles = &ticket_menu_titles;
                }
                MenuItem::EditForm => {
                    menu_titles = &edit_menu_titles;
                }     
            }
            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(app.active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::LightYellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match app.active_menu_item {
                MenuItem::Tickets => {
                    let tickets_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_tickets(&app.ticket_list_state, &app);
                    rect.render_stateful_widget(left, tickets_chunks[0], &mut app.ticket_list_state);
                    rect.render_widget(right, tickets_chunks[1]);
                }
                MenuItem::EditForm => {
                    let edit_form_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(100), Constraint::Percentage(0)].as_ref(),
                        )
                        .split(chunks[1]);
                    let edit_form = render_edit_form(&app.ticket_list_state, &app);
                    rect.render_widget(edit_form, edit_form_chunks[0]);
                }
            }
            
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    terminal.clear()?;
                    let mut stdout = io::stdout();
                    execute!(stdout, LeaveAlternateScreen)?;
                    break;
                }
                KeyCode::Char('t') => app.active_menu_item = MenuItem::Tickets,
                KeyCode::Char('a') => {
                    match app.active_menu_item {
                        MenuItem::Tickets => {
                        add_ticket(&mut app).expect("Cannot add ticket");
                    }
                        _ => {}
                }
            }
                KeyCode::Char('e') => {
                    app.active_menu_item = MenuItem::EditForm;
                    //edit_ticket_at_index(&mut app).expect("Cannot edit ticket");
                }
                KeyCode::Char('d') => {
                    match app.active_menu_item {
                        MenuItem::Tickets => {
                            remove_ticket_at_index(&mut app).expect("Cannot remove ticket");
                }
                _ => {}
                }
                }
                KeyCode::Char('2') => {
                    match app.active_menu_item {
                        MenuItem::Tickets => {
                            app.ticket_view_mode = TicketViewMode::Closed;
                            //set index to 0 to prevent crash
                            app.ticket_list_state.select(Some(0));
                        }
                        _ => {}
                    }

                }
                KeyCode::Char('1') => {
                    match app.active_menu_item {
                        MenuItem::Tickets => {
                    app.ticket_view_mode = TicketViewMode::Open;
                    //set index to 0 to prevent crash
                    app.ticket_list_state.select(Some(0));
                        }
                        _ => {}
                    }
                }
                KeyCode::Char('s') => {
                    //save
                }
                KeyCode::Down => {
                    match app.active_menu_item {
                        MenuItem::Tickets => {
                            //only try if something is selected
                          
                    if let Some(selected) = app.ticket_list_state.selected() {
                        
                        let mut amount_tickets = 0;

                        if app.ticket_view_mode == TicketViewMode::Open {
                            amount_tickets = app.open_count;
                        } else if app.ticket_view_mode == TicketViewMode::Closed {
                            amount_tickets = app.closed_count;
                        }
                        
                        if amount_tickets == 0 {
                            continue;
                        }
                        if selected >= (amount_tickets - 1).try_into().unwrap() {
                            app.ticket_list_state.select(Some(0));
                        } else {
                            app.ticket_list_state.select(Some(selected + 1));                            
                        }
                    
                }
                }
                        _ => {}
               }
                }
                KeyCode::Up => {
                    match app.active_menu_item {
                        MenuItem::Tickets => {
                            
                            if let Some(selected) = app.ticket_list_state.selected() {

                        let mut amount_tickets = 0;

                        if app.ticket_view_mode == TicketViewMode::Open {
                            amount_tickets = app.open_count;
                        } else if app.ticket_view_mode == TicketViewMode::Closed {
                            amount_tickets = app.closed_count;
                        }
                        if amount_tickets == 0 {
                            continue;
                        }
                        if selected > 0 {
                            app.ticket_list_state.select(Some(selected - 1));
                        } else {
                            app.ticket_list_state.select(Some((amount_tickets - 1).try_into().unwrap()));
                        }
                    }
                
                }
                        _ => {}
               }
                }
                KeyCode::Char('c') => {

                    match app.active_menu_item {
                        MenuItem::Tickets => {
                        }
                        MenuItem::EditForm => {
                            app.active_menu_item = MenuItem::Tickets;
                        }
                    }
                }
                KeyCode::Char('[')=> {
                    match app.active_menu_item {
                        MenuItem::Tickets => {
                    //Close selected ticket
                    match app.ticket_view_mode {
                        TicketViewMode::Open => {
                            if let Some(selected) = app.ticket_list_state.selected() {
                                app.open_tickets[selected].status = TicketStatus::Closed;
        
                                update_ticket_count(&mut app);
                                update_db(&app);
                            }
                        }
                        TicketViewMode::Closed =>     {                   
                        }
                    }

                }
                        _ => {}
                    }
                }
                KeyCode::Char(']')=> {
                    match app.active_menu_item {
                        MenuItem::Tickets => {
                    //Open selected ticket
                        match app.ticket_view_mode {
                            TicketViewMode::Open => {
                                if let Some(selected) = app.ticket_list_state.selected() {
                                    app.open_tickets[selected].status = TicketStatus::Open;
            
                                    update_ticket_count(&mut app);
                                    update_db(&app);
                                }
                            }
                            TicketViewMode::Closed =>     {                   
                            }
                        }
                         }
                        _ => {}
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}


fn render_tickets<'a>(ticket_list_state: &TableState, app: &AppState) -> (Table<'a>, Table<'a>) {
    
 
    let mut tickets = Vec::new();
    match app.ticket_view_mode {
        TicketViewMode::Open => {
            let ticket_list = app.open_tickets.clone();
            for ticket in ticket_list {
                if ticket.status.to_string() == "Open" { //FIX THIS!
                    tickets.push(ticket);
                }
            }
        }
        TicketViewMode::Closed => {
            let ticket_list = app.open_tickets.clone();
            for ticket in ticket_list {
                if ticket.status.to_string() == "Closed" { //FIX THIS!
                    tickets.push(ticket);
                }
            }
        }
    }


    let mut selected_ticket = Tickets {
        id: 0,
        title: "No tickets".to_owned(),
        description: "".to_owned(),
        status: TicketStatus::Open,
        priority: "Low".to_owned(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    //If there is at least ticket
    if tickets.len() > 0 {
    selected_ticket = tickets
        .get(
            ticket_list_state
                .selected()
                .expect("there is always a selected ticket"),
        )
        .expect("exists")
        .clone();
    }

    let rows = tickets.iter().enumerate().map(|(i, item)| {
        Row::new(vec![
            Cell::from(item.id.to_string()),
            Cell::from(item.title.clone()),
            Cell::from(item.created_at.format("%Y-%m-%d %H:%M").to_string()),
            Cell::from(item.updated_at.format("%Y-%m-%d %H:%M").to_string()),
            Cell::from(item.priority.to_string()),
        ])
    });

    let list = Table::new(rows)
        .block(Block::default().borders(Borders::ALL).title("Tickets"))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black))
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "ID",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Title",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Creation Date",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Last Updated",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Priority",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .widths(&[
            Constraint::Percentage(10),
            Constraint::Percentage(38),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(12),
        ]);

    let ticket_detail = Table::new(vec![
        Row::new(vec![
        Cell::from(Span::raw(selected_ticket.id.to_string())),
        Cell::from(Span::raw(selected_ticket.title.clone())),
        Cell::from(Span::raw(selected_ticket.description.clone())),
        Cell::from(Span::raw(selected_ticket.status.to_string().to_owned())),
        Cell::from(Span::raw(selected_ticket.created_at.to_string())),
    ]),
    ])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Title",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Description",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Status",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);
    
    (list, ticket_detail)
}

fn add_ticket(app: &mut AppState) -> Result<(), Error> {

    let parsed: Vec<Tickets> = read_db().unwrap();
    let mut max_id = 0;
    for ticket in parsed.iter() {
        if ticket.id > max_id {
            max_id = ticket.id;
        }
    }



    let new_ticket = Tickets {
        id: max_id + 1,
        title: "Zabbix Setup".to_owned(),
        description: "Setup Zabbix".to_owned(),
        status: TicketStatus::Open,
        priority: "Low".to_owned(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    // let request = Request {
    //     action: TicketAction::Create,
    //     ticket: new_ticket.clone()
    // };
    //client::send_request(request);

    app.open_tickets.push(new_ticket);
    update_db(&app);
    update_ticket_count(app);
    Ok(())
}



fn remove_ticket_at_index(app: &mut AppState) -> Result<(), Error> {
    if let Some(selected) = app.ticket_list_state.selected() {
       
        match app.ticket_view_mode {
            TicketViewMode::Open => {
                if app.open_tickets.len() != 0 {
                    app.open_tickets.remove(selected);
                    update_db(&app);
                }
            }
            TicketViewMode::Closed => {
                if app.closed_tickets.len() != 0 {
                app.closed_tickets.remove(selected);
                update_db(&app);
                }
            }
        }
        //Set new selected ticket
        let mut amount_tickets = 0;

        if app.ticket_view_mode == TicketViewMode::Open {
            amount_tickets = app.open_count;
        } else if app.ticket_view_mode == TicketViewMode::Closed {
            amount_tickets = app.closed_count;
        }

        if amount_tickets == 0 {
            app.ticket_list_state.select(None);
        }
        //If there is at least one ticket, it will move up if greater than 1, otherwise it will stay at 0
        if selected > 0 {
            app.ticket_list_state.select(Some(selected - 1));
        } else {
            app.ticket_list_state.select(Some((0).try_into().unwrap()));
        }
    }

    update_db(&app);
    update_ticket_count(app);

    Ok(())
     
}

// pub fn edit_ticket_at_index(app: &AppState) -> Result<(), Error> {
//     if let Some(selected) = app.ticket_list_state.selected() {
//         if selected != 0 {
//             let db_content = fs::read_to_string(DB_PATH)?;
//             let mut parsed: Vec<Tickets> = serde_json::from_str(&db_content)?;
//             let ticket = &mut parsed[selected];
//             render_edit_form();
//         }
//     }
    
// Ok(())

// }



fn update_ticket_count(app: &mut AppState) {
    
    app.open_tickets = get_open_tickets();
    app.closed_tickets = get_closed_tickets();

    //Currently reiterates through all the tickets
    //Will be more efficient to increment count when added or deleted

    app.open_count = 0;
    app.closed_count = 0;

    for ticket in &app.open_tickets {
            app.open_count += 1;
    }
    for ticket in &app.closed_tickets {
            app.closed_count += 1;
    }


}

fn update_db(app: &AppState) {
    //Concatenate open and closed tickets
    let mut all_tickets = app.open_tickets.clone();
    all_tickets.append(&mut app.closed_tickets.clone());
    write_changes(&all_tickets);
}