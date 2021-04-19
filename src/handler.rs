//! Example of a main binary for gengar module
use super::*;


pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_certs(googleuserid).unwrap();
    serde_json::to_string(&reply).unwrap()
}
