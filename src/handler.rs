use super::*;
use warp::{Filter, Reply};

pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_certs(googleuserid).unwrap();

    serde_json::to_string(&reply).unwrap()
}

pub fn userdata_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_user_data(googleuserid).unwrap();
    serde_json::to_string(&reply).unwrap()
}

pub fn post_token_handler(token: Token) -> impl Reply {
    let mut token = token.clone();
    token.token = String::from("hej");
    Ok(warp::reply::json(
            &token,
    ))
}