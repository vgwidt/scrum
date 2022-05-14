fn remove_ticket_at_index(app: &mut AppState) -> Result<(), Error> {
    if let Some(selected) = app.ticket_list_state.selected() {

        
        let ticket_list = app.ticket_list.clone();

        //Sort tickets by status
        let mut tickets = Vec::new();
        match app.ticket_view_mode {
            TicketViewMode::Open => {
                for ticket in &ticket_list {
                    if ticket.status.to_string() == "Open" { //FIX THIS!
                        tickets.push(ticket);
                    }
                }
            }
            TicketViewMode::Closed => {
                for ticket in &ticket_list {
                    if ticket.status.to_string() == "Closed" { //FIX THIS!
                        tickets.push(ticket);
                    }
                }
            }
        }
    
        let mut selected_ticket = tickets
            .get(
                app.ticket_list_state
                    .selected()
                    .expect("there is always a selected ticket"),
            )
            .expect("exists")
            .clone();

        //remove ticket from app.ticket_list where id = selected_ticket.id
        let mut parsed: Vec<Tickets> = app.ticket_list.clone();
        let mut index = 0;
        for ticket in parsed.iter() {
            if ticket.id == selected_ticket.id {
                parsed.remove(index);
                break;
            }
            index += 1;
        }


        //update database
        write_changes(&parsed);

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

        //If it was last ticket, set selected to none

    }

    Ok(())
}