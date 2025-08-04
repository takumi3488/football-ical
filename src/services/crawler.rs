use anyhow::Result;
use chrono::{Datelike, NaiveDateTime, Utc};
use regex::Regex;
use scraper::Html;

use crate::models::event::Event;

pub struct CrawlResponse {
    pub name: String,
    pub events: Vec<Event>,
}

pub async fn crawl(document: &str) -> Result<CrawlResponse> {
    let document = Html::parse_document(document);
    let name = document
        .select(&scraper::Selector::parse(".sc-teamTitle__name").unwrap())
        .next()
        .ok_or(anyhow::anyhow!("Failed to find team name"))?
        .text()
        .collect::<String>();

    let mut last_start_at = NaiveDateTime::parse_from_str(
        &format!("{}-01-01 00:00", Utc::now().year()),
        "%Y-%m-%d %H:%M",
    )
    .unwrap();
    let events = document
        .select(&scraper::Selector::parse("#scheduleTable > table > tbody > tr").unwrap())
        .filter_map(|row| {
            if row
                .select(
                    &scraper::Selector::parse("td.sc-tableGame__data.sc-tableGame__data--score")
                        .unwrap(),
                )
                .next()
                .is_none_or(|score| {
                    score
                        .text()
                        .collect::<Vec<_>>()
                        .join("")
                        .contains("試合終了")
                })
            {
                return None;
            }

            let start_at_ptn = Regex::new(r#"(\d{1,2})\/(\d{1,2})（.）(\d{2}):(\d{2})"#).unwrap();
            let start_at = row
                .select(
                    &scraper::Selector::parse("td.sc-tableGame__data.sc-tableGame__data--date")
                        .unwrap(),
                )
                .next()
                .unwrap()
                .text()
                .map(|text| text.trim())
                .collect::<Vec<_>>()
                .join("");
            let start_at_matches = start_at_ptn.captures(&start_at);
            start_at_matches.as_ref()?;
            let start_at_matches = start_at_matches.unwrap();
            let mut year = last_start_at.year();
            let month = start_at_matches
                .get(1)
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap();
            if month < last_start_at.month() {
                year += 1;
            }
            let date = start_at_matches
                .get(2)
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap();
            let hour = start_at_matches
                .get(3)
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap();
            let minute = start_at_matches
                .get(4)
                .unwrap()
                .as_str()
                .parse::<u32>()
                .unwrap();
            let start_at_result = NaiveDateTime::parse_from_str(
                &format!("{}-{}-{} {:02}:{:02}", year, month, date, hour % 24, minute),
                "%Y-%m-%d %H:%M",
            );
            if start_at_result.is_err() {
                return None;
            }
            let mut start_at = start_at_result.unwrap();
            if hour >= 24 {
                start_at += chrono::Duration::days(1);
            }
            if start_at < last_start_at {
                start_at = NaiveDateTime::parse_from_str(
                    &format!(
                        "{}-{}-{} {:02}:{:02}",
                        last_start_at.year() + 1,
                        start_at_matches.get(1).unwrap().as_str(),
                        start_at_matches.get(2).unwrap().as_str(),
                        start_at_matches.get(3).unwrap().as_str(),
                        start_at_matches.get(4).unwrap().as_str()
                    ),
                    "%Y-%m-%d %H:%M",
                )
                .map_err(|e| anyhow::anyhow!("Failed to parse start_at: {}", e))
                .unwrap();
                last_start_at = start_at;
            }

            let summary = row
                .select(
                    &scraper::Selector::parse(
                        "td.sc-tableGame__data.sc-tableGame__data--team > a.sc-tableGame__team:last-child > span",
                    )
                    .unwrap(),
                )
                .filter_map(|team| team.text().next())
                .collect::<Vec<_>>()
                .join(" - ");

            let location = row
                .select(
                    &scraper::Selector::parse("td.sc-tableGame__data.sc-tableGame__data--venue")
                        .unwrap(),
                )
                .map(|location| location.text().next().unwrap())
                .next()
                .map(|location| location.to_string());

            let description = row
                .select(
                    &scraper::Selector::parse("td.sc-tableGame__data.sc-tableGame__data--category")
                        .unwrap(),
                )
                .map(|description| description.text().next().unwrap())
                .next()
                .unwrap()
                .to_string();

            Some(Event::new(start_at, summary, location, description))
        })
        .collect();

    Ok(CrawlResponse { name, events })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_crawl() {
        let document = include_str!("./fixtures/1.html");
        let events = crawl(document).await.unwrap().events;
        assert_eq!(events.len(), 10);
        assert_eq!(
            events.last().unwrap().start_at,
            NaiveDateTime::parse_from_str("2025-07-02 19:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
        assert_eq!(events.last().unwrap().summary, "神戸 - 広島".to_string());
        assert_eq!(
            events.last().unwrap().location,
            Some("ノエスタ".to_string())
        );

        let document = include_str!("./fixtures/2.html");
        let events = crawl(document).await.unwrap().events;
        assert_eq!(events.len(), 21);
        assert_eq!(
            events.last().unwrap().start_at,
            NaiveDateTime::parse_from_str("2025-05-26 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );

        let document = include_str!("./fixtures/3.html");
        let events = crawl(document).await.unwrap().events;
        assert_eq!(events.len(), 2);
        assert_eq!(
            events.last().unwrap().start_at,
            NaiveDateTime::parse_from_str("2025-03-25 19:35:00", "%Y-%m-%d %H:%M:%S").unwrap()
        );
    }
}
