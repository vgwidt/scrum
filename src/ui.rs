use std::vec;

use chrono::{Utc, Local};
use scrum_lib::*;
use tui::{
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text, self},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs, TableState, Wrap,
    }, layout::{Constraint, Alignment},
};
use crate::app::*;


pub fn render_tickets<'a>(app: &AppState) -> (Table<'a>, Paragraph<'a>) {
 
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
    //Gets selected ticket by index, but requires nifty alignment
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
        .block(Block::default().borders(Borders::ALL).title(" Tickets"))
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

    let mut text = vec![
        Spans::from(vec![
            Span::styled("ID: ", Style::default().fg(Color::Yellow)),
            Span::raw(selected_ticket.id.to_string()),
            Span::styled(" | Status: ", Style::default().fg(Color::Yellow)),
            Span::raw(selected_ticket.status.to_string().to_owned()),
            Span::styled(" | Priority: ", Style::default().fg(Color::Yellow)),
            Span::raw(selected_ticket.priority.to_string().to_owned()),
            Span::styled(" | Created: ", Style::default().fg(Color::Yellow)),
            Span::raw(selected_ticket.created_at.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()),
            Span::styled(" | Updated: ", Style::default().fg(Color::Yellow)),
            Span::raw(selected_ticket.updated_at.with_timezone(&Local).format("%Y-%m-%d %H:%M").to_string()),
        ]),
        Spans::from(vec![Span::raw("\n")]),
        Spans::from(vec![
        Span::styled("Title: ", Style::default().fg(Color::Yellow)),
        Span::raw(selected_ticket.title.clone()),
    ]),
    Spans::from(vec![Span::raw("\n")]),
    Spans::from(vec![
        Span::styled("Description: ", Style::default().fg(Color::Yellow)),
        Span::raw(selected_ticket.description.clone()),
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
                .title(" Ticket Detail")
                .border_type(BorderType::Plain),
        ).wrap(Wrap { trim: true });
    
    (list, ticket_detail)
}

pub fn render_edit_form<'a>(app: &AppState) -> (Paragraph<'a>, Paragraph<'a>, List<'a>) {
    

    let input1 = Paragraph::new(app.edit_ticket.title.clone())
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL).title(app.prompt.clone())).wrap(Wrap { trim: true });

    let input2 = Paragraph::new(app.edit_ticket.description.clone())
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL).title("Description")).wrap(Wrap { trim: true });

    //Create ListItem for each priority
    let priorityrows = vec![
        ListItem::new(Span::styled("High", Style::default().fg(Color::Yellow))),
        ListItem::new(Span::styled("Medium", Style::default().fg(Color::Yellow))),
        ListItem::new(Span::styled("Low", Style::default().fg(Color::Yellow))),
    ];

    let input3 = List::new(priorityrows)
    .block(Block::default().borders(Borders::ALL).title("Priority"))
    .style(Style::default().fg(Color::White))
    .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

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
    
    //create ListItem for each note in selected ticket
    //Might want to rethink how we determine selected ticket at this stage?  How do we pass selected ticket to here?  Perhaps we hold it in AppState?
    // let mut notespan = Vec::new();
    // if app.selected_ticket.notes.is_some() {
    //     let notes = app.selected_ticket.notes.clone().unwrap();
    //     for note in notes {
    //         notespan.push(
    //             Spans::from(vec![
    //             Span::raw(note.updated_at.format("%Y-%m-%d %H:%M").to_string()),
    //             Span::styled(" Update: ", Style::default().fg(Color::Yellow)),
    //             Span::raw(note.text.clone()),
    //         ]));
    //     }
    // }


(input1, input2, input3)
 
}

pub fn render_notes_form<'a>(app: &'a mut AppState) -> (Paragraph<'a>, List<'a>) {
    
    let input1 = Paragraph::new(app.input.as_ref())
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL).title(app.prompt.clone())).wrap(Wrap { trim: true });

    let input2 = Paragraph::new(app.input.as_ref())
    .style(Style::default().fg(Color::Yellow))
    .block(Block::default().borders(Borders::ALL).title("Input")).wrap(Wrap { trim: true });

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