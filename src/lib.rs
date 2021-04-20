//! This is the gengar module.
use dotenv::dotenv;
use mysql::{prelude::Queryable, Pool};
use std::{env, net::ToSocketAddrs};
use warp::{Filter, Reply};

pub mod handler;
/// Gengar user and vaccine certificate database.

/*
pub struct UserData {
    certificate: String,
    registerdate: mysql::Value,
    expirationdate: mysql::Value,
}*/

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

    //NOT WORKING - problem with dates
    pub fn get_user_data(&self, googleuserid: String) -> mysql::Result<Vec<mysql::Value>> {
        self.pool.get_conn()?.query(format!(
            r"SELECT VaccineName, RegisterDate, ExpirationDate
            FROM UserVaccine
            JOIN Users ON Users.UserID = UserVaccine.UserID
            JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
            WHERE GoogleUserID = '{}';",
            googleuserid
        ))
    }

    //NOT WORKING -- see date
    pub fn get_user_dates(&self, googleuserid: String) -> mysql::Result<Vec<mysql::chrono::NaiveDate>> {
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

    let route = init_route(db);
    
//  let route = route_get_dates(db);

    warp::serve(route)
        .tls()
        .cert_path("tls/localhost.crt")
        .key_path("tls/localhost.key")
        .run(server_url).await;
}

//GET example.org/usercert/:googleuserid 
fn init_route(db: Database) -> warp::filters::BoxedFilter<(impl Reply,)> {
        warp::path!("usercert" / String)
        .map(move |googleuserid: String| handler::usercert_handler(db.clone(), googleuserid)).boxed()
}

//fn route_get_dates(db: Database) -> warp::filters::BoxedFilter<(impl Reply,)> {
//    warp::path!("usercert" / String)
//    .map(move |googleuserid: String| handler::userdate_handler(db.clone(), googleuserid)).boxed()
//}


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
    
        assert_eq!(result[0].to_string(), String::from("sdaas"));
    }
}
