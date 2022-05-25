use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEvent, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, LeaveAlternateScreen, EnterAlternateScreen}, execute,
};
use unicode_width::UnicodeWidthStr;
use std::{io::{self, Write, Stdout}, sync::Arc};
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

use scrum_lib::*;
use crate::{db::*, Event};
use crate::ui::*;
use crate::ticket::*;
use crate::theme::*;

const TICKRATE: u64 = 1000;

pub struct AppState {
    pub ticket_view_mode: TicketViewMode,
    pub active_menu_item: MenuItem,
    pub open_tickets: Vec<Tickets>,
    pub closed_tickets: Vec<Tickets>,
    pub ticket_list_state: TableState,
    pub edit_priority_state: ListState,
    pub edit_status_state: ListState,
    pub edit_note_state: ListState,
    pub edit_ticket: Tickets,
    pub edit_focus: EditItem,
    pub messages: Vec<String>,
    pub input: String,
    pub prompt: String,
    pub scroll: u16,
    pub sort_by: SortBy,
    pub theme: Theme,
}

impl AppState {
    pub fn default() -> AppState {
        AppState {
            ticket_view_mode: TicketViewMode::Open,
            active_menu_item: MenuItem::Tickets,
            open_tickets: get_open_tickets(),
            closed_tickets: get_closed_tickets(),
            ticket_list_state: TableState::default(),
            edit_priority_state: ListState::default(),
            edit_status_state: ListState::default(),
            edit_note_state: ListState::default(),
            edit_ticket: Tickets::default(),
            edit_focus: EditItem::Title,
            messages: Vec::new(),
            input: String::new(),
            prompt: "Enter Title".to_string(),
            scroll: 0,
            sort_by: SortBy::ID,
            theme: Theme::gruvbox(),
        }
    }
}

#[derive(PartialEq)]
pub enum EditItem {
    Title,
    Description,
    Priority,
    Status,
    Notes,
}

impl EditItem {
    pub fn set_focus(&mut self, new_focus: EditItem) {
        *self = new_focus;
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
    EditForm,
    NoteForm,
    ConfirmForm,
    Help,
}

pub enum SortBy {
    ID,
    Title,
    Priority,
    Updated,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Tickets => 0,
            MenuItem::EditForm => 1,
            MenuItem::NoteForm => 2,
            MenuItem::ConfirmForm => 3,
            MenuItem::Help => 4,
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
    

        let edit_menu_titles = vec!["Edit (Press escape to cancel)"]; //Convert to const?
        let note_menu_titles = vec!["Add note (Press escape to cancel)"]; //Convert to const?
        let confirm_menu_titles = vec!["Confirmation (Press escape to cancel)"]; //Convert to const?
        let help_menu_titles = vec!["Help (Press escape to return)"]; //Convert to const?
        
    
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

