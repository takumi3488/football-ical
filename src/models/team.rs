use anyhow::Result;
use regex::Regex;
use serde::Serialize;

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct Team {
    pub id: i32,
    pub url: String,
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct NewTeam {
    pub url: String,
    pub name: String,
    pub enabled: bool,
}

impl NewTeam {
    pub fn new(url: &str, name: &str, enabled: bool) -> Self {
        Self {
            url: url.to_string(),
            name: name.to_string(),
            enabled,
        }
    }

    pub async fn save(&self, pool: &sqlx::PgPool) -> Result<Team> {
        sqlx::query_as!(
            Team,
            "INSERT INTO teams (url, name, enabled) VALUES ($1, $2, $3) RETURNING *",
            self.url,
            self.name,
            self.enabled
        )
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to save team: {}", e))
    }
}

impl Team {
    pub async fn find_all(pool: &sqlx::PgPool) -> Result<Vec<Self>> {
        let teams = sqlx::query_as!(Team, "SELECT * FROM teams")
            .fetch_all(pool)
            .await?;

        Ok(teams)
    }

    pub async fn find_all_active(pool: &sqlx::PgPool) -> Result<Vec<Self>> {
        let teams = sqlx::query_as!(Team, "SELECT * FROM teams WHERE enabled = true")
            .fetch_all(pool)
            .await?;

        Ok(teams)
    }

    pub async fn flip_status(id: i32, pool: &sqlx::PgPool) -> Result<()> {
        sqlx::query!("UPDATE teams SET enabled = NOT enabled WHERE id = $1", id)
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn get_document(&self) -> Result<String> {
        let response = reqwest::get(&self.url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get {}: {}", self.url, e))?;
        let text = response.text().await?;
        Ok(text)
    }
}

/// sportsnaviのURLをスケジュールページのURLに変換する
pub async fn to_schedule_url(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;
    let url = response.url().as_str();
    let base_url_ptn = Regex::new(r"^(https://soccer.yahoo.co.jp/.+/teams?/\d+)")?;
    let base_url = base_url_ptn
        .captures(url)
        .ok_or_else(|| anyhow::anyhow!("Invalid URL"))?[1]
        .to_string();
    let schedule_url = format!("{}/schedule", base_url);
    Ok(schedule_url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_to_schedule_url() {
        let url = "https://soccer.yahoo.co.jp/jleague/team/136";
        let schedule_url = to_schedule_url(url).await.unwrap();
        assert_eq!(
            schedule_url,
            "https://soccer.yahoo.co.jp/jleague/category/j1/teams/136/schedule"
        );

        let url = "https://soccer.yahoo.co.jp/ws/category/eng/teams/4075/info?gk=52";
        let schedule_url = to_schedule_url(url).await.unwrap();
        assert_eq!(
            schedule_url,
            "https://soccer.yahoo.co.jp/ws/category/eng/teams/4075/schedule"
        );

        let url = "https://soccer.yahoo.co.jp/japan/category/men/teams/142/schedule";
        let schedule_url = to_schedule_url(url).await.unwrap();
        assert_eq!(
            schedule_url,
            "https://soccer.yahoo.co.jp/japan/category/men/teams/142/schedule"
        );

        let url = "https://soccer.yahoo.co.jp/ws/team/4066";
        let schedule_url = to_schedule_url(url).await.unwrap();
        assert_eq!(
            schedule_url,
            "https://soccer.yahoo.co.jp/ws/category/eng/teams/4066/schedule"
        );
    }
}
