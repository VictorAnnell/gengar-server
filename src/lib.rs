use dotenv::dotenv;
use mysql::prelude::*;
use mysql::*;
use std::env;

pub fn add_two(a: i32) -> i32 {
    a + 2
}

fn establish_pool() -> Pool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Pool::new(&database_url).unwrap()
}

pub struct Database {
    pool: Pool,
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

impl Database {
    pub fn new() -> Self {
        Self {
            pool: establish_pool(),
        }
    }

    pub fn get_users(&self) -> mysql::Result<Vec<String>> {
        self.pool.get_conn()?.query("SELECT UserName FROM Users")
    }

    pub fn get_certs(&self, name: String) -> Result<Vec<String>> {
        self.pool.get_conn()?.query(format!(
            r"SELECT VaccineName
                FROM UserVaccine
                JOIN Users ON Users.UserID = UserVaccine.UserID
                JOIN Vaccines ON Vaccines.VaccineID = UserVaccine.VaccineID
                WHERE UserName = '{}'",
            name
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_two() {
        assert_eq!(4, add_two(2));
    }

    #[test]
    fn test_db() {
        Database::new();
    }

    #[test]
    fn test_get_certs() {
        let db = Database::new();

        let result = db.get_certs("user1".to_string()).unwrap();
        assert_eq!(result[0], "cert1");
        assert_eq!(result[1], "cert2");

        let result = db.get_certs("nonexistant_user".to_string()).unwrap();
        assert_eq!(result.len(), 0);
    }
}
