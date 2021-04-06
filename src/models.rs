#[derive(Queryable)]
pub struct User {
    pub id: u64,
    pub user: String,
    pub certs: String,
}
