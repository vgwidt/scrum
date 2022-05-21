//For functions related to handling tickets

use chrono::Utc;
use scrum_lib::*;
use crate::{db::*, Event};
use crate::ui::*;
use crate::app::*;

pub fn init_add_ticket(app: &mut AppState) -> Result<(), Error> {

    app.edit_ticket.id = -7;
    app.edit_ticket.status = TicketStatus::Open;
    app.edit_ticket.title = String::new();
    app.edit_ticket.description = String::new();
    app.edit_ticket.priority = "Low".to_string();

    app.edit_priority_state.select(Some(0)); //Can be fixed to match set priority above
    app.edit_status_state.select(Some(0));

    app.prompt = "Enter Title".to_string();
    app.active_menu_item = MenuItem::EditForm;
    Ok(())
}

pub fn add_ticket (app: &mut AppState) -> Result<(), Error> {
    if let Some(selected) = app.ticket_list_state.selected() {
    //if new

        //This can be fixed by using ticket priority enum with impl fn
        if app.edit_priority_state.selected() == Some(0) {
            app.edit_ticket.priority = "High".to_string();
        } else if app.edit_priority_state.selected() == Some(1) {
            app.edit_ticket.priority = "Medium".to_string();
        } else if app.edit_priority_state.selected() == Some(2) {
            app.edit_ticket.priority = "Low".to_string();
        }

        if app.edit_status_state.selected() == Some(0) {
            app.edit_ticket.status = TicketStatus::Open;
        } else if app.edit_status_state.selected() == Some(1) {
            app.edit_ticket.status = TicketStatus::Closed;
        }

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
        app.edit_ticket.created_at = Utc::now();
        app.edit_ticket.updated_at = Utc::now();


        app.open_tickets.push(app.edit_ticket.clone());
    } else {
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

        app.edit_priority_state.select(
            if app.edit_ticket.priority == "High" {Some(0)} else if app.edit_ticket.priority == "Medium" {Some(1)} else {Some(2)}  
          );
          app.edit_status_state.select(
            if app.edit_ticket.status.to_string() == "Open" {Some(0)} else {Some(1)}  
          );
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

pub fn remove_ticket_at_index(app: &mut AppState) -> Result<(), Error> {
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

pub fn update_selected_ticket(app: &mut AppState, selected: usize) {
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

pub fn update_db(app: &AppState) {
    //Concatenate open and closed tickets
    let mut all_tickets = app.open_tickets.clone();
    all_tickets.append(&mut app.closed_tickets.clone());
    write_changes(&all_tickets).unwrap();
}

pub fn toggle_ticket_status(app: &mut AppState) -> Result<(), Error> {
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