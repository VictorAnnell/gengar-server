use super::*;
use warp::{http::StatusCode, Rejection, Reply};

pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_certs(googleuserid).unwrap();

    serde_json::to_string(&reply).unwrap()
}

pub fn post_token_handler(token: String) -> String {
    token
}