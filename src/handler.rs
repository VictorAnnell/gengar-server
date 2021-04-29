//! Module containing the handlers of the applications API endpoints
use futures::{FutureExt, StreamExt};
use warp::Reply;
use serde_json::json;
use super::*;


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

pub fn get_qr_handler(body: serde_json::Value, qr_codes: QrCodes) -> impl Reply {
    let googleuserid = body["googleuserid"].to_string();
    let qr = generate_qr_string();
    qr_codes.clone().insert(googleuserid, qr.rand_string.clone());
    let ser_qr = serde_json::to_string(&(qr)).unwrap();

    Ok(warp::reply::json(&ser_qr))
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usercert_handler_test() {
        let db = Database::new();

        let result = usercert_handler(db.clone(), "234385785823438578589".to_string());
        assert_eq!(result, "[\"cert1\",\"cert2\"]");

        let result = usercert_handler(db, "fakeuser".to_string());
        assert_eq!(result, "[]");
    }

    #[test]
    fn userdata_handler_test() {
        let db = Database::new();

        let result = userdata_handler(db.clone(), "234385785823438578589".to_string());
        assert_eq!(result, "{\"certificates\":[{\"name\":\"cert1\",\"registerdate\":\"1988-12-30\",\"expirationdate\":\"2022-03-30\"},{\"name\":\"cert2\",\"registerdate\":\"2015-02-19\",\"expirationdate\":\"2021-06-02\"}]}");

        let result = userdata_handler(db, "fakeuser".to_string());
        assert_eq!(result, "{\"certificates\":[]}");
    }

    #[test]
    fn get_qr_handler_test() {
        let body = json!({"googleuserid": "234385785823438578589"});
        let qr_codes: QrCodes = HashMap::new();
        let result = get_qr_handler(body, qr_codes).into_response();
        println!("{:?}", result);
        //Work in progress
    }
}
