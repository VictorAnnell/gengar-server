use super::*;
use futures::{FutureExt, StreamExt};

pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_certs(googleuserid).unwrap();

    serde_json::to_string(&reply).unwrap()
}

pub fn userdata_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_user_data(googleuserid).unwrap();
    serde_json::to_string(&reply).unwrap()
}

pub fn post_token_handler(token: Token) -> impl Reply {
    let mut token = token;
    token.token = String::from("hej");
    Ok(warp::reply::json(&token))
}

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