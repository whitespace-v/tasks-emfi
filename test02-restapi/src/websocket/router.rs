use std::sync::Arc;

use crate::{
    broadcast, response,
    structs::{ApiResponse, Book, BroadcastMessage, Command, WsSession},
};
use actix::Addr;
use actix_web::web;
use tokio::sync::Mutex;
use uuid::Uuid;

pub fn obtain(
    cmd: Result<Command, serde_json::Error>,
    addr: Addr<WsSession>,
    books: web::Data<Mutex<Vec<Book>>>,
    sessions: actix_web::web::Data<Arc<tokio::sync::Mutex<Vec<Addr<WsSession>>>>>,
) {
    match cmd {
        Ok(Command::GetBooks) => {
            actix::spawn(async move {
                addr.do_send(BroadcastMessage(
                    serde_json::to_string(&*books.lock().await).unwrap(),
                ));
            });
        }
        Ok(Command::AddBook { book }) => {
            actix::spawn(async move {
                let mut books = books.lock().await;
                books.push(Book {
                    id: Uuid::new_v4(),
                    title: book.title,
                    author: book.author,
                    year: book.year,
                });
                broadcast(books, sessions).await;
                response(
                    &addr,
                    ApiResponse {
                        status: 201,
                        message: "The book was added successfully".to_owned(),
                    },
                )
                .await
            });
        }
        Ok(Command::GetBook { id }) => {
            actix::spawn(async move {
                if let Some(book) = books.lock().await.iter().find(|b| b.id == id) {
                    addr.do_send(BroadcastMessage(serde_json::to_string(&book).unwrap()));
                } else {
                    response(
                        &addr,
                        ApiResponse {
                            status: 204,
                            message: "The book was not found".to_owned(),
                        },
                    )
                    .await
                }
            });
        }
        Ok(Command::UpdateBook { id, book }) => {
            actix::spawn(async move {
                let mut books = books.lock().await;
                if let Some(existing_book) = books.iter_mut().find(|b| b.id == id) {
                    existing_book.title = book.title;
                    existing_book.author = book.author;
                    existing_book.year = book.year;
                    broadcast(books, sessions).await;
                    response(
                        &addr,
                        ApiResponse {
                            status: 200,
                            message: "The book was updated".to_owned(),
                        },
                    )
                    .await
                } else {
                    response(
                        &addr,
                        ApiResponse {
                            status: 204,
                            message: "The book was not found".to_owned(),
                        },
                    )
                    .await
                }
            });
        }
        Ok(Command::DeleteBook { id }) => {
            actix::spawn(async move {
                let mut books = books.lock().await;
                if let Some(_) = books.iter().find(|b| b.id == id) {
                    books.retain(|b| b.id != id);
                    broadcast(books, sessions).await;
                    response(
                        &addr,
                        ApiResponse {
                            status: 200,
                            message: "The book was successfully deleted".to_owned(),
                        },
                    )
                    .await;
                } else {
                    response(
                        &addr,
                        ApiResponse {
                            status: 204,
                            message: "The book was not found".to_owned(),
                        },
                    )
                    .await
                }
            });
        }
        Err(_) => {
            actix::spawn(async move {
                response(
                    &addr,
                    ApiResponse {
                        status: 405,
                        message: "Incorrect Request".to_owned(),
                    },
                )
                .await
            });
        }
    }
}
