//! Module containing the handlers of the applications API endpoints
use super::*;
use crate::{Database, GoogleToken};
use futures::{FutureExt, StreamExt};
use google_jwt_verify::*;
use serde_json::json;
use warp::Reply;

/// Handler for endpoint /usercert/:googleuserid.
pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_certs(googleuserid).unwrap();

    serde_json::to_string(&reply).unwrap()
}

/// Handler for endpoint /userdata/:googleuserid.
pub fn userdata_handler(body: serde_json::Value, db: Database) -> impl Reply {
    let googleuserid = body["googleuserid"].to_string();
    //deserilze googleuserid
    let reply = db.get_user_data(googleuserid).unwrap();

    Ok(warp::reply::json(&reply))
}

pub fn post_token_handler(
    token: GoogleToken,
    db: Database,
    client_id: String,
    session_ids: SessionIds,
) -> impl Reply {
    let client = Client::new(&client_id);

    let id_token = client.verify_id_token(&token.id_token).unwrap();
    let googleuserid = id_token.get_claims().get_subject();

    if !db.user_exist(googleuserid.to_string()).unwrap() {
        panic!()
    };

    let sessionid = SessionId::new();

    session_ids.write().unwrap().remove_by_right(&googleuserid);

    session_ids
        .write()
        .unwrap()
        .insert(sessionid.sessionid.clone(), googleuserid);

    Ok(warp::reply::json(&sessionid))
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

pub fn get_qr_handler(body: serde_json::Value, qr_codes: QrCodes, db: Database) -> impl Reply {
    let googleuserid = body["googleuserid"].as_str().unwrap();
    if !db.user_exist(googleuserid.to_string()).unwrap() {
        panic!()
    };
    let qr = QrString::new();

    qr_codes
        .write()
        .unwrap()
        .remove_by_right(&googleuserid.to_string());

    qr_codes
        .write()
        .unwrap()
        .insert(qr.qr_string.clone(), googleuserid.to_string());

    Ok(warp::reply::json(&qr))
}

pub fn verify_cert_handler(body: serde_json::Value, db: Database, qr_codes: QrCodes) -> impl Reply {
    let qrstring: String = body["qrstring"].as_str().unwrap().to_string();
    let req_cert: String = body["certificatestocheck"].as_str().unwrap().to_string();

    let temp = qr_codes.read().unwrap();

    let googleuserid = temp.get_by_left(&qrstring).unwrap().to_string();

    let usr_data: UserData = db.get_user_data(googleuserid).unwrap();

    let mut success: bool = false;
    for i in usr_data.certificates {
        if i.name == req_cert {
            success = true;
            break;
        }
    }

    let reply = json!({
        "successful": success,
    });
    Ok(warp::reply::json(&reply))
}

pub fn qr_for_user_id_handler(body: serde_json::Value, qr_codes: QrCodes) -> impl Reply {
    let qrstring = body["qrstring"].to_string();

    let qrstring: String = serde_json::from_str(&qrstring).unwrap();

    let temp = qr_codes.read().unwrap();

    let guid = temp.get_by_left(&qrstring).unwrap();

    let ser_guid = serde_json::to_string(&guid).unwrap();

    Ok(warp::reply::json(&ser_guid))
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

        let json_string = serde_json::json!({ "googleuserid": "234385785823438578589" });
        println!("{:#?}", json_string);
        let _result = userdata_handler(json_string, db.clone());
        // assert_eq!(result.into_response().into_body(), warp::hyper::Body::from("{\"certificates\":[{\"name\":\"cert1\",\"registerdate\":\"1988-12-30\",\"expirationdate\":\"2022-03-30\"},{\"name\":\"cert2\",\"registerdate\":\"2015-02-19\",\"expirationdate\":\"2021-06-02\"}]}"));

        let json_string = serde_json::json!({ "googleuserid": "fakeid" });
        println!("{:#?}", json_string);
        let _result = userdata_handler(json_string, db);
        // // assert_eq!(result, "{\"certificates\":[]}");
    }
}
