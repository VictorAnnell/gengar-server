use gengar::{add_two, Database};

// Integration test of example function
#[test]
fn it_adds_two() {
    assert_eq!(4, add_two(2));
}

// Integration test printing the certificates belonging to user1.
// Note: does not actually print upon successful test.
#[test]
fn test_print_certs() {
    let db = Database::new();

    let result = db.get_certs("user1".to_string()).unwrap();
    assert_eq!(result[0], "cert1");
    assert_eq!(result[1], "cert2");
    for cert in result {
        println!("{}", cert);
    }

    let result = db.get_certs("nonexistant_user".to_string()).unwrap();
    assert_eq!(result.len(), 0);
}
