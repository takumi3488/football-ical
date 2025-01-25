use actix_files as fs;
use actix_web::{get, web, App, HttpServer};
use anyhow::Result;
use football_ical::{
    handlers::team::{create_team, find_all_teams, flip_team_status},
    AppState,
};
use std::{env, path::Path};
use tracing::debug;
use tracing_actix_web::TracingLogger;

#[actix_web::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/postgres".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await?;
    let state = AppState { pool: pool.clone() };
    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(state.clone()))
            .wrap(TracingLogger::default())
            .service(
                web::scope("/api")
                    .service(health)
                    .service(find_all_teams)
                    .service(create_team)
                    .service(flip_team_status),
            );
        if Path::new("./dist").is_dir() {
            app = app.service(fs::Files::new("/", "./dist").index_file("index.html"));
        }
        app
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await?;

    Ok(())
}

#[get("/health")]
async fn health() -> String {
    debug!("health check");
    "OK".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::{test, App};

    #[actix_web::test]
    async fn test_health() {
        let app = test::init_service(App::new().service(health)).await;
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
