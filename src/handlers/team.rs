use actix_web::{get, patch, post, web, HttpResponse, Responder};
use serde::Deserialize;

use crate::{
    models::team::{to_schedule_url, NewTeam, Team},
    services::crawler::crawl,
    AppState,
};

#[derive(Deserialize)]
pub struct TeamRequest {
    url: String,
}

#[get("/teams")]
async fn find_all_teams(state: web::Data<AppState>) -> impl Responder {
    let teams = Team::find_all(&state.pool).await.unwrap();
    HttpResponse::Ok().json(teams)
}

#[post("/teams")]
async fn create_team(state: web::Data<AppState>, req: web::Json<TeamRequest>) -> impl Responder {
    let url = to_schedule_url(&req.url).await.unwrap();
    let document = reqwest::get(&url).await.unwrap().text().await.unwrap();
    let crawl_response = crawl(&document).await.unwrap();
    let team = NewTeam::new(&url, &crawl_response.name, true);
    team.save(&state.pool).await.unwrap();
    HttpResponse::Ok().json(team)
}

#[patch("/teams/{id}/flip_status")]
async fn flip_team_status(id: web::Path<i32>, state: web::Data<AppState>) -> impl Responder {
    Team::flip_status(*id, &state.pool).await.unwrap();
    HttpResponse::NoContent().finish()
}
