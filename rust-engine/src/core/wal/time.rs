/// UTC `YYYYMMDDHH` bucket for hourly WAL segment filenames.
pub fn utc_hour_bucket(now: std::time::SystemTime) -> u64 {
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_secs();
    let days = secs / 86_400;
    let hour = (secs % 86_400) / 3_600;
    let (year, month, day) = civil_from_days(days as i64);
    year as u64 * 1_000_000 + month as u64 * 10_000 + day as u64 * 100 + hour
}

/// Days since 1970-01-01 → (year, month, day) in UTC.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year as i32, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, UNIX_EPOCH};

    #[test]
    fn epoch_hour_bucket() {
        assert_eq!(
            utc_hour_bucket(UNIX_EPOCH + Duration::from_secs(3_600)),
            1970010101
        );
    }
}
