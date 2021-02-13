use sqlx::FromRow;

#[derive(FromRow)]
pub struct Server {
    pub id: i64,
    pub name: String,
}
