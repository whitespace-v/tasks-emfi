#![warn(clippy::all, clippy::pedantic)]
use actix::prelude::*;
use actix::{Actor, Addr, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
struct BroadcastMessage(pub String);
impl Handler<BroadcastMessage> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
struct WsSession {
    hb: Instant,
    books: web::Data<Mutex<Vec<Book>>>,
    sessions: web::Data<Arc<Mutex<Vec<Addr<WsSession>>>>>,
}
impl WsSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.ping(&[]);
    }
    fn broadcast_books(&self) {
        let books = self.books.lock().unwrap();
        let books_json = serde_json::to_string(&*books).unwrap();
        let sessions = self.sessions.lock().unwrap();
        for session in sessions.iter() {
            session.do_send(BroadcastMessage(books_json.clone()));
        }
    }
}
impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let mut sessions = self.sessions.lock().unwrap();
        sessions.push(ctx.address());
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let current_addr = ctx.address();
        let mut sessions = self.sessions.lock().unwrap();
        sessions.retain(|s| s != &current_addr);
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct Book {
    #[serde(skip_deserializing)]
    id: Uuid,
    title: String,
    author: String,
    year: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "action")]
enum Command {
    GetBooks,
    GetBook { id: Uuid },
    AddBook { book: Book },
    UpdateBook { id: Uuid, book: Book },
    DeleteBook { id: Uuid },
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => self.hb = Instant::now(),
            Ok(ws::Message::Text(text)) => {
                let command = serde_json::from_slice(text.as_ref());
                match command {
                    Ok(Command::GetBooks) => {
                        let books = self.books.lock().unwrap();
                        let books_json = serde_json::to_string(&*books).unwrap();
                        ctx.text(books_json);
                    }
                    // broadcast
                    Ok(Command::AddBook { book }) => {
                        let mut books = self.books.lock().unwrap();
                        books.push(Book {
                            id: Uuid::new_v4(),
                            title: book.title,
                            author: book.author,
                            year: book.year,
                        });
                        drop(books);
                        self.broadcast_books();
                        ctx.text(r#"{"status: 201, "message": "The book was added successfully"}"#);
                    }
                    Ok(Command::GetBook { id }) => {
                        let books = self.books.lock().unwrap();
                        if let Some(book) = books.iter().find(|b| b.id == id) {
                            let book_json = serde_json::to_string(&book).unwrap();
                            ctx.text(book_json);
                        } else {
                            ctx.text(r#"{"status: 204, "message": "The book was not found"}"#)
                        }
                    }
                    // broadcast
                    Ok(Command::UpdateBook { id, book }) => {
                        let mut books = self.books.lock().unwrap();
                        if let Some(existing_book) = books.iter_mut().find(|b| b.id == id) {
                            existing_book.title = book.title;
                            existing_book.author = book.author;
                            existing_book.year = book.year;
                            ctx.text(r#"{"status: 200, "message": "The book was updated"}"#);
                            drop(books);
                            self.broadcast_books();
                        } else {
                            ctx.text(r#"{"status: 204, "message": "The book was not found"}"#)
                        }
                    }
                    // broadcast
                    Ok(Command::DeleteBook { id }) => {
                        let mut books = self.books.lock().unwrap();
                        if let Some(_) = books.iter().find(|b| b.id == id) {
                            books.retain(|b| b.id != id);
                            ctx.text(
                                r#"{"status: 200, "message": "The book was successfully deleted"}"#,
                            );
                            drop(books);
                            self.broadcast_books();
                        } else {
                            ctx.text(r#"{"status: 204, "message": "The book was not found"}"#)
                        }
                    }
                    Err(_err) => {
                        // eprintln!("{err:#?}");
                        ctx.text(r#"{"status: 405, "message": "Incorrect Request"}"#)
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    books: web::Data<Mutex<Vec<Book>>>,
    sessions: web::Data<Arc<Mutex<Vec<Addr<WsSession>>>>>,
) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        WsSession {
            hb: Instant::now(),
            books: books.clone(),
            sessions: sessions.clone(),
        },
        &req,
        stream,
    );
    resp
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let books = web::Data::new(Mutex::new(Vec::<Book>::new()));
    let sessions = web::Data::new(Arc::new(Mutex::new(Vec::<Addr<WsSession>>::new())));

    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    HttpServer::new(move || {
        App::new()
            .app_data(sessions.clone())
            .app_data(books.clone())
            .route("/ws/", web::get().to(ws_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
