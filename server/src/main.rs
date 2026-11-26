use anyhow::Error;
use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    response::{Form, Html, Response},
    routing::{any, get},
};
use futures_util::SinkExt;
use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::{fs, os::linux::raw::stat};
use tokio::{
    select,
    sync::broadcast::{self, Sender},
};
#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel(16);
    let app = Router::new()
        .route("/test", axum::routing::get(test))
        .route("/newuser", get(show_form).post(accept_form))
        .route("/ws", any(handler))
        .with_state(AppState { tx: tx.clone() });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
    tx.send(String::from("hallo")).unwrap();
}
async fn show_form() -> Html<String> {
    match fs::read_to_string("assets/newuserform.html") {
        Ok(html) => Html(html),
        Err(_) => Html(String::from(
            r#"
        <!doctype html>
        <html lang="en">
        <head>
          <meta charset="utf-8">
          <meta name="viewport" content="width=device-width, initial-scale=1">
          <title>Sign in</title>
        </head>
        <body>
          <h> ONLINE DND APP </h>
          <form action="/newuser" method="post">
            <label for="username">Username</label>
            <input
              id="username"
              name="username"
              type="text"
              autocomplete="username"
              required
            >

            <label for="password">Password</label>
            <input
              id="password"
              name="password"
              type="password"
              autocomplete="current-password"
              required
            >

            <button type="submit">Sign in</button>
          </form>
        </body>
        </html>
    "#,
        )),
    }
}
#[derive(Deserialize, Debug)]
struct Input {
    username: String,
    password: String,
}
async fn accept_form(Form(input): Form<Input>) -> Html<String> {
    Html(format!("name='{}'\n", input.username))
}
async fn test() -> String {
    String::from("Hello World")
}
#[derive(Clone)]
struct AppState {
    tx: Sender<String>,
}
async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}
async fn handle_socket(socket: WebSocket, state: AppState) {
    let mut rx = state.tx.subscribe();
    let (mut sender, mut receiver) = socket.split();
    loop {
        let mut awaitmsg = async || {
            if let Some(Ok(msg)) = receiver.next().await {
                if let Ok(text) = msg.to_text() {
                    if let Err(e) = state.tx.send(text.to_string()) {
                        println!("Cannot send message to broadcast channel {e}")
                    }
                }
            } else {
                println!("Client disconnected");
                return Err(());
            }
            Ok(())
        };
        let mut bc = {
            async || {
                let bcmsg = rx.recv().await.unwrap();
                match sender
                    .send(Message::from(format!(
                        "Message from another client {}",
                        bcmsg
                    )))
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        println!("{e}");
                    }
                };
            }
        };
        select! {
            val1 =  awaitmsg() =>{ println!("Got message from client");}
            val2= bc() =>{println!("got message from broadcast channel");}
        };
    }
}
