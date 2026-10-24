// SPDX-License-Identifier: MIT OR Apache-2.0
//! RFC 3339 timestamps in the one form the matrix file uses: `YYYY-MM-DDTHH:MM:SSZ`, UTC.

use crate::Timestamp;

/// Days since 1970-01-01 of a civil date (proleptic Gregorian).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        _ => 28,
    }
}

/// Format as `YYYY-MM-DDTHH:MM:SSZ` (years 0000-9999 only; outside that, the year is clamped to
/// the nearest end rather than writing a form the reader refuses).
pub fn format(t: Timestamp) -> String {
    const MIN: i64 = -62_167_219_200; // 0000-01-01T00:00:00Z
    const MAX: i64 = 253_402_300_799; // 9999-12-31T23:59:59Z
    let t = t.clamp(MIN, MAX);
    let (y, mo, d) = civil_from_days(t.div_euclid(86_400));
    let s = t.rem_euclid(86_400);
    format!(
        "{y:04}-{mo:02}-{d:02}T{:02}:{:02}:{:02}Z",
        s / 3600,
        s % 3600 / 60,
        s % 60
    )
}

/// Parse exactly `YYYY-MM-DDTHH:MM:SSZ`. Anything else (offsets, fractions, lower case) is `None`.
pub fn parse(s: &str) -> Option<Timestamp> {
    let b = s.as_bytes();
    if b.len() != 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
        || b[19] != b'Z'
    {
        return None;
    }
    let num = |a: usize, z: usize| -> Option<i64> {
        let part = &s[a..z];
        part.bytes()
            .all(|c| c.is_ascii_digit())
            .then(|| part.parse().ok())?
    };
    let (y, mo, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (h, mi, sec) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&mo) || d < 1 || d > days_in_month(y, mo) || h > 23 || mi > 59 || sec > 59
    {
        return None;
    }
    Some(days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + sec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_instants() {
        // Values from the Unix epoch definition and well-known dates, not from this code.
        assert_eq!(format(0), "1970-01-01T00:00:00Z");
        assert_eq!(format(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(format(1_709_164_800), "2024-02-29T00:00:00Z");
        assert_eq!(parse("2024-02-29T00:00:00Z"), Some(1_709_164_800));
        assert_eq!(parse("1969-12-31T23:59:59Z"), Some(-1));
    }

    #[test]
    fn round_trips_across_centuries_and_leap_rules() {
        for t in [
            -62_167_219_200,
            -1,
            0,
            951_782_400,   // 2000-02-29
            4_107_542_400, // 2100-03-01 (2100 is not a leap year)
            253_402_300_799,
        ] {
            assert_eq!(parse(&format(t)), Some(t), "{t}");
        }
    }

    #[test]
    fn rejects_everything_but_the_one_form() {
        for bad in [
            "",
            "2024-02-29",
            "2024-02-29T00:00:00",
            "2024-02-29T00:00:00+00:00",
            "2024-02-29t00:00:00z",
            "2024-02-29T00:00:00.5Z",
            "2024-13-01T00:00:00Z",
            "2023-02-29T00:00:00Z",
            "2100-02-29T00:00:00Z",
            "2024-04-31T00:00:00Z",
            "2024-02-29T24:00:00Z",
            "2024-02-29T00:60:00Z",
            "2024-02-29T00:00:60Z",
            "+024-02-29T00:00:00Z",
        ] {
            assert_eq!(parse(bad), None, "{bad}");
        }
    }
}
