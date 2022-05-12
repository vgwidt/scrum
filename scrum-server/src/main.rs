use std::fs;

use futures::prelude::*;
use serde_json::{Value};
use serde_json::json;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;
use tokio_serde::formats::*;
use tokio_util::codec::{FramedRead, LengthDelimitedCodec};
use scrum_lib::*;
use chrono::Utc;
use tokio_serde_json::{ReadJson, WriteJson};


const DB_PATH: &str = "ticketdb.json";

#[tokio::main]
pub async fn main() {
    // Bind a server socket
    let listener = TcpListener::bind("127.0.0.1:17653").await.unwrap();

    println!("listening on {:?}", listener.local_addr());

    loop {
        let (socket, _) = listener.accept().await.unwrap();

        // Delimit frames using a length header
        let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());

        // Deserialize frames
        let mut deserialized = tokio_serde::SymmetricallyFramed::new(
            length_delimited,
            SymmetricalJson::<Value>::default(),
        );

        // Spawn a task that prints all received messages to STDOUT
        tokio::spawn(async move {
            while let Some(msg) = tokio_stream::StreamExt::try_next(&mut deserialized).await.unwrap() {
                //convert message to Request


                //println!("GOT: {:?}", msg);

                let request:Request = serde_json::from_value(msg).unwrap();

                println!("JSON: {:?}", request);
               match request.action {
                TicketAction::Create => { add_ticket_to_db(request.ticket);               
            
                }
                TicketAction::Update => todo!(),
                TicketAction::Delete => todo!(),
                TicketAction::UpdateDb => {
                    //let db_content = fs::read_to_string(DB_PATH).unwrap();
                    //let mut parsed: Vec<Tickets> = serde_json::from_str(&db_content).unwrap();
                    //return struct to TCP client


                },
                }
            }
        }
    );
    }
}

fn add_ticket_to_db(ticket: Tickets){
    let db_content = fs::read_to_string(DB_PATH).unwrap();
    let mut parsed: Vec<Tickets> = serde_json::from_str(&db_content).unwrap();
    let mut max_id = 0;
    for ticket in parsed.iter() {
        if ticket.id > max_id {
            max_id = ticket.id;
        }
    }

    parsed.push(ticket);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed).unwrap()).unwrap();
   // Ok(parsed)
}







// fn create_db() -> Result<(), Error> {
//     //create sample ticket
//     let sample_ticket = Tickets {
//         id: 0,
//         title: "Zabbix Setup".to_owned(),
//         description: "Setup Zabbix".to_owned(),
//         status: TicketStatus::Open.to_owned(),
//         priority: "Low".to_owned(),
//         created_at: Utc::now(),
//         updated_at: Utc::now(),
//     };
//     //Add sample ticket to vector
//     //create empty file
//     File::create(DB_PATH)?;
//     fs::write(DB_PATH, &serde_json::to_vec(&sample_ticket)?)?;
//     Ok(())
// }