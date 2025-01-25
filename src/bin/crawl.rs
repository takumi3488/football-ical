use std::{collections::HashSet, env};

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{config::Builder, primitives::ByteStream};
use football_ical::{models::team::Team, services::crawler::crawl};

#[actix_web::main]
async fn main() {
    // クライアントの初期化
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/postgres".to_string());
    let pool = sqlx::PgPool::connect(&database_url).await.unwrap();
    let s3_endpoint =
        env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9001".to_string());
    let s3_region = env::var("S3_REGION").unwrap_or_else(|_| "ap-northeast-1".to_string());
    let config_loader = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(&s3_endpoint)
        .region(Region::new(s3_region));
    let config = config_loader.load().await;
    let config = Builder::from(&config).force_path_style(true).build();
    let s3_client = aws_sdk_s3::Client::from_conf(config);

    // イベント一覧の取得
    let teams = Team::find_all_active(&pool).await.unwrap();
    let mut events_list = HashSet::new();
    for team in teams {
        let document = team.get_document().await.unwrap();
        let crawl_response = crawl(&document).await.unwrap();
        events_list.extend(crawl_response.events);
    }

    // イベント一覧をical形式に変換
    let mut ical = r#"BEGIN:VCALENDAR
VERSION:2.0
BEGIN:VTIMEZONE
TZID:Asia/Tokyo
BEGIN:STANDARD
DTSTART:19700101T000000
TZOFFSETFROM:+0900
TZOFFSETTO:+0900
END:STANDARD
END:VTIMEZONE
"#
    .to_string();
    for event in events_list {
        ical.push_str(&event.to_ical());
    }
    ical.push_str("END:VCALENDAR");

    // icalファイルをS3にアップロード
    let bucket = env::var("S3_BUCKET").unwrap();
    let key = env::var("S3_KEY").unwrap();
    s3_client
        .put_object()
        .bucket(&bucket)
        .key(&key)
        .body(ByteStream::from(ical.as_bytes().to_vec()))
        .send()
        .await
        .unwrap();
}
