//! Module containing the handlers of the applications API endpoints
use super::*;
use futures::{FutureExt, StreamExt};
use google_jwt_verify::*;
use warp::Reply;

use crate::{Database, Token};

/// Handler for endpoint /usercert/:googleuserid.
pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    println!("{}", googleuserid);
    let reply = db.get_certs(googleuserid).unwrap();

    serde_json::to_string(&reply).unwrap()
}

/// Handler for endpoint /userdata/:googleuserid.
pub fn userdata_handler(body: serde_json::Value, db: Database) -> impl Reply {
    let googleuserid = body["googleuserid"].to_string();
    //deserilze googleuserid
    let googleuserid = serde_json::from_str(&googleuserid).unwrap();
    let reply = db.get_user_data(googleuserid).unwrap();
    let ser_reply = serde_json::to_string(&reply).unwrap();

    Ok(warp::reply::json(&ser_reply))
}

// TODO: finish google auth implementation
pub fn post_token_handler(token: serde_json::Value) -> impl Reply {
    let client_id = "821695412865-f6sndakvma08hqnjkqrjpmm7b2da2hmu.apps.googleusercontent.com";
    let client = Client::new(&client_id);

    let id_token = client.verify_id_token(token["idToken"].as_str().unwrap());

    match id_token {
        Ok(_) => {
            let user_id = id_token.unwrap().get_claims().get_subject();
            String::from(user_id)
        }
        Err(_) => String::from("Err"),
    }
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
}
