use super::*;
use google_jwt_verify::*;


pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    println!("{}",googleuserid);
    let reply = db.get_certs(googleuserid).unwrap();

    serde_json::to_string(&reply).unwrap()
}

pub fn userdata_handler(body: serde_json::Value, db: Database, ) -> impl Reply {
    let googleuserid = body["googleuserid"].to_string();
    //deserilze googleuserid
    let googleuserid = serde_json::from_str(&googleuserid).unwrap();
    let reply = db.get_user_data(googleuserid).unwrap();
    let ser_reply = serde_json::to_string(&reply).unwrap();
    
    Ok(warp::reply::json(&ser_reply))
}

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

//pub fn post_session_id() -> impl Reply {
//    Ok(warp::reply::json(&token))
//}