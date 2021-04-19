//! Example of a main binary for gengar module


use warp::{filters::BoxedFilter};

use super::*;


pub fn usercert_handler(db: Database) -> BoxedFilter<(impl Reply,)> {
    //let reply = db.get_certs(username).unwrap().join("|");
    //reply
    let reply = db.get_users().unwrap();
    warp::any()
    .map(move || {
        warp::reply::json(&reply)
    }).boxed()
}
