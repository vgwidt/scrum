use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tickets{
    pub id: i32,
    pub title: String,
    pub description: String,
    pub status: TicketStatus,
    pub priority: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Tickets{
    pub fn new(id: i32, title: String, description: String, status: TicketStatus, priority: String, created_at: DateTime<Utc>, updated_at: DateTime<Utc>) -> Tickets{
        Tickets{
            id,
            title,
            description,
            status,
            priority,
            created_at,
            updated_at,
        }
    }
    pub fn next_id(&self) -> i32{
        self.id + 1
    }
    pub fn prev_id(&self) -> i32{
        self.id - 1
    }
    pub fn default() -> Tickets{
        Tickets{
            id: 0,
            title: String::from(""),
            description: String::from(""),
            status: TicketStatus::Open,
            priority: String::from(""),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TicketStatus {
    Open,
    Closed,    
}

impl TicketStatus {
    pub fn to_string(&self) -> &str {
        match self {
            TicketStatus::Open => "Open",
            TicketStatus::Closed => "Closed",
        }   
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TicketPriority {
    Low,
    Medium,
    High,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Request{
    pub action: TicketAction,
    pub ticket: Tickets,
}

//Request into <T> value
impl From<Request> for Value {
    fn from(request: Request) -> Self {
        json!({
            "action": request.action.to_string(),
            "ticket": request.ticket,
        })
    }
}




#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TicketAction {
    Create,
    Update,
    Delete,
    UpdateDb,
}

impl TicketAction {
    pub fn to_string(&self) -> &str {
        match self {
            TicketAction::Create => "Create",
            TicketAction::Update => "Update",
            TicketAction::Delete => "Delete",
            TicketAction::UpdateDb => "UpdateDb",
        }   
    }
}