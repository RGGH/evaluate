// src/api/handlers/ws.rs
use actix::{Actor, StreamHandler, Handler, Message, Addr, AsyncContext};
use actix_web::{web, HttpRequest, HttpResponse, Error};
use actix_web_actors::ws;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Message, Clone, Serialize)]
#[rtype(result = "()")]
pub struct EvalUpdate {
    pub id: String,
    pub status: String,
    pub model: Option<String>,
    pub verdict: Option<String>,
    pub latency_ms: Option<u64>,
}

#[derive(Clone)]
pub struct WsBroker {
    clients: Arc<RwLock<Vec<Addr<WsConnection>>>>,
}

impl WsBroker {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn register(&self, addr: Addr<WsConnection>) {
        let mut clients = self.clients.write().await;
        clients.push(addr);
    }

    pub async fn unregister(&self, addr: &Addr<WsConnection>) {
        let mut clients = self.clients.write().await;
        clients.retain(|c| c != addr);
    }

    pub async fn broadcast(&self, msg: EvalUpdate) {
        let clients = self.clients.read().await;
        for client in clients.iter() {
            client.do_send(msg.clone());
        }
    }
}

pub struct WsConnection {
    broker: WsBroker,
}

impl WsConnection {
    pub fn new(broker: WsBroker) -> Self {
        Self { broker }
    }
}

impl Actor for WsConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let broker = self.broker.clone();
        actix::spawn(async move {
            broker.register(addr).await;
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let broker = self.broker.clone();
        actix::spawn(async move {
            broker.unregister(&addr).await;
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsConnection {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Close(reason)) => ctx.close(reason),
            _ => (),
        }
    }
}

impl Handler<EvalUpdate> for WsConnection {
    type Result = ();

    fn handle(&mut self, msg: EvalUpdate, ctx: &mut Self::Context) {
        if let Ok(json) = serde_json::to_string(&msg) {
            ctx.text(json);
        }
    }
}

pub async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    broker: web::Data<WsBroker>,
) -> Result<HttpResponse, Error> {
    let conn = WsConnection::new(broker.get_ref().clone());
    ws::start(conn, &req, stream)
}
