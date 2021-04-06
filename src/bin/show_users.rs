extern crate diesel;
extern crate gengar;

use self::models::*;
use diesel::prelude::*;
use gengar::*;

fn main() {
    use self::schema::users::dsl::*;

    let connection = establish_connection();
    let results = users
        // .limit(5)
        .load::<User>(&connection)
        .expect("Error loading users");

    println!("Displaying {} users", results.len());
    for user in results {
        println!("{}", user.user);
        println!("-----------\n");
        println!("{}", user.certs);
    }
}
