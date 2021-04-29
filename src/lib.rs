//! This is the gengar module.
use dotenv::dotenv;
use mysql::{Pool, chrono::NaiveDate, prelude::Queryable};
use std::{env, net::ToSocketAddrs, convert::Infallible};
use warp::{Filter, Reply};
use serde::{Serialize, Deserialize};

pub mod handler;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Token {
    token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserData {
    certificates: Vec<CertData>,
}

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
        self.pool.get_conn()?.query("SELECT GoogleUserID FROM Users")
    }

    /// Returns all vaccination certificates associated with `username`.
    pub fn get_certs(&self, username: String) -> mysql::Result<Vec<String>> {
        self.pool.get_conn()?.query(format!(
            r"SELECT VaccineName
            FROM UserVaccine
            JOIN Users ON Users.UserID = UserVaccine.UserID
            JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
            WHERE GoogleUserID = '{}';",
            username
        ))
    }

    pub fn get_user_data(&self, googleuserid: String) -> mysql::Result<UserData> {
        let mut conn = self.pool.get_conn()?;
        let row: Vec<(String, NaiveDate, NaiveDate)> = conn.query(format!(
            r"SELECT VaccineName, RegisterDate, ExpirationDate
            FROM UserVaccine
            JOIN Users ON Users.UserID = UserVaccine.UserID
            JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
            WHERE GoogleUserID = '{}';",
            googleuserid
        ))?;
        Ok( 
            UserData {
                certificates: row.into_iter().map(row_to_certdata).collect(),
            }
        )
    }

    pub fn get_user_dates(&self, googleuserid: String) -> mysql::Result<Vec<(NaiveDate, NaiveDate)>> {
        self.pool.get_conn()?.query(format!(
            r"SELECT RegisterDate, ExpirationDate
            FROM UserVaccine
            JOIN Users ON Users.UserID = UserVaccine.UserID
            JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
            WHERE GoogleUserID = '{}';",
            googleuserid
        ))
    }
}

fn with_db(db: Database) -> impl Filter<Extract = (Database,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn row_to_certdata(tuple: (String, NaiveDate, NaiveDate)) -> CertData {
    CertData {
        name: tuple.0,
        registerdate: tuple.1,
        expirationdate: tuple.2,
    }
}

pub fn init_db() -> Database {
    Database::new()
}

pub async fn start_server() {
    dotenv().ok();

    let server_url = env::var("SERVER_URL")
        .expect("SERVER_URL must be set")
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let db = init_db();

    let route = warp::any()
                .and(user_certs_route(db.clone()))
                .or(user_data_route(db.clone()))
                .or(post_token_route());

    warp::serve(route)
        .tls()
        .cert_path("tls/localhost.crt")
        .key_path("tls/localhost.key")
        .run(server_url).await;
}

//GET example.org/usercert/:googleuserid 
fn user_certs_route(db: Database) -> warp::filters::BoxedFilter<(impl Reply,)> {
        warp::path!("usercert" / String)
        .map(move |googleuserid: String| handler::usercert_handler(db.clone(), googleuserid)).boxed()
}

//POST example.org/login
fn post_token_route() -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path("login")
    .and(warp::post())
    .and(warp::body::json())
    .map(handler::post_token_handler).boxed()
}

/*
curl -X POST \
-H "Content-type: application/json" \
-H "Accept: application/json" \
-d '{"googleuserid":"234385785823438578589"}' \
"localhost:8000/userdata" 
*/
fn user_data_route(db: Database) -> warp::filters::BoxedFilter<(impl Reply,)> {
    warp::path!("userdata")
    .and(warp::post())
    .and(warp::body::json())
    .and(with_db(db))
    .map(handler::userdata_handler).boxed()
}


// GET example.org/sessionid
// fn session_id_route() -> warp::filters::BoxedFilter<(impl Reply,)> {
//    warp::path!("sessionid")
//    .and(warp::get())
//    .map(handler::post_session_id).boxed()
// }
 
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
        assert_eq!(result[0], "234385785823438578589");
        assert_eq!(result[1], "418446744073709551615");
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

        let result = db.get_user_dates(String::from("234385785823438578589")).unwrap();
    
        assert_eq!(result[0].0.to_string(), String::from("1988-12-30"));
        assert_eq!(result[0].1.to_string(), String::from("2022-03-30"));
    }

    #[test]
    fn get_user_data() {
        let db = Database::new();

        let userdata = db.get_user_data(String::from("234385785823438578589")).unwrap();

        let result = userdata.certificates;
        assert_eq!(result[0].name.to_string(), String::from("cert1"));
        assert_eq!(result[0].registerdate.to_string(), String::from("1988-12-30"));
        assert_eq!(result[0].expirationdate.to_string(), String::from("2022-03-30"));

        assert_eq!(result[1].name.to_string(), String::from("cert2"));
        assert_eq!(result[1].registerdate.to_string(), String::from("2015-02-19"));
        assert_eq!(result[1].expirationdate.to_string(), String::from("2021-06-02"));
    }
}
