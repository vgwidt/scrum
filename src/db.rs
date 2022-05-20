use std::{fs::{self, File}, path::Path};
use scrum_lib::*;
use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

const DB_PATH: &str = "ticketdb.json";

pub fn write_changes(tickets: &Vec<Tickets>) -> Result<(), Error> {
    fs::write(DB_PATH, &serde_json::to_vec(&tickets)?)?;
    Ok(())
}

pub fn read_db() -> Result<Vec<Tickets>, Error> {

    if !Path::new(DB_PATH).exists() {
        //create the file
        File::create(DB_PATH)?;
        //write the default ticket
        let default_ticket = Tickets::default();
        write_changes(&vec![default_ticket])?;
    }

    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Tickets> = serde_json::from_str(&db_content)?; 
    Ok(parsed)
}

pub fn get_open_tickets() -> Vec<Tickets> {
    let tickets = read_db().unwrap();
    let mut open_tickets = Vec::new();
    for ticket in tickets {
        if ticket.status.to_string() == "Open" {
            open_tickets.push(ticket.clone());
        }
    }
    open_tickets
}

pub fn get_closed_tickets() -> Vec<Tickets> {
    let tickets = read_db().unwrap();
    let mut closed_tickets = Vec::new();
    for ticket in tickets {
        if ticket.status.to_string() == "Closed" {
            closed_tickets.push(ticket.clone());
        }
    }
    closed_tickets
}