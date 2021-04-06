use dotenv::dotenv;
use mysql::*;
use std::env;

pub fn add_two(a: i32) -> i32 {
    a + 2
}

pub fn establish_pool() -> Result<Pool> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Pool::new(&database_url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_two() {
        assert_eq!(4, add_two(2));
    }

    #[test]
    fn test_establish_pool() {
        assert!(establish_pool().is_ok());
    }
}
