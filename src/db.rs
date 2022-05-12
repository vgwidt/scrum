use std::{fs::{self, File}, process::exit, env};
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
    //if DB exists, read it, otherwise create it
    // if !DB_PATH.exists() {
    //     let _ = create_db();
    // }
    
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Tickets> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}