//! This is the gengar module.
use bimap::BiMap;
use dotenv::dotenv;
use mysql::{chrono::NaiveDate, prelude::Queryable, Pool};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    sync::Arc,
};
use std::{convert::Infallible, env, net::ToSocketAddrs};
use warp::{Filter, Reply};

pub mod handler;

/// Google user token information.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GoogleToken {
    id_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct QrCode {
    qr_string: String,
    scanned: bool,
}

impl QrCode {
    pub fn new() -> Self {
        Self {
            qr_string: generate_rand_string(),
            scanned: false,
        }
    }
    pub fn newcustom(qr_string: String) -> Self {
        Self {
            qr_string,
            scanned: false,
        }
    }
}

impl Default for QrCode {
    fn default() -> Self {
        Self::new()
    }
}

// equality only depends on the qr_string data
impl PartialEq for QrCode {
    fn eq(&self, other: &QrCode) -> bool {
        self.qr_string == other.qr_string
    }
}

impl Eq for QrCode {}

impl PartialOrd for QrCode {
    fn partial_cmp(&self, other: &QrCode) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ordering only depends on the qr_string data
impl Ord for QrCode {
    fn cmp(&self, other: &QrCode) -> Ordering {
        self.qr_string.cmp(&other.qr_string)
    }
}

// hash only depends on the qr_string data
impl Hash for QrCode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.qr_string.hash(state);
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionId {
    sessionid: String,
}

impl SessionId {
    pub fn new() -> Self {
        Self {
            sessionid: generate_rand_string(),
        }
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about one or more certificates associated with a single user.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserData {
    certificates: Vec<CertData>,
}

/// Information for a single certificate associated with a user.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CertData {
    name: String,
    registerdate: NaiveDate,
    expirationdate: NaiveDate,
}

type QrCodes = Arc<RwLock<BiMap<QrCode, String>>>;
type SessionIds = Arc<RwLock<BiMap<String, String>>>;

/// Gengar user and vaccine certificate database.
#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    /// Creates a new [Database] object connected to the Gengar MySQL database.
    ///
    /// # Panics
    ///
    /// Will panic if connection to database fails.
    pub fn new() -> Self {
        Self {
            pool: Self::establish_pool(),
        }
    }

    fn establish_pool() -> Pool {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        Pool::new(&database_url).unwrap()
    }

    /// Returns all users in the database.  
    /// Note: function might be removed in future version.
    pub fn get_users(&self) -> mysql::Result<Vec<String>> {
        self.pool
            .get_conn()?
            .query("SELECT GoogleUserID FROM Users")
    }

    /// Returns all vaccination certificates associated with `username`.
    pub fn get_certs(&self, username: String) -> mysql::Result<Vec<String>> {
        let hashed_username = blake3::hash(username.as_bytes()).to_hex().to_string();
        self.pool.get_conn()?.query(format!(
            r"SELECT VaccineName
            FROM UserVaccine
            JOIN Users ON Users.UserID = UserVaccine.UserID
            JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
            WHERE GoogleUserID = '{}';",
            hashed_username
        ))
    }

    /// Returns all vaccination certificates and their created and expiery dates associated with `username`.
    pub fn get_user_data(&self, googleuserid: String) -> mysql::Result<UserData> {
        let hashed_googleuserid = blake3::hash(googleuserid.as_bytes()).to_hex().to_string();
        let mut conn = self.pool.get_conn()?;
        let row: Vec<(String, NaiveDate, NaiveDate)> = conn.query(format!(
            r"SELECT VaccineName, RegisterDate, ExpirationDate
            FROM UserVaccine
            JOIN Users ON Users.UserID = UserVaccine.UserID
            JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
            WHERE GoogleUserID = '{}';",
            hashed_googleuserid
        ))?;
        Ok(UserData {
            certificates: row.into_iter().map(row_to_certdata).collect(),
        })
    }

