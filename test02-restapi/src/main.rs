#![warn(clippy::all, clippy::pedantic)]
use actix::prelude::*;
use actix::{spawn, Actor, Addr, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use uuid::Uuid;
mod structs;
use structs::{ApiResponse, Book, BroadcastMessage, Command, WsSession};

impl Handler<BroadcastMessage> for WsSession {
    type Result = ();
    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl WsSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.ping(&[]);
    }
    async fn broadcast_books(&self) {
        let books = self.books.lock().await;
        let books_json = serde_json::to_string(&*books).unwrap();
        let sessions = self.sessions.lock().await;
        for session in sessions.iter() {
            session.do_send(BroadcastMessage(books_json.clone()));
        }
    }
    fn send_response(&self, ctx: &mut ws::WebsocketContext<Self>, res_body: ApiResponse) {
        let json = serde_json::to_string(&res_body).unwrap();
        ctx.text(json);
    }
}
impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);

        // Асинхронно добавляем сессию
        let sessions = self.sessions.clone();
        let addr = ctx.address();
        spawn(async move {
            let mut sessions = sessions.lock().await;
            sessions.push(addr);
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let sessions = self.sessions.clone();
        let current_addr = ctx.address();
        spawn(async move {
            let mut sessions = sessions.lock().await;
            sessions.retain(|s| s != &current_addr);
        });
    }
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
                        let books = self.books.clone();
                        let addr = ctx.address();
                        actix::spawn(async move {
                            let books = books.lock().await;
                            let books_json = serde_json::to_string(&*books).unwrap();
                            addr.do_send(BroadcastMessage(books_json.clone()));
                        });
                    }
                    // broadcast
                    Ok(Command::AddBook { book }) => {
                        let books = self.books.clone();
                        let addr = ctx.address();
                        actix::spawn(async move {
                            let mut books = books.lock().await;
                            books.push(Book {
                                id: Uuid::new_v4(),
                                title: book.title,
                                author: book.author,
                                year: book.year,
                            });
                            self.broadcast_books();
                            self.send_response(
                                addr,
                                ApiResponse {
                                    status: 201,
                                    message: "The book was added successfully",
                                },
                            )
                        });
                    }
                    Ok(Command::GetBook { id }) => {
                        let books = self.books.clone();
                        let addr = ctx.address();
                        actix::spawn(async move {
                            let books = self.books.lock().await;
                            if let Some(book) = books.iter().find(|b| b.id == id) {
                                let book_json = serde_json::to_string(&book).unwrap();
                                ctx.text(book_json);
                            } else {
                                self.send_response(
                                    ctx,
                                    ApiResponse {
                                        status: 204,
                                        message: "The book was not found",
                                    },
                                )
                            }
                        });
                    }
                    // broadcast
                    Ok(Command::UpdateBook { id, book }) => {
                        let books = self.books.clone();
                        let addr = ctx.address();
                        actix::spawn(async move {
                            let mut books = self.books.lock().await;
                            if let Some(existing_book) = books.iter_mut().find(|b| b.id == id) {
                                existing_book.title = book.title;
                                existing_book.author = book.author;
                                existing_book.year = book.year;
                                self.send_response(
                                    ctx,
                                    ApiResponse {
                                        status: 200,
                                        message: "The book was updated",
                                    },
                                );
                                drop(books);
                                self.broadcast_books();
                            } else {
                                self.send_response(
                                    ctx,
                                    ApiResponse {
                                        status: 204,
                                        message: "The book was not found",
                                    },
                                )
                            }
                        });
                    }
                    // broadcast
                    Ok(Command::DeleteBook { id }) => {
                        let books = self.books.clone();
                        let addr = ctx.address();
                        actix::spawn(async move {
                            let mut books = self.books.lock().await;
                            if let Some(_) = books.iter().find(|b| b.id == id) {
                                books.retain(|b| b.id != id);
                                self.send_response(
                                    ctx,
                                    ApiResponse {
                                        status: 200,
                                        message: "The book was successfully deleted",
                                    },
                                );
                                drop(books);
                                self.broadcast_books();
                            } else {
                                self.send_response(
                                    ctx,
                                    ApiResponse {
                                        status: 204,
                                        message: "The book was not found",
                                    },
                                )
                            }
                        });
                    }
                    Err(_err) => self.send_response(
                        ctx,
                        ApiResponse {
                            status: 405,
                            message: "Incorrect Request",
                        },
                    ),
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
