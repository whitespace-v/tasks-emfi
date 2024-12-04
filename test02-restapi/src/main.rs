#![warn(clippy::all, clippy::pedantic)]
mod structs;
mod websocket;
use actix::prelude::*;
use actix::Addr;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::sync::Arc;
use std::time::Instant;
use structs::{ApiResponse, Book, BroadcastMessage, WsSession};
use tokio::sync::Mutex;

async fn broadcast(
    books: tokio::sync::MutexGuard<'_, Vec<Book>>,
    sessions: web::Data<Arc<Mutex<Vec<Addr<WsSession>>>>>,
) {
    for session in sessions.lock().await.iter() {
        session.do_send(BroadcastMessage(serde_json::to_string(&*books).unwrap()));
    }
}
async fn response(addr: &Addr<WsSession>, body: ApiResponse) {
    addr.do_send(BroadcastMessage(serde_json::to_string(&body).unwrap()));
}

async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    books: web::Data<Mutex<Vec<Book>>>,
    sessions: web::Data<Arc<Mutex<Vec<Addr<WsSession>>>>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsSession {
            hb: Instant::now(),
            books,
            sessions,
        },
        &req,
        stream,
    )
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(Arc::new(Mutex::new(
                Vec::<Addr<WsSession>>::new(),
            ))))
            .app_data(web::Data::new(Mutex::new(Vec::<Book>::new())))
            .route("/ws/", web::get().to(ws_handler))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