    /// Returns the creating and expiration dates of all vaccination certificates associated with `username`.
    pub fn get_user_dates(
        &self,
        googleuserid: String,
    ) -> mysql::Result<Vec<(NaiveDate, NaiveDate)>> {
        let hashed_googleuserid = blake3::hash(googleuserid.as_bytes()).to_hex().to_string();
        self.pool.get_conn()?.query(format!(
            r"SELECT RegisterDate, ExpirationDate
            FROM UserVaccine
            JOIN Users ON Users.UserID = UserVaccine.UserID
            JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
            WHERE GoogleUserID = '{}';",
            hashed_googleuserid
        ))
    }

    /// Check if user with `googleuserid` exist in database
    pub fn user_exist(&self, googleuserid: String) -> mysql::Result<bool> {
        let hashed_googleuserid = blake3::hash(googleuserid.as_bytes()).to_hex().to_string();
        let mut conn = self.pool.get_conn()?;
        let row: Vec<String> = conn.query(format!(
            r"SELECT 1
                FROM Users
                WHERE GoogleUserID = '{}';",
            hashed_googleuserid
        ))?;
        Ok(row.len() == 1)
    }
}

fn generate_rand_string() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

fn with_qr_codes(
    qr_codes: QrCodes,
) -> impl Filter<Extract = (QrCodes,), Error = Infallible> + Clone {
    warp::any().map(move || qr_codes.clone())
}

fn with_session_ids(
    session_ids: SessionIds,
) -> impl Filter<Extract = (SessionIds,), Error = Infallible> + Clone {
    warp::any().map(move || session_ids.clone())
}

fn with_db(db: Database) -> impl Filter<Extract = (Database,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_client_id(
    client_id: String,
) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    warp::any().map(move || client_id.clone())
}

/// Converts one row from the database as returned by [`get_user_data`](Database::get_user_data())
/// into a [`CertData`] struct.
fn row_to_certdata(tuple: (String, NaiveDate, NaiveDate)) -> CertData {
    CertData {
        name: tuple.0,
        registerdate: tuple.1,
        expirationdate: tuple.2,
    }
}

/// Launch server and connects to database as defined by settings in file `.env`.
pub async fn start_server() {
    dotenv().ok();

    if env::var_os("RUST_LOG").is_none() {
        // Set `RUST_LOG=debug` to see debug logs,
        // this only shows access logs.
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    let server_url = env::var("SERVER_URL")
        .expect("SERVER_URL must be set")
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let client_id = env::var("CLIENT_ID").expect("CLIENT_ID must be set");

    let db = Database::new();

    let qr_codes: QrCodes = Arc::new(RwLock::new(BiMap::new()));
    let session_ids: SessionIds = Arc::new(RwLock::new(BiMap::new()));

    let route = warp::any()
        .and(user_certs_route(db.clone()))
        .or(user_data_route(db.clone(), session_ids.clone()))
        .or(post_token_route(
            client_id.clone(),
            db.clone(),
            session_ids.clone(),
        ))
        .or(websocket_route())
        .or(user_get_qr_string_route(qr_codes.clone(), db.clone()))
        .or(get_user_id_with_qr_string(qr_codes.clone()))
        .or(verify_cert_route(db.clone(), qr_codes.clone()));

    let route = route.with(warp::log(""));

    let tls = env::var("TLS").expect("TLS must be set");

    if tls == "true" {
        warp::serve(route)
            .tls()
            .cert_path(env::var("CERT_PATH").expect("CERT_PATH must be set"))
            .key_path(env::var("KEY_PATH").expect("KEY_PATH must be set"))
            .run(server_url)
            .await;
    } else {
        warp::serve(route).run(server_url).await;
    }
}

//GET example.org/usercert/:googleuserid
fn user_certs_route(db: Database) -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path!("usercert" / String)
        .map(move |googleuserid: String| handler::usercert_handler(db.clone(), googleuserid))
        .boxed()
}

//POST example.org/getsessionid
fn post_token_route(
    client_id: String,
    db: Database,
    session_ids: SessionIds,
) -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path("getsessionid")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db))
        .and(with_client_id(client_id))
        .and(with_session_ids(session_ids))
        .map(handler::post_token_handler)
        .boxed()
}

