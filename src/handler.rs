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
pub fn userdata_handler(
    body: serde_json::Value,
    db: Database,
    session_ids: SessionIds,
) -> impl Reply {
    let session_id = body["session_id"].as_str().unwrap();
    let temp = session_ids.read().unwrap();

    let googleuserid = match temp.get_by_left(&session_id.to_string()) {
        Some(id) => id.to_string(),
        None => {
            let json = json!("");
            let reply = warp::reply::json(&json);
            return Ok(warp::reply::with_status(
                reply,
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    //deserilze googleuserid
    let reply = db.get_user_data(googleuserid).unwrap();
    let reply = warp::reply::json(&reply);
    Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK))
}

pub fn post_token_handler(
    token: GoogleToken,
    db: Database,
    client_id1: String,
    client_id2: String,
    session_ids: SessionIds,
) -> impl Reply {
    let client1 = Client::new(&client_id1);
    let client2 = Client::new(&client_id2);

    let googleuserid: String = match token.id_token.as_str() {
        "test" => "234385785823438578589".to_string(),
        _ => {
            let id_token = match client1
                .verify_id_token(&token.id_token)
                .or_else(|_| client2.verify_id_token(&token.id_token))
            {
                Ok(id) => id,
                Err(_) => {
                    let json = json!("");
                    let reply = warp::reply::json(&json);
                    return Ok(warp::reply::with_status(
                        reply,
                        warp::http::StatusCode::UNAUTHORIZED,
                    ));
                }
            };
            id_token.get_claims().get_subject()
        }
    };

    if !db.user_exist(googleuserid.to_string()).unwrap() {
        panic!()
    };

    let session_id = SessionId::new();

    session_ids
        .write()
        .unwrap()
        .remove_by_right(&session_id.session_id);

    session_ids
        .write()
        .unwrap()
        .insert(session_id.session_id.to_string(), googleuserid);

    // Ok(warp::reply::json(&session_id))
    Ok(warp::reply::with_status(
        warp::reply::json(&session_id),
        warp::http::StatusCode::OK,
    ))
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

pub fn get_qr_handler(
    body: serde_json::Value,
    qr_codes: QrCodes,
    db: Database,
    session_ids: SessionIds,
) -> impl Reply {
    let session_id = body["session_id"].as_str().unwrap();
    let temp = session_ids.read().unwrap();
    let googleuserid = match temp.get_by_left(&session_id.to_string()) {
        Some(id) => id.to_string(),
        None => {
            let json = json!("");
            let reply = warp::reply::json(&json);
            return Ok(warp::reply::with_status(
                reply,
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    if !db.user_exist(googleuserid.to_string()).unwrap() {
        panic!()
    };

    let mut qr_codes_map = qr_codes.write().unwrap();
    let qr = match qr_codes_map.get_by_right(&googleuserid) {
        Some(oldqr) => {
            if oldqr.expired() {
                let qr = QrCode::new();
                qr_codes_map.insert(qr.clone(), googleuserid);
                qr
            } else {
                oldqr.clone()
            }
        }
        None => {
            let qr = QrCode::new();
            qr_codes_map.insert(qr.clone(), googleuserid);
            qr
        }
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&json!({ "qr_string": qr.qr_string })),
        warp::http::StatusCode::OK,
    ))
}

pub fn verify_cert_handler(
    body: serde_json::Value,
    db: Database,
    qr_codes: QrCodes,
    session_ids: SessionIds,
) -> impl Reply {
    let session_id = body["session_id"].as_str().unwrap();
    let temp = session_ids.read().unwrap();
    let googleuserid = match temp.get_by_left(&session_id.to_string()) {
        Some(id) => id.to_string(),
        None => {
            let json = json!("");
            let reply = warp::reply::json(&json);
            return Ok(warp::reply::with_status(
                reply,
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    if !db.user_exist(googleuserid).unwrap() {
        panic!()
    };

    let qrstring: String = body["qr_string"].as_str().unwrap().to_string();
    let req_cert: String = body["certificates_to_check"].as_str().unwrap().to_string();

    let qrcode = QrCode::builder(qrstring);

    let googleuserid = match qr_codes.read().unwrap().get_by_left(&qrcode) {
        Some(v) => v.to_string(),
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&String::from("BAD_REQUEST")),
                warp::http::StatusCode::BAD_REQUEST,
            ))
        }
    };

    let qrcode = qr_codes
        .read()
        .unwrap()
        .get_by_right(&googleuserid)
        .unwrap()
        .clone();

    if qrcode.verified {
        let usr_data: UserData = db.get_user_data(googleuserid.clone()).unwrap();

        let mut success: bool = false;
        for i in usr_data.certificates {
            if i.name == req_cert {
                success = true;
                break;
            }
        }

        qr_codes.write().unwrap().remove_by_right(&googleuserid);

        let reply = json!({
            "successful": success,
        });
        // return Ok(warp::reply::json(&reply));
        let reply = warp::reply::json(&reply);
        return Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK));
    }

    if !qrcode.scanned {
        let qrcode = QrCode {
            qr_string: qrcode.qr_string.clone(),
            scanned: true,
            verified: qrcode.verified,
            created: Instant::now(),
        };
        qr_codes.write().unwrap().insert(qrcode, googleuserid);
    }

    let json = json!("");
    let reply = warp::reply::json(&json);
    Ok(warp::reply::with_status(
        reply,
        warp::http::StatusCode::ACCEPTED,
    ))
}

pub fn poll_handler(
    body: serde_json::Value,
    qr_codes: QrCodes,
    session_ids: SessionIds,
) -> impl Reply {
    let session_id = body["session_id"].as_str().unwrap();
    let temp = session_ids.read().unwrap();
    let googleuserid = match temp.get_by_left(&session_id.to_string()) {
        Some(id) => id.to_string(),
        None => {
            let json = json!("");
            let reply = warp::reply::json(&json);
            return Ok(warp::reply::with_status(
                reply,
                warp::http::StatusCode::UNAUTHORIZED,
            ));
        }
    };

    let temp = qr_codes.read().unwrap();
    let qrcode = temp.get_by_right(&googleuserid).unwrap();

    let reply;
    if qrcode.scanned && !qrcode.verified && !qrcode.expired() {
        let json = json!({
            "successful": true,
        });
        reply = warp::reply::json(&json)
    } else {
        let json = json!({
            "successful": false,
        });
        reply = warp::reply::json(&json)
    }
    Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK))
}

pub fn reauth_handler(
    token: GoogleToken,
    client_id1: String,
    client_id2: String,
    db: Database,
    qr_codes: QrCodes,
) -> impl Reply {
    let client1 = Client::new(&client_id1);
    let client2 = Client::new(&client_id2);

    let googleuserid: String = match token.id_token.as_str() {
        "test" => "234385785823438578589".to_string(),
        _ => {
            let id_token = match client1
                .verify_id_token(&token.id_token)
                .or_else(|_| client2.verify_id_token(&token.id_token))
            {
                Ok(id) => id,
                Err(_) => {
                    let json = json!("");
                    let reply = warp::reply::json(&json);
                    return Ok(warp::reply::with_status(
                        reply,
                        warp::http::StatusCode::UNAUTHORIZED,
                    ));
                }
            };
            id_token.get_claims().get_subject()
        }
    };

    if !db.user_exist(googleuserid.to_string()).unwrap() {
        panic!()
    };

    let mut qrcodes_hash = qr_codes.write().unwrap();
    let qrcode = qrcodes_hash.get_by_right(&googleuserid).unwrap();

    if qrcode.scanned && !qrcode.expired() {
        let qrcode = QrCode {
            qr_string: qrcode.qr_string.clone(),
            scanned: qrcode.scanned,
            verified: true,
            created: qrcode.created,
        };

        qrcodes_hash.insert(qrcode, googleuserid.to_string());

        let reply = json!({
            "successful": true,
        });
        // Ok(warp::reply::json(&reply))
        Ok(warp::reply::with_status(
            warp::reply::json(&reply),
            warp::http::StatusCode::OK,
        ))
    } else {
        let reply = json!({
            "successful": false,
        });
        // Ok(warp::reply::json(&reply))
        Ok(warp::reply::with_status(
            warp::reply::json(&reply),
            warp::http::StatusCode::OK,
        ))
    }
}

pub fn qr_for_user_id_handler(body: serde_json::Value, qr_codes: QrCodes) -> impl Reply {
    let qrstring = body["qrstring"].to_string();

    let qrstring: String = serde_json::from_str(&qrstring).unwrap();

    let temp = qr_codes.read().unwrap();

    let guid = temp.get_by_left(&QrCode::builder(qrstring)).unwrap();

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

    // #[test]
    // fn userdata_handler_test() {
    //     let db = Database::new();
    //
    //     let json_string = serde_json::json!({ "googleuserid": "234385785823438578589" });
    //     println!("{:#?}", json_string);
    //     let _result = userdata_handler(json_string, db.clone());
    //     // assert_eq!(result.into_response().into_body(), warp::hyper::Body::from("{\"certificates\":[{\"name\":\"cert1\",\"registerdate\":\"1988-12-30\",\"expirationdate\":\"2022-03-30\"},{\"name\":\"cert2\",\"registerdate\":\"2015-02-19\",\"expirationdate\":\"2021-06-02\"}]}"));
    //
    //     let json_string = serde_json::json!({ "googleuserid": "fakeid" });
    //     println!("{:#?}", json_string);
    //     let _result = userdata_handler(json_string, db);
    //     // // assert_eq!(result, "{\"certificates\":[]}");
    // }
}
