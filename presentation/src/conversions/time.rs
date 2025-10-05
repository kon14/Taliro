pub(crate) trait DateTimeExtPresentation {
    fn to_dmn_datetime(&self) -> domain::types::time::DateTime;
}

impl DateTimeExtPresentation for chrono::DateTime<chrono::Utc> {
    fn to_dmn_datetime(&self) -> domain::types::time::DateTime {
        let ms: u64 = self.timestamp_millis() as u64;
        domain::types::time::DateTime::from_ms(ms)
    }
}
