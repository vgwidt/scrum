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
use unicode_width::UnicodeWidthStr;
use std::{io::{self, Write}, option};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs, TableState, Wrap,
    },
    Terminal
};
use db::*;
use form::*;
use crate::app::*;

const TICKRATE: u64 = 1000;

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
        open_count: 0,
        closed_count: 0,
        ticket_list_state: TableState::default(),
        edit_ticket: Tickets::default(),
        messages: Vec::new(),
        input: String::new(),
        prompt: "Enter Title".to_string(),
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

    enable_raw_mode().expect("raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let ticket_menu_titles = vec!["Tickets", "Add", "Edit", "Note (Add)", "Toggle Open/Closed View", "0: Toggle Open/Close", "Quit"];
    let edit_menu_titles = vec!["Edit (Press escape to cancel)"]; //Convert to const?
    let note_menu_titles = vec!["Add note (Press escape to cancel)"]; //Convert to const?
    let confirm_menu_titles = vec!["Confirmation (Press escape to cancel)"]; //Convert to const?
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
                MenuItem::NoteForm => {
                    menu_titles = &note_menu_titles;
                }
                MenuItem::ConfirmForm => {
                    menu_titles = &confirm_menu_titles;
                },     
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
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match app.active_menu_item {
                MenuItem::Tickets => {
                    let tickets_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [Constraint::Percentage(40), Constraint::Percentage(60)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_tickets(&app);
                    rect.render_stateful_widget(left, tickets_chunks[0], &mut app.ticket_list_state);
                    rect.render_widget(right, tickets_chunks[1]);
                }
                MenuItem::EditForm => {
                    let edit_form_chunks = Layout::default().direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),).split(chunks[1]);
                    let (input, output) = render_edit_form(&mut app);
                    rect.render_widget(input, edit_form_chunks[0]);
                    rect.render_widget(output, edit_form_chunks[1]);
                    //Dangerous, if we add more fields this needs to be changed
                    if app.messages.len() < 3 {
                     rect.set_cursor(chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1,)
                    }
                }
                MenuItem::NoteForm => {
                    let edit_form_chunks = Layout::default().direction(Direction::Vertical)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),).split(chunks[1]);
                    let (input, output) = render_edit_form(&mut app);
                    rect.render_widget(input, edit_form_chunks[0]);
                    rect.render_widget(output, edit_form_chunks[1]);
                    //Dangerous, if we add more fields this needs to be changed
                    if app.messages.len() < 1 {
                     rect.set_cursor(chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1,)
                    }
                },
                MenuItem::ConfirmForm => todo!(),
            }
            
        })?;

        match app.active_menu_item{
            MenuItem::Tickets => {
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
                        KeyCode::Char('a') => {
                                init_add_ticket(&mut app).expect("Cannot add ticket");
                    }
                        KeyCode::Char('e') => {
                            edit_ticket_at_index(&mut app).expect("Cannot edit ticket");
                        }
                        KeyCode::F(8) => {
                            match app.ticket_view_mode {
                                TicketViewMode::Open => {
                                    //delete_ticket_at_index(&mut app).expect("Cannot delete ticket");
                                }
                                TicketViewMode::Closed => {
                                    remove_ticket_at_index(&mut app).expect("Cannot remove ticket");
                                }
                            }
                            
                        }
                        KeyCode::Char('t') => {
                            match app.ticket_view_mode {
                                TicketViewMode::Open => {
                                    app.ticket_view_mode = TicketViewMode::Closed;
                                    //set index to 0 to prevent crash
                                    app.ticket_list_state.select(Some(0));
                                }
                                TicketViewMode::Closed => {
                                    app.ticket_view_mode = TicketViewMode::Open;
                                    //set index to 0 to prevent crash
                                    app.ticket_list_state.select(Some(0));
                                }
                            }

                        }
                        KeyCode::Char('n') => {
                            init_add_note(&mut app);
                        }
                        KeyCode::Down => {      
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
                        KeyCode::Up => {
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
                        KeyCode::Char('0')=> {
                           toggle_ticket_status(&mut app).expect("Cannot toggle ticket status");                         
                        }
                        KeyCode::Tab => {
                             app.active_menu_item = MenuItem::EditForm;
                        }
                        _ => {}
                    },
                    Event::Tick => {}
                }
            }
            MenuItem::EditForm => {
                match rx.recv()? {           
                    Event::Input(event) => match event.code {
                    KeyCode::Enter => {
                        app.messages.push(app.input.drain(..).collect());
                        if app.messages.len() == 1 {
                            app.prompt = "Enter Description".to_string();
                            app.input = app.edit_ticket.description.to_string();
                        }
                        else if app.messages.len() == 2 {
                            app.prompt = "Enter Priority".to_string();
                            app.input = app.edit_ticket.priority.to_string();
                        }
                        else if app.messages.len() == 3 {
                            app.prompt = "Hit Enter to save or Esc to cancel".to_string();
                            app.input = app.messages[0].clone() + "\n" + &app.messages[1].clone() + "\n" + &app.messages[2].clone();
                        }
                        else if app.messages.len() >= 3 {
                            add_ticket(&mut app);
                            }
                    }
                    KeyCode::Char(c) => {
                        //Only allow edit up to 3 lines
                        if app.messages.len() < 3 {
                        app.input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        //Only allow edit up to 3 lines
                        if app.messages.len() < 3 {
                        app.input.pop();
                        }
                    }
                    KeyCode::Left => {
                        //Move cursor left
                    }
                    KeyCode::Right => {
                        //Move cursor right

                    }
                    KeyCode::Esc => {
                        //return to Ticket menu without saving
                        app.active_menu_item = MenuItem::Tickets;
                        app.input = String::new();
                        app.messages = Vec::new();
                    }
                    _ => {}
                },
                    Event::Tick => {}
                }
            },
            MenuItem::NoteForm => {
                match rx.recv()? {           
                    Event::Input(event) => match event.code {
                    KeyCode::Enter => {
                        app.messages.push(app.input.drain(..).collect());
                        if app.messages.len() == 1 {
                            app.prompt = "Hit Enter to save or Esc to cancel".to_string();
                            app.input = app.messages[0].clone();
                        }
                        else if app.messages.len() >= 2 {
                           let newnote = Note {
                                text: app.messages[0].clone(),
                                created_at: Utc::now(),
                                updated_at: Utc::now(),
                            };
                            //Init vector if it doesn't exist
                            if app.edit_ticket.notes.is_none() {
                                app.edit_ticket.notes = Some(Vec::new());
                            }
                            //Add note to vector
                            app.edit_ticket.notes.as_mut().unwrap().push(newnote);
                            add_note(&mut app);
                    }
                }
                    KeyCode::Char(c) => {
                        //Only allow edit up to 1 line
                        if app.messages.len() < 1 {
                        app.input.push(c);
                        }
                    }
                    KeyCode::Backspace => {
                        //Only allow edit up to 1 line
                        if app.messages.len() < 1 {
                        app.input.pop();
                        }
                    }
                    KeyCode::Esc => {
                        //return to Ticket menu without saving
                        app.active_menu_item = MenuItem::Tickets;
                        app.input = String::new();
                        app.messages = Vec::new();
                    }
                    _ => {}
                },
                    Event::Tick => {}
                }
            },
            MenuItem::ConfirmForm => todo!(),
        }
        
    }

    Ok(())
}


