use super::router;
use crate::structs::{BroadcastMessage, WsSession};
use actix::{Actor, StreamHandler};
use actix::{AsyncContext, Handler};
use actix_web_actors::ws;
use std::time::Instant;

impl WsSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.ping(&[]);
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        let sessions = self.sessions.clone();
        let addr = ctx.address();
        actix::spawn(async move {
            let mut sessions = sessions.lock().await;
            sessions.push(addr);
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let sessions = self.sessions.clone();
        let current_addr = ctx.address();
        actix::spawn(async move {
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
            Ok(ws::Message::Text(text)) => router::obtain(
                serde_json::from_slice(text.as_ref()),
                ctx.address(),
                self.books.clone(),
                self.sessions.clone(),
            ),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}
impl Handler<BroadcastMessage> for WsSession {
    type Result = ();
    fn handle(&mut self, msg: BroadcastMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}
