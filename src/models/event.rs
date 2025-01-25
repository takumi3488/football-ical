use chrono::NaiveDateTime;

#[derive(PartialEq, Eq, Hash)]
pub struct Event {
    pub start_at: NaiveDateTime,
    pub summary: String,
    pub location: Option<String>,
    pub description: String,
}

impl Event {
    pub fn new(
        start_at: NaiveDateTime,
        summary: String,
        location: Option<String>,
        description: String,
    ) -> Self {
        Self {
            start_at,
            summary,
            location,
            description,
        }
    }

    pub fn to_ical(&self) -> String {
        format!(
            r#"BEGIN:VEVENT
DTSTART:{}
DTEND:{}
SUMMARY:{}
{}
DESCRIPTION:{}
END:VEVENT
"#,
            self.start_at.format("%Y%m%dT%H%M%SZ"),
            (self.start_at + chrono::Duration::hours(2)).format("%Y%m%dT%H%M%SZ"),
            self.summary,
            self.location
                .as_ref()
                .map_or("".to_string(), |location| format!(
                    "LOCATION:{}\n",
                    location
                )),
            self.description
        )
    }
}
