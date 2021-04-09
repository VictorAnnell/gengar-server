//! Example binary to demonstrate usage of gengar library
//! Run this binary with the command:
//! ```
//! $ cargo run --bin show_users
//! ```
use gengar::Database;

fn main() {
    let result = Database::new().get_users().unwrap();

    println!("Displaying {} users", result.len());
    for user in result {
        println!("{}", user);
        println!("-----------");
    }
}
