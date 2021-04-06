use gengar::establish_pool;
use mysql::prelude::*;
use mysql::*;

fn main() -> std::result::Result<(), Error> {
    let pool = establish_pool().unwrap();
    let result: Vec<(String, String)> = pool
        .get_conn()
        .unwrap()
        .query("SELECT name, certs from users")?;

    println!("Displaying {} users", result.len());
    for user in result {
        println!("{}", user.0);
        println!("{}", user.1);
        println!("-----------\n");
    }
    Ok(())
}