/*
curl -X POST \
-H "Content-type: application/json" \
-H "Accept: application/json" \
-d '{"googleuserid":"234385785823438578589"}' \
"localhost:8000/userdata"
*/
//GET example.org/userdata/:googleuserid
fn user_data_route(
    db: Database,
    session_ids: SessionIds,
) -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path!("userdata")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db))
        .and(with_session_ids(session_ids))
        .map(handler::userdata_handler)
        .boxed()
}

fn user_get_qr_string_route(
    qr_codes: QrCodes,
    db: Database,
) -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path!("getqr")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_qr_codes(qr_codes))
        .and(with_db(db))
        .map(handler::get_qr_handler)
        .boxed()
}

fn get_user_id_with_qr_string(qr_codes: QrCodes) -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path!("postqr")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_qr_codes(qr_codes))
        .map(handler::qr_for_user_id_handler)
        .boxed()
}

// TODO: complete websocket route
fn websocket_route() -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path("echo")
    // The `ws()` filter will prepare the Websocket handshake.
    .and(warp::ws())
    .map(handler::websocket_handler).boxed()
}

// GET example.org/sessionid
// fn session_id_route() -> warp::filters::BoxedFilter<(impl Reply,)> {
//    warp::path!("sessionid")
//    .and(warp::get())
//    .map(handler::post_session_id).boxed()
// }

fn verify_cert_route(db: Database, qr_codes: QrCodes) -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path!("verify")
        .and(warp::post())
        .and(warp::body::json())
        .and(with_db(db))
        .and(with_qr_codes(qr_codes))
        .map(handler::verify_cert_handler)
        .boxed()
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn db_new() {
        Database::new();
    }

    #[test]
    fn get_users() {
        let db = Database::new();

        let result = db.get_users().unwrap();
        assert_eq!(
            result[0],
            "5fa6e51bb84716c9b0ba630712997abb0ac5bd178ef1b2a59c4f64073d07055d"
        );
        assert_eq!(
            result[1],
            "1170c459c8c96ef276770a88225879153dbe1b3f2810dd53928633bbcd13a955"
        );
    }

    #[test]
    fn get_certs() {
        let db = Database::new();

        let result = db.get_certs("234385785823438578589".to_string()).unwrap();
        assert_eq!(result[0], "cert1");
        assert_eq!(result[1], "cert2");

        let result = db.get_certs("418446744073709551615".to_string()).unwrap();
        assert_eq!(result[0], "cert2");

        let result = db.get_certs("nonexistant_user".to_string()).unwrap();
        assert_eq!(result.len(), 0);
    }
    #[test]
    fn get_user_dates() {
        let db = Database::new();

        let result = db
            .get_user_dates(String::from("234385785823438578589"))
            .unwrap();

        assert_eq!(result[0].0.to_string(), String::from("1988-12-30"));
        assert_eq!(result[0].1.to_string(), String::from("2022-03-30"));
    }

    #[test]
    fn get_user_data() {
        let db = Database::new();

        let userdata = db
            .get_user_data(String::from("234385785823438578589"))
            .unwrap();

        let result = userdata.certificates;
        assert_eq!(result[0].name.to_string(), String::from("cert1"));
        assert_eq!(
            result[0].registerdate.to_string(),
            String::from("1988-12-30")
        );
        assert_eq!(
            result[0].expirationdate.to_string(),
            String::from("2022-03-30")
        );

        assert_eq!(result[1].name.to_string(), String::from("cert2"));
        assert_eq!(
            result[1].registerdate.to_string(),
            String::from("2015-02-19")
        );
        assert_eq!(
            result[1].expirationdate.to_string(),
            String::from("2021-06-02")
        );
    }

    #[test]
    fn user_exist() {
        let db = Database::new();
        assert_eq!(
            db.user_exist(String::from("234385785823438578589"))
                .unwrap(),
            true
        );
        assert_eq!(
            db.user_exist(String::from("nonexistant_user")).unwrap(),
            false
        );
    }

    #[test]
    fn generate_rand_string_test() {
        let rand_1 = generate_rand_string();
        let rand_2 = generate_rand_string();
        assert_eq!(rand_1.len(), rand_2.len());
        assert_ne!(rand_1, rand_2);
    }
}
