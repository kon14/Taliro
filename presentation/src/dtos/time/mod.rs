use chrono::TimeZone;

pub(crate) trait DateTimeExtPresentation {
    fn to_chrono(self) -> chrono::DateTime<chrono::Utc>;
}

impl DateTimeExtPresentation for domain::types::time::DateTime {
    fn to_chrono(self) -> chrono::DateTime<chrono::Utc> {
        chrono::Utc
            .timestamp_millis_opt(self.to_ms() as i64)
            .single()
            .expect("Invalid timestamp") // let's be real...
    }
}