fn render_tickets<'a>(app: &AppState) -> (Table<'a>, Paragraph<'a>) {
 
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
            let ticket_list = app.closed_tickets.clone();
            for ticket in ticket_list {
                if ticket.status.to_string() == "Closed" { //FIX THIS!
                    tickets.push(ticket);
                }
            }
        }
    }


    let mut selected_ticket = Tickets {
        id: 0,
        title: "".to_owned(),
        description: "No tickets".to_owned(),
        notes: None,
        status: TicketStatus::Open,
        priority: "".to_owned(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    //If there is at least ticket
    if tickets.len() > 0 {
    selected_ticket = tickets
        .get(
            app.ticket_list_state
                .selected()
                .expect("ticket list state"),
        )
        .expect("selected ticket")
        .clone();
    }

    let rows = tickets.iter().enumerate().map(|(i, item)| {
        Row::new(vec![
            Cell::from(item.id.to_string()),
            Cell::from(item.title.clone()),
            Cell::from(item.created_at.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()),
            Cell::from(item.updated_at.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()),
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

    //Create vector of spans for each note in selected ticket
    let mut notespan = Vec::new();
    if selected_ticket.notes.is_some() {
        let notes = selected_ticket.notes.clone().unwrap();
        for note in notes {
            notespan.push(
                Spans::from(vec![
                Span::raw(note.updated_at.format("%Y-%m-%d %H:%M").to_string()),
                Span::styled(" Update: ", Style::default().fg(Color::Yellow)),
                Span::raw(note.text.clone()),
            ]));
        }
    }

    let mut text = vec![Spans::from(vec![
        Span::styled("Title: ", Style::default().fg(Color::Yellow)),
        Span::raw(selected_ticket.title.clone()),
    ]),
    Spans::from(vec![Span::raw("\n")]),
    Spans::from(vec![
        Span::styled("Description: ", Style::default().fg(Color::Yellow)),
        Span::raw(selected_ticket.description.clone()),
    ]),
    Spans::from(vec![Span::raw("\n")]),
    Spans::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Yellow)),
        Span::raw(selected_ticket.status.to_string().to_owned()),
    ]),
    Spans::from(vec![Span::raw("\n")]),
    Spans::from(vec![
        Span::styled("Priority: ", Style::default().fg(Color::Yellow)),
        Span::raw(selected_ticket.priority.clone()),
    ]),
    Spans::from(vec![Span::raw("\n")]),
    Spans::from(vec![
        Span::styled("Created At: ", Style::default().fg(Color::Yellow)),
        Span::raw(selected_ticket.created_at.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()),
    ]),
    Spans::from(vec![Span::raw("\n")]),
    Spans::from(vec![
        Span::styled("Updated At: ", Style::default().fg(Color::Yellow)),
        Span::raw(selected_ticket.updated_at.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string()),
    ]),
    Spans::from(vec![Span::raw("\n")]),
    ];

    //add notespan to text
    text.extend(notespan);

    let ticket_detail = Paragraph::new(text)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Ticket Detail")
                .border_type(BorderType::Plain),
        ).wrap(Wrap { trim: true });
    
    (list, ticket_detail)
}

