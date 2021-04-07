use gengar::Database;

fn main() {
    let result = Database::new().get_users().unwrap();

    println!("Displaying {} users", result.len());
    for user in result {
        println!("{}", user.0);
        println!("{}", user.1);
        println!("-----------\n");
    }
}
