impl From<tokio_postgres::row::Row> for User {
    fn from(row: tokio_postgres::row::Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
        }
    }
}
impl<'a> From<&'a tokio_postgres::row::Row> for User {
    fn from(row: &'a tokio_postgres::row::Row) -> Self {
        Self {
            id: row.get("id"),
            name: row.get("name"),
            email: row.get("email"),
        }
    }
}
fn main() {}
