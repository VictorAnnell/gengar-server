use super::*;
use google_jwt_verify::*;

pub fn usercert_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_certs(googleuserid).unwrap();

    serde_json::to_string(&reply).unwrap()
}

pub fn userdata_handler(db: Database, googleuserid: String) -> String {
    let reply = db.get_user_data(googleuserid).unwrap();
    serde_json::to_string(&reply).unwrap()
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

    // token.token = String::from("hej");
    // Ok(warp::reply::json(&token))
}
