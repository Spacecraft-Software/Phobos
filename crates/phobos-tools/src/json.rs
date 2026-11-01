// SPDX-FileCopyrightText: 2026 Mohamed Hammad <Mohamed.Hammad@SpacecraftSoftware.org>
// SPDX-License-Identifier: GPL-3.0-or-later

//! JSON envelopes and integer conversion of Unix seconds to UTC civil dates.

use serde::Serialize;
use std::io::{self, Write};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
struct Metadata<'a> {
    tool: &'static str,
    version: &'static str,
    command: &'a str,
    timestamp: String,
}

#[derive(Debug, Serialize)]
struct Envelope<'a, T> {
    metadata: Metadata<'a>,
    data: T,
}

/// Writes one success envelope followed by a newline.
///
/// # Errors
/// Returns clock, serialization, or output errors.
pub(crate) fn write(out: &mut impl Write, command: &str, data: impl Serialize) -> io::Result<()> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_secs();
    let envelope = Envelope {
        metadata: Metadata {
            tool: "phobos-tools",
            version: env!("CARGO_PKG_VERSION"),
            command,
            timestamp: timestamp(seconds),
        },
        data,
    };
    write_line(out, &envelope)
}

/// Writes one error envelope followed by a newline.
///
/// # Errors
/// Returns serialization or output errors.
pub(crate) fn write_error(out: &mut impl Write, command: &str, message: &str) -> io::Result<()> {
    #[derive(Serialize)]
    struct Diagnostic<'a> {
        message: &'a str,
        command: &'a str,
    }
    #[derive(Serialize)]
    struct Failure<'a> {
        error: Diagnostic<'a>,
    }
    write_line(
        out,
        &Failure {
            error: Diagnostic { message, command },
        },
    )
}

fn write_line(out: &mut impl Write, data: &impl Serialize) -> io::Result<()> {
    serde_json::to_writer(&mut *out, data).map_err(io::Error::other)?;
    out.write_all(b"\n")
}

fn timestamp(seconds: u64) -> String {
    // Unix timestamps count SI seconds; civil UTC days here have 24 hours.
    const SECONDS_PER_MINUTE: u64 = 60;
    const MINUTES_PER_HOUR: u64 = 60;
    const HOURS_PER_DAY: u64 = 24;
    const SECONDS_PER_HOUR: u64 = SECONDS_PER_MINUTE * MINUTES_PER_HOUR;
    const SECONDS_PER_DAY: u64 = SECONDS_PER_HOUR * HOURS_PER_DAY;
    let (year, month, day) = civil_from_days(i128::from(seconds / SECONDS_PER_DAY));
    let hour = seconds % SECONDS_PER_DAY / SECONDS_PER_HOUR;
    let minute = seconds % SECONDS_PER_HOUR / SECONDS_PER_MINUTE;
    let second = seconds % SECONDS_PER_MINUTE;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

fn civil_from_days(days: i128) -> (i128, i128, i128) {
    // Hinnant's civil_from_days uses March-based Gregorian 400-year eras.
    // These constants encode the Gregorian leap rules and month polynomial.
    const EPOCH_SHIFT: i128 = 719_468;
    const YEARS_PER_ERA: i128 = 400;
    const DAYS_PER_ERA: i128 = 146_097;
    const DAYS_PER_YEAR: i128 = 365;
    const FOUR_YEAR_CORRECTION: i128 = 1_460;
    const CENTURY_CORRECTION: i128 = 36_524;
    const ERA_CORRECTION: i128 = DAYS_PER_ERA - 1;
    const YEARS_PER_LEAP: i128 = 4;
    const YEARS_PER_CENTURY: i128 = 100;
    const MONTH_NUMERATOR: i128 = 153;
    const MONTH_DENOMINATOR: i128 = 5;
    const MONTH_ROUNDING: i128 = 2;
    const MARCH_OFFSET: i128 = 3;
    const JANUARY_OFFSET: i128 = 9;
    const MARCH_MONTHS: i128 = 10;
    let shifted = days + EPOCH_SHIFT;
    let era = shifted.div_euclid(DAYS_PER_ERA);
    let day_of_era = shifted.rem_euclid(DAYS_PER_ERA);
    let year_of_era = (day_of_era - day_of_era / FOUR_YEAR_CORRECTION
        + day_of_era / CENTURY_CORRECTION
        - day_of_era / ERA_CORRECTION)
        / DAYS_PER_YEAR;
    let day_of_year = day_of_era
        - (DAYS_PER_YEAR * year_of_era + year_of_era / YEARS_PER_LEAP
            - year_of_era / YEARS_PER_CENTURY);
    let march_month = (MONTH_DENOMINATOR * day_of_year + MONTH_ROUNDING) / MONTH_NUMERATOR;
    let day =
        day_of_year - (MONTH_NUMERATOR * march_month + MONTH_ROUNDING) / MONTH_DENOMINATOR + 1;
    let month = march_month
        + if march_month < MARCH_MONTHS {
            MARCH_OFFSET
        } else {
            -JANUARY_OFFSET
        };
    let year = year_of_era + era * YEARS_PER_ERA + i128::from(month <= MONTH_ROUNDING);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{timestamp, write, write_error};
    use serde_json::{Value, json};
    use std::io;

    #[test]
    fn known_utc_timestamps() {
        for (seconds, expected) in [
            (0, "1970-01-01T00:00:00Z"),
            (951_782_400, "2000-02-29T00:00:00Z"),
            (1_793_449_200, "2026-10-31T12:20:00Z"),
            (86_399, "1970-01-01T23:59:59Z"),
            (86_400, "1970-01-02T00:00:00Z"),
            (4_107_542_400, "2100-03-01T00:00:00Z"),
        ] {
            assert_eq!(timestamp(seconds), expected);
        }
        assert!(timestamp(u64::MAX).ends_with('Z'));
    }

    #[test]
    fn envelopes_are_single_json_lines() {
        let mut out = Vec::new();
        write(&mut out, "wad ls", json!({"kind":"IWAD", "lumps":[]})).unwrap();
        assert_eq!(out.last(), Some(&b'\n'));
        assert_eq!(std::str::from_utf8(&out).unwrap().lines().count(), 1);
        let value: Value = serde_json::from_slice(&out).unwrap();
        assert_eq!(value["metadata"]["tool"], "phobos-tools");
        assert_eq!(value["metadata"]["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(value["metadata"]["command"], "wad ls");
        assert!(
            value["metadata"]["timestamp"]
                .as_str()
                .unwrap()
                .ends_with('Z')
        );
        assert_eq!(value["data"], json!({"kind":"IWAD", "lumps":[]}));
        out.clear();
        write_error(&mut out, "wad ls", "bad \"file\"\nname").unwrap();
        assert_eq!(
            serde_json::from_slice::<Value>(&out).unwrap(),
            json!({"error":{"message":"bad \"file\"\nname", "command":"wad ls"}})
        );
        assert_eq!(std::str::from_utf8(&out).unwrap().lines().count(), 1);
    }

    #[test]
    fn reports_output_errors() {
        let mut full = io::Cursor::new([0_u8; 0]);
        assert_eq!(
            write(&mut full, "wad ls", ()).unwrap_err().kind(),
            io::ErrorKind::Other
        );
        assert_eq!(
            write_error(&mut full, "wad ls", "error")
                .unwrap_err()
                .kind(),
            io::ErrorKind::Other
        );
    }
}