                let openorclosed = match app.ticket_view_mode {
                    TicketViewMode::Open => "View Closed",
                    TicketViewMode::Closed => "View Open",
                };
                let ticket_menu_titles = vec!["Tickets", "Add", "Edit", "Note (+)", openorclosed, "Help", "Quit"];
                let mut menu_titles = &ticket_menu_titles;
                    
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
                    MenuItem::Help => {
                        menu_titles = &help_menu_titles
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
                                    .fg(app.theme.selection)
                                    .add_modifier(Modifier::UNDERLINED),
                            ),
                            Span::styled(rest, Style::default().fg(app.theme.text)),
                        ])
                    })
                    .collect();
    
                let tabs = Tabs::new(menu)
                    .select(app.active_menu_item.into())
                    .block(Block::default().title(" Menu").borders(Borders::ALL))
                    .style(Style::default().fg(Color::White))
                    .highlight_style(Style::default().fg(app.theme.selection))
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
                        let editchunk = Layout::default().direction(Direction::Vertical)
                            .constraints([Constraint::Percentage(40),Constraint::Percentage(30),Constraint::Percentage(30)].as_ref())
                            .split(chunks[1]);
                        
                        let chunk1 = Layout::default().direction(Direction::Vertical)
                            .constraints([Constraint::Length(3), Constraint::Min(3)].as_ref(),).split(editchunk[0]);
                        let chunk2 = Layout::default().direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)].as_ref(),).split(editchunk[1]);
                        let chunk3 = Layout::default().direction(Direction::Horizontal)
                            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),).split(editchunk[2]);
                        let (titleinput, descinput, priorityinput, statusinput, notesinput) = render_edit_form(app);
                        rect.render_widget(titleinput, chunk1[0]);
                        rect.render_widget(descinput, chunk1[1]);
                        rect.render_stateful_widget(priorityinput, chunk2[0], &mut app.edit_priority_state);
                        rect.render_stateful_widget(statusinput, chunk2[1], &mut app.edit_status_state);
                        rect.render_stateful_widget(notesinput, chunk3[0], &mut app.edit_note_state);
                    }
                    MenuItem::NoteForm => {
                        let chunks = Layout::default().direction(Direction::Vertical)
                            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),).split(chunks[1]);
                        let (input, output) = render_notes_form(app);
                        
                        rect.render_widget(input, chunks[0]);
                        rect.render_widget(output, chunks[1]);
                    
                        //rect.set_cursor(chunks[0].x + app.input.width() as u16 + add_y, chunks[0].y + add_y,)
                        
                    },
                    MenuItem::ConfirmForm => todo!(),
                    MenuItem::Help => {
                        let text = render_help_form(app);
                        rect.render_widget(text, chunks[1]);
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
                            KeyCode::Char('h') => {
                                app.active_menu_item = MenuItem::Help;
                            }
                            KeyCode::Char('a') => {
                                    init_add_ticket(app).expect("Cannot add ticket");
                                    app.edit_focus = EditItem::Title;
                        }
                            KeyCode::Char('e') => {
                                edit_ticket_at_index(app).expect("Cannot edit ticket");
                                app.edit_focus = EditItem::Title;
                            }
                            KeyCode::Char('k') => {
                                if event.modifiers == KeyModifiers::CONTROL {
                                match app.ticket_view_mode {
                                    TicketViewMode::Open => {
                                        //delete_ticket_at_index(&mut app).expect("Cannot delete ticket");
                                    }
                                    TicketViewMode::Closed => {
                                        remove_ticket_at_index(app).expect("Cannot remove ticket");
                                    }
                                }
                            }
                            }
                            KeyCode::PageDown => {
                                app.scroll += 1;   
                            }
                            KeyCode::PageUp => {
                                if app.scroll > 0 {
                                    app.scroll -= 1;
                                } 
                            }
                            KeyCode::Char('v') => {
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
                                    app.scroll = 0;
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
                                    app.scroll = 0;
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
                            KeyCode::F(1) => {
                                app.sort_by = SortBy::ID;
                                sort(app);
                            }
                            KeyCode::F(2) => {
                                app.sort_by = SortBy::Title;
                                sort(app);
                            }
                            KeyCode::F(3) => {
                                app.sort_by = SortBy::Priority;
                                sort(app);
                            }
                            KeyCode::F(4) => {
                                app.sort_by = SortBy::Updated;
                                sort(app);
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
                            match app.edit_focus {
                                EditItem::Title => app.edit_focus = EditItem::Description,
                                EditItem::Description => app.edit_focus = EditItem::Priority,
                                EditItem::Priority => app.edit_focus = EditItem::Status,
                                EditItem::Status => {
                                    add_ticket(app).unwrap();
                                    app.active_menu_item = MenuItem::Tickets;
                                },
                                EditItem::Notes => todo!(),
                        }
                    }
                        KeyCode::F(5) => {
                            //Save ticket
                            add_ticket(app).unwrap();
                            app.active_menu_item = MenuItem::Tickets;
                        }
                        KeyCode::Tab => {
                            //Set focus to next EditItem
                            app.edit_focus = match app.edit_focus {
                                EditItem::Title => EditItem::Description,
                                EditItem::Description => EditItem::Priority,
                                EditItem::Priority => EditItem::Status,
                                EditItem::Status => EditItem::Title,
                                EditItem::Notes => todo!(),
                            };
                       }
                        //Shift Tab
                        KeyCode::BackTab => {
                            //Set focus to previous EditItem
                            app.edit_focus = match app.edit_focus {
                                EditItem::Title => EditItem::Status,
                                EditItem::Description => EditItem::Title,
                                EditItem::Priority => EditItem::Description,
                                EditItem::Status => EditItem::Priority,
                                EditItem::Notes => todo!(),
                            };
                        }
                        KeyCode::Char(c) => {
                            match app.edit_focus {
                                EditItem::Title => {
                                    app.edit_ticket.title.push(c);
                                }
                                EditItem::Description => {
                                    app.edit_ticket.description.push(c);
                                }
                                EditItem::Priority => {}
                                EditItem::Status => {}
                                EditItem::Notes => {}
                            }
                        }
                        KeyCode::Backspace => {
                            match app.edit_focus {
                                EditItem::Title => {
                                    app.edit_ticket.title.pop();
                                }
                                EditItem::Description => {
                                    app.edit_ticket.description.pop();
                                }
                                EditItem::Priority => {}
                                EditItem::Status => {}
                                EditItem::Notes => {}
                            }
                        }
                        KeyCode::Up => {
                            match app.edit_focus {
                                EditItem::Title => {}
                                EditItem::Description => {}
                                EditItem::Priority => {
                                    if app.edit_priority_state.selected() == Some(0) {
                                        app.edit_priority_state.select(Some(2));
                                    } else {
                                        app.edit_priority_state.select(Some(app.edit_priority_state.selected().unwrap() - 1));
                                    }
                                }
                                EditItem::Status => {
                                    if app.edit_status_state.selected() == Some(0) {
                                        app.edit_status_state.select(Some(1));
                                    } else {
                                        app.edit_status_state.select(Some(app.edit_status_state.selected().unwrap() - 1));
                                    }
                                }
                                EditItem::Notes => {}
                            }
                        }
                        KeyCode::Down => {
                            match app.edit_focus {
                                EditItem::Title => {}
                                EditItem::Description => {}
                                EditItem::Priority => {
                                    if app.edit_priority_state.selected() == Some(2) {
                                        app.edit_priority_state.select(Some(0));
                                    } else {
                                        app.edit_priority_state.select(Some(app.edit_priority_state.selected().unwrap() + 1));
                                    }

                                }
                                EditItem::Status => {
                                    if app.edit_status_state.selected() == Some(1) {
                                        app.edit_status_state.select(Some(0));
                                    } else {
                                        app.edit_status_state.select(Some(app.edit_status_state.selected().unwrap() + 1));
                                    }
                                }
                                EditItem::Notes => {}
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
                MenuItem::Help =>  match rx.recv()? {           
                    Event::Input(event) => match event.code {
                    KeyCode::Esc => {
                        app.active_menu_item = MenuItem::Tickets;
                    }
                    _ => {}
                },
                    Event::Tick => {}
                },
            }
            
        }
        Ok(())  
}