fn init_add_ticket(app: &mut AppState) -> Result<(), Error> {

    app.edit_ticket.id = -7;
    app.edit_ticket.status = TicketStatus::Open;

    app.prompt = "Enter Title".to_string();
    app.active_menu_item = MenuItem::EditForm;
    Ok(())
}

fn add_ticket (app: &mut AppState) -> Result<(), Error> {
    if let Some(selected) = app.ticket_list_state.selected() {
    //if new
    if app.edit_ticket.id == -7 {
        //Generate unique ID
        let parsed: Vec<Tickets> = read_db().unwrap();
        let mut max_id = 0;
        for ticket in parsed.iter() {
            if ticket.id > max_id {
                max_id = ticket.id;
            }
        }
        app.edit_ticket.id = max_id + 1;

        app.edit_ticket.title = app.messages[0].trim().to_string();
        app.edit_ticket.description = app.messages[1].trim().to_string();
        app.edit_ticket.priority = app.messages[2].trim().to_string();
        app.edit_ticket.created_at = Utc::now();
        app.edit_ticket.updated_at = Utc::now();

        app.open_tickets.push(app.edit_ticket.clone());

    } else {

        //replace ticket at selected index
        app.edit_ticket.title = app.messages[0].trim().to_string();
        app.edit_ticket.description = app.messages[1].trim().to_string();
        app.edit_ticket.priority = app.messages[2].trim().to_string();
        app.edit_ticket.updated_at = Utc::now();

        //Makes sure to update the right index by checking if on open or close page, very inefficient and requires blocking changing in other menus
        match app.ticket_view_mode {
            TicketViewMode::Open => {
                app.open_tickets[selected] = app.edit_ticket.clone();
            },
            TicketViewMode::Closed => {
                app.closed_tickets[selected] = app.edit_ticket.clone();
            },
        }    
    }


    update_db(&app);
    update_ticket_count(app);
    render_tickets(app);

    app.edit_ticket = Tickets::default();
    app.input = String::new();
    app.messages = Vec::new();
    app.active_menu_item = MenuItem::Tickets;
    }

    Ok(())
}


