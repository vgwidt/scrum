mod client;

use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen}, execute,
};
use serde::{Deserialize, Serialize};
use scrum_lib::*;
use std::{fs::{self, File}, process::exit};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
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

const DB_PATH: &str = "ticketdb.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

struct AppState {
    ticket_view_mode: TicketViewMode,
    active_menu_item: MenuItem,
    ticket_list: Vec<Tickets>,
    open_count: i32,
    closed_count: i32,
    ticket_list_state: TableState,
}

#[derive(PartialEq)]
enum TicketViewMode {
    Open,
    Closed,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Tickets,
    EditForm,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Tickets => 0,
            MenuItem::EditForm => 1,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    enable_raw_mode().expect("raw mode");

    let mut app = AppState {
        ticket_view_mode: TicketViewMode::Open,
        active_menu_item: MenuItem::Tickets,
        ticket_list: read_db()?,
        open_count: 0,
        closed_count: 0,
        ticket_list_state: TableState::default(),
    };

    //Count tickets with status of "Open"
    update_ticket_count(&mut app);

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
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

    let menu_titles = vec!["Tickets", "Add", "Edit", "Delete", "Quit", "Opened Tickets", "Closed Tickets"];
    //let mut active_menu_item = MenuItem::Tickets;
    //let mut ticket_list_state = TableState::default();
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
                        add_ticket().expect("Cannot add ticket");
                        app.ticket_list = read_db()?;
                        update_ticket_count(&mut app);
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
                    remove_ticket_at_index(&mut app.ticket_list_state).expect("Cannot remove ticket");
                    app.ticket_list = read_db()?;
                    update_ticket_count(&mut app);
                }
                _ => {}
                }
                }
                KeyCode::Char('c') => {
                    app.ticket_view_mode = TicketViewMode::Closed;
                    //set index to 0 to prevent crash
                    app.ticket_list_state.select(Some(0));
                }
                KeyCode::Char('o') => {
                    app.ticket_view_mode = TicketViewMode::Open;
                    //set index to 0 to prevent crash
                    app.ticket_list_state.select(Some(0));
                }
                KeyCode::Char('s') => {
                    //save
                }
                KeyCode::Down => {
                    if let Some(selected) = app.ticket_list_state.selected() {
                        
                        let mut amount_tickets = 0;

                        if app.ticket_view_mode == TicketViewMode::Open {
                            amount_tickets = app.open_count;
                        } else if app.ticket_view_mode == TicketViewMode::Closed {
                            amount_tickets = app.closed_count;
                        }
                        
                        if selected >= (amount_tickets - 1).try_into().unwrap() {
                            app.ticket_list_state.select(Some(0));
                        } else {
                            app.ticket_list_state.select(Some(selected + 1));                            
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = app.ticket_list_state.selected() {

                        let mut amount_tickets = 0;

                        if app.ticket_view_mode == TicketViewMode::Open {
                            amount_tickets = app.open_count;
                        } else if app.ticket_view_mode == TicketViewMode::Closed {
                            amount_tickets = app.closed_count;
                        }

                        if selected > 0 {
                            app.ticket_list_state.select(Some(selected - 1));
                        } else {
                            app.ticket_list_state.select(Some((amount_tickets - 1).try_into().unwrap()));
                        }
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
    
    //let ticket_list = read_db().expect("can fetch ticket list");
    let ticket_list = app.ticket_list.clone();

    //Sort tickets by status
    let mut tickets = Vec::new();
    match app.ticket_view_mode {
        TicketViewMode::Open => {
            for ticket in ticket_list {
                if ticket.status.to_string() == "Open" { //FIX THIS!
                    tickets.push(ticket);
                }
            }
        }
        TicketViewMode::Closed => {
            for ticket in ticket_list {
                if ticket.status.to_string() == "Closed" { //FIX THIS!
                    tickets.push(ticket);
                }
            }
        }
    }
    
    let selected_ticket = tickets
        .get(
            ticket_list_state
                .selected()
                .expect("there is always a selected ticket"),
        )
        .expect("exists")
        .clone();

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

fn read_db() -> Result<Vec<Tickets>, Error> {
    //if DB exists, read it, otherwise create it
    // if !DB_PATH.exists() {
    //     let _ = create_db();
    // }
    
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Tickets> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn add_ticket() -> Result<Vec<Tickets>, Error> {

    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Tickets> = serde_json::from_str(&db_content)?;
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

    parsed.push(new_ticket);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}

fn remove_ticket_at_index(ticket_list_state: &mut TableState) -> Result<(), Error> {
    if let Some(selected) = ticket_list_state.selected() {
        if selected != 0 {
        let db_content = fs::read_to_string(DB_PATH)?;
        let mut parsed: Vec<Tickets> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
        // Only deincrement if ticket ID is not 0
        
             ticket_list_state.select(Some(selected - 1));
        }
    }

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

fn render_edit_form<'a>(ticket_list_state: &TableState, app: &'a AppState) -> Paragraph<'a> {
        
    //Get ticket at selected index
    let selected = ticket_list_state.selected().unwrap();
    let ticket = &app.ticket_list[selected];


    let mut text = vec![Spans::from(vec![
        Span::raw("Title:" ),
        Span::styled("line",Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(ticket.title.clone()),
    ]),
    Spans::from(vec![
        Span::raw("Description:" ),
        Span::styled("line",Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(ticket.description.clone()),
    ]),
    Spans::from(vec![
        Span::raw("Status:" ),
        Span::styled("line",Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(ticket.status.to_string()),
    ]),
    Spans::from(vec![
        Span::raw("Priority:" ),
        Span::styled("line",Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(ticket.priority.clone()),
    ]),
    Spans::from(vec![
        Span::raw("Created At:" ),
        Span::styled("line",Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(ticket.created_at.to_string()),
    ]),
    Spans::from(vec![
        Span::raw("Updated At:" ),
        Span::styled("line",Style::default().add_modifier(Modifier::ITALIC)),
        Span::raw(ticket.updated_at.to_string()),
    ]),
    ];

    Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Ticket")
                .border_type(BorderType::Plain),
        ) 
    
}


fn update_ticket_count(app: &mut AppState) {
    
    //Currently reiterates through all the tickets
    //Will be more efficient to increment count when added or deleted

    app.open_count = 0;
    app.closed_count = 0;

    for ticket in &app.ticket_list {
        if ticket.status.to_string() == "Open" { //FIX!
            app.open_count += 1;
        } else if ticket.status.to_string() == "Closed" {
            app.closed_count += 1;
        }
    }

}
