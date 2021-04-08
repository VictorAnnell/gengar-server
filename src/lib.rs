//! This is the gengar module.
use dotenv::dotenv;
use mysql::{prelude::Queryable, Pool};
use std::env;

// Example function
#[doc(hidden)]
pub fn add_two(a: i32) -> i32 {
    a + 2
}

/// Gengar user and vaccine certificate database.
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
        self.pool.get_conn()?.query("SELECT UserName FROM Users")
    }

    /// Returns all vaccination certificates associated with `username`.
    pub fn get_certs(&self, username: String) -> mysql::Result<Vec<String>> {
        self.pool.get_conn()?.query(format!(
            r"SELECT VaccineName
                FROM UserVaccine
                JOIN Users ON Users.UserID = UserVaccine.UserID
                JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
                WHERE UserName = '{}'",
            username
        ))
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    // Test of example function
    #[test]
    fn test_add_two() {
        assert_eq!(4, add_two(2));
    }

    #[test]
    fn db_new() {
        Database::new();
    }

    #[test]
    fn get_users() {
        let db = Database::new();

        let result = db.get_users().unwrap();
        assert_eq!(result[0], "user1");
        assert_eq!(result[1], "user2");
    }

    #[test]
    fn get_certs() {
        let db = Database::new();

        let result = db.get_certs("user1".to_string()).unwrap();
        assert_eq!(result[0], "cert1");
        assert_eq!(result[1], "cert2");

        let result = db.get_certs("nonexistant_user".to_string()).unwrap();
        assert_eq!(result.len(), 0);
    }
}
