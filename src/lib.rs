pub mod handlers;
pub mod models;
pub mod services;

#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
}
