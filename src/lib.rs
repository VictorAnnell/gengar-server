//! This is the gengar module.
use dotenv::dotenv;
use mysql::{chrono::NaiveDate, prelude::Queryable, Pool};
use serde::{Deserialize, Serialize};
use std::{env, net::ToSocketAddrs};
use warp::{Filter, Reply};

pub mod handler;

/// Google user token information.
// TODO: change to reflect token information to be recieved from client
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    token: String,
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
}

/// Converts one row from the database as returned by [`get_user_data`](Database::get_user_data())
/// into a [`CertData`] struct.
pub fn row_to_certdata(tuple: (String, NaiveDate, NaiveDate)) -> CertData {
    CertData {
        name: tuple.0,
        registerdate: tuple.1,
        expirationdate: tuple.2,
    }
}

/// Launch server and connects to database as defined by settings in file `.env`.
pub async fn start_server() {
    dotenv().ok();

    let server_url = env::var("SERVER_URL")
        .expect("SERVER_URL must be set")
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let db = Database::new();

    let route = warp::any()
        .and(user_certs_route(db.clone()))
        .or(user_data_route(db.clone()))
        .or(post_token_route())
        .or(websocket_route());

    let tls = env::var("TLS").expect("TLS must be set");

    if tls == "true" {
        warp::serve(route)
            .tls()
            .cert_path("tls/localhost.crt")
            .key_path("tls/localhost.key")
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

// TODO: complete google auth route
//POST example.org/login
fn post_token_route() -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path!("login")
        .and(warp::post())
        .and(warp::body::json())
        .map(handler::post_token_handler)
        .boxed()
}

//GET example.org/userdata/:googleuserid
fn user_data_route(db: Database) -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path!("userdata" / String)
        .map(move |googleuserid: String| handler::userdata_handler(db.clone(), googleuserid))
        .boxed()
}

// TODO: complete websocket route
fn websocket_route() -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path("echo")
    // The `ws()` filter will prepare the Websocket handshake.
    .and(warp::ws())
    .map(handler::websocket_handler).boxed()
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
}
