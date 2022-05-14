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
use crate::Tickets;
use crate::app::*;

pub fn render_edit_form<'a>(ticket_list_state: &TableState, app: &'a AppState) -> Paragraph<'a> {
    //Get ticket at selected index
    let selected = ticket_list_state.selected().unwrap();

    let ticket = &app.open_tickets[selected];


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