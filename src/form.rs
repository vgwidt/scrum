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
    text::{Span, Spans, Text, self},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs, TableState,
    },
    Terminal
};
use crate::Tickets;
use crate::app::*;

pub fn render_edit_form<'a>(app: &'a mut AppState) -> (Paragraph<'a>, List<'a>) {
    
    let input1 = Paragraph::new(app.input.as_ref())
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL).title(app.prompt.clone()));

    let input2 = Paragraph::new(app.input.as_ref())
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL).title("Input"));

    let messages: Vec<ListItem> = app
    .messages
    .iter()
    .enumerate()
    .map(|(i, m)| {
        let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
        ListItem::new(content)
    })
    .collect();
    let messages =
    List::new(messages).block(Block::default().borders(Borders::ALL).title("Messages"));
    

(input1, messages)


        
}