pub fn edit_ticket_at_index(app: &mut AppState) -> Result<(), Error> {
     if let Some(selected) = app.ticket_list_state.selected() {
        app.prompt = "Enter Title".to_string();
        match app.ticket_view_mode {
            TicketViewMode::Open => {
                if app.open_tickets.len() != 0 {
                    app.edit_ticket = app.open_tickets[selected].clone();
                    app.input = app.edit_ticket.title.to_string();
                    app.active_menu_item = MenuItem::EditForm;
                }
            },
            TicketViewMode::Closed => {
                if app.closed_tickets.len() != 0 {
                    app.edit_ticket = app.closed_tickets[selected].clone();
                    app.input = app.edit_ticket.title.to_string();
                    app.active_menu_item = MenuItem::EditForm;
                }
            },
        }

     }
    
    Ok(())

}

pub fn init_add_note(app: &mut AppState) -> Result<(), Error> {
   //If menus exactly the same, I could set an AppState variable that sets the amount of expected messages to save from having to create different forms
    app.messages = Vec::new();
    if let Some(selected) = app.ticket_list_state.selected() {
        app.prompt = "Enter Note".to_string();
        match app.ticket_view_mode {
            TicketViewMode::Open => {
                if app.open_tickets.len() != 0 {
                    app.edit_ticket = app.open_tickets[selected].clone();
                    app.input = "".to_string();
                    app.active_menu_item = MenuItem::NoteForm;
                }
            },
            TicketViewMode::Closed => {
                if app.closed_tickets.len() != 0 {
                    app.edit_ticket = app.closed_tickets[selected].clone();
                    app.input = app.edit_ticket.title.to_string();
                    app.active_menu_item = MenuItem::NoteForm;
                }
            },
        }

     }
    
    Ok(())
}

pub fn add_note(app: &mut AppState) -> Result<(), Error> {
    if let Some(selected) = app.ticket_list_state.selected() {
    match app.ticket_view_mode {
        TicketViewMode::Open => {
            app.open_tickets[selected] = app.edit_ticket.clone();
        },
        TicketViewMode::Closed => {
            app.closed_tickets[selected] = app.edit_ticket.clone();
        },
    }  
    update_db(&app);
    update_ticket_count(app);
    render_tickets(app);

    app.edit_ticket = Tickets::default();
    app.input = String::new();
    app.messages = Vec::new();
    app.active_menu_item = MenuItem::Tickets;
}
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
        update_selected_ticket(app, selected);
    }

    update_db(&app);
    update_ticket_count(app);

    Ok(())
     
}

fn update_selected_ticket(app: &mut AppState, selected: usize) {
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

fn toggle_ticket_status(app: &mut AppState) -> Result<(), Error> {
    if let Some(selected) = app.ticket_list_state.selected() {
        match app.ticket_view_mode {
            TicketViewMode::Open => {
                if app.open_tickets.len() != 0 {
                    app.open_tickets[selected].status = TicketStatus::Closed;
                    app.open_tickets[selected].updated_at = Utc::now();
                }
            },
            TicketViewMode::Closed => {
                if app.closed_tickets.len() != 0 {
                app.closed_tickets[selected].status = TicketStatus::Open;
                app.closed_tickets[selected].updated_at = Utc::now();
                }
            },
        }

        update_selected_ticket(app, selected);

        update_db(&app);
        update_ticket_count(app);
        render_tickets(app);
    }
    Ok(())
}

fn add_sample_ticket(app: &mut AppState) -> Result<(), Error> {

    let parsed: Vec<Tickets> = read_db().unwrap();
    let mut max_id = 0;
    for ticket in parsed.iter() {
        if ticket.id > max_id {
            max_id = ticket.id;
        }
    }

    let new_ticket = Tickets {
        id: max_id + 1,
        title: "Make Coffee".to_owned(),
        description: "Extra dark".to_owned(),
        notes: None,
        status: TicketStatus::Open,
        priority: "High".to_owned(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    app.open_tickets.push(new_ticket);
    update_db(&app);
    update_ticket_count(app);
    render_tickets(app);
    Ok(())
}