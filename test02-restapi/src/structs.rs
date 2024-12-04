use crate::Message;
use actix::Addr;
use actix_web::web;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct BroadcastMessage(pub String);

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: u16,
    pub message: String,
}

pub struct WsSession {
    pub hb: Instant,
    pub books: web::Data<Mutex<Vec<Book>>>,
    pub sessions: web::Data<Arc<Mutex<Vec<Addr<WsSession>>>>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Book {
    #[serde(skip_deserializing)]
    pub id: Uuid,
    pub title: String,
    pub author: String,
    pub year: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "action")]
pub enum Command {
    GetBooks,
    GetBook { id: Uuid },
    AddBook { book: Book },
    UpdateBook { id: Uuid, book: Book },
    DeleteBook { id: Uuid },
}
