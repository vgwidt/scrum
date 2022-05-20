use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen}, execute,
};
use scrum_lib::*;
use unicode_width::UnicodeWidthStr;
use std::{io::{self, Write, Stdout}};
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

use crate::{db::*, Event};
use crate::ui::*;

const TICKRATE: u64 = 1000;

pub struct AppState {
    pub ticket_view_mode: TicketViewMode,
    pub active_menu_item: MenuItem,
    pub open_tickets: Vec<Tickets>,
    pub closed_tickets: Vec<Tickets>,
    pub ticket_list_state: TableState,
    pub edit_ticket: Tickets,
    pub messages: Vec<String>,
    pub input: String,
    pub prompt: String,
}

impl AppState {
    pub fn default() -> AppState {
        AppState {
            ticket_view_mode: TicketViewMode::Open,
            active_menu_item: MenuItem::Tickets,
            open_tickets: get_open_tickets(),
            closed_tickets: get_closed_tickets(),
            ticket_list_state: TableState::default(),
            edit_ticket: Tickets::default(),
            messages: Vec::new(),
            input: String::new(),
            prompt: "Enter Title".to_string(),
        }
    }
}

#[derive(PartialEq)]
pub enum TicketViewMode {
    Open,
    Closed,
}

#[derive(Copy, Clone, Debug)]
pub enum MenuItem {
    Tickets,
    Notes,
    EditForm,
    NoteForm,
    ConfirmForm,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Tickets => 0,
            MenuItem::Notes => 1,
            MenuItem::EditForm => todo!(),
            MenuItem::NoteForm => todo!(),
            MenuItem::ConfirmForm => todo!(),
        }
    }
}

pub fn run(app: &mut AppState) -> Result<(), Box<dyn std::error::Error>> {

        enable_raw_mode().expect("raw mode");
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

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
    
        let ticket_menu_titles = vec!["Tickets", "Add", "Edit", "Note (Add)", "Toggle Open/Closed View", "0: Toggle Open/Close", "Quit"];
        let edit_menu_titles = vec!["Edit (Press escape to cancel)"]; //Convert to const?
        let note_menu_titles = vec!["Add note (Press escape to cancel)"]; //Convert to const?
        let confirm_menu_titles = vec!["Confirmation (Press escape to cancel)"]; //Convert to const?
        let notes_menu_titles = vec!["Notes"]; //Convert to const?
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
                           // Constraint::Length(1), //To make room for the footer which doesn't exist at the moment
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
                    MenuItem::Notes => {
                        menu_titles = &note_menu_titles;    
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
                    .block(Block::default().title(" Menu").borders(Borders::ALL))
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
                        let chunks = Layout::default().direction(Direction::Vertical)
                            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),).split(chunks[1]);
                        let (input, output) = render_edit_form(app);
                        rect.render_widget(input, chunks[0]);
                        rect.render_widget(output, chunks[1]);
                        //Dangerous, if we add more fields this needs to be changed
                        if app.messages.len() < 3 {
                         rect.set_cursor(chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1,)
                        }
                    }
                    MenuItem::NoteForm => {
                        let chunks = Layout::default().direction(Direction::Vertical)
                            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),).split(chunks[1]);
                        let (input, output) = render_edit_form(app);
                        rect.render_widget(input, chunks[0]);
                        rect.render_widget(output, chunks[1]);
                        //Dangerous, if we add more fields this needs to be changed
                        if app.messages.len() < 1 {
                         rect.set_cursor(chunks[1].x + app.input.width() as u16 + 1, chunks[1].y + 1,)
                        }
                    },
                    MenuItem::ConfirmForm => todo!(),
                    MenuItem::Notes => {
                       // let notelist = render_notes(app);
                        rect.render_stateful_widget(notelist, chunks[0], &mut app.ticket_list_state);
                    },
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
                                    init_add_ticket(app).expect("Cannot add ticket");
                        }
                            KeyCode::Char('e') => {
                                edit_ticket_at_index(app).expect("Cannot edit ticket");
                            }
                            KeyCode::F(8) => {
                                match app.ticket_view_mode {
                                    TicketViewMode::Open => {
                                        //delete_ticket_at_index(&mut app).expect("Cannot delete ticket");
                                    }
                                    TicketViewMode::Closed => {
                                        remove_ticket_at_index(app).expect("Cannot remove ticket");
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
                                init_add_note(app).unwrap();
                            }
                            KeyCode::Down => {      
                                if let Some(selected) = app.ticket_list_state.selected() {
    
                                    let mut amount_tickets = 0;
    
                                    if app.ticket_view_mode == TicketViewMode::Open {
                                        amount_tickets = app.open_tickets.len().try_into().unwrap();
                                    } else if app.ticket_view_mode == TicketViewMode::Closed {
                                        amount_tickets = app.closed_tickets.len().try_into().unwrap();
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
                                        amount_tickets = app.open_tickets.len().try_into().unwrap();
                                    } else if app.ticket_view_mode == TicketViewMode::Closed {
                                        amount_tickets = app.closed_tickets.len().try_into().unwrap();
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
                               toggle_ticket_status(app).expect("Cannot toggle ticket status");                         
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
                                add_ticket(app).unwrap();
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
                                add_note(app).unwrap();
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
                MenuItem::Notes => todo!(),
            }
            
        }
        Ok(())  
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
        amount_tickets = app.open_tickets.len().try_into().unwrap();
    } else if app.ticket_view_mode == TicketViewMode::Closed {
        amount_tickets = app.closed_tickets.len().try_into().unwrap();
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


pub fn update_ticket_count(app: &mut AppState) {
    app.open_tickets = get_open_tickets();
    app.closed_tickets = get_closed_tickets();
}

fn update_db(app: &AppState) {
    //Concatenate open and closed tickets
    let mut all_tickets = app.open_tickets.clone();
    all_tickets.append(&mut app.closed_tickets.clone());
    write_changes(&all_tickets).unwrap();
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