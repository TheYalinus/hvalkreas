use std::fs;

use axum::{
    Router,
    extract::{WebSocketUpgrade, ws::WebSocket},
    response::{Form, Html, Response},
    routing::get,
};
use serde::Deserialize;
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/test", axum::routing::get(test))
        .route("/newuser", get(show_form).post(accept_form));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
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
async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}
async fn handle_socket(mut socket: WebSocket) {
    unimplemented!()
}
