//! Module containing the handlers of the applications API endpoints
use futures::{FutureExt, StreamExt};
use warp::Reply;

use crate::{Database, Token};

/// Handler for endpoint /usercert/:googleuserid.
pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_certs(googleuserid).unwrap();

    serde_json::to_string(&reply).unwrap()
}

/// Handler for endpoint /userdata/:googleuserid.
pub fn userdata_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_user_data(googleuserid).unwrap();
    serde_json::to_string(&reply).unwrap()
}

// TODO: finish google auth implementation
pub fn post_token_handler(token: Token) -> impl Reply {
    let mut token = token;
    token.token = String::from("hej");
    Ok(warp::reply::json(&token))
}

// TODO: finish websocket implementation
pub fn websocket_handler(ws: warp::ws::Ws) -> impl Reply {
    // And then our closure will be called when it completes...
    ws.on_upgrade(|websocket| {
        // Just echo all messages back...
        let (tx, rx) = websocket.split();
        rx.forward(tx).map(|result| {
            if let Err(e) = result {
                eprintln!("websocket error: {:?}", e);
            }
        })
    })
}