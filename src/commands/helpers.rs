//! Small, pure helpers shared across the command modules.

use foukoapi::util::capitalize;

/// Turn `[(lang, desc), ...]` into the HashMap that `command_described_i18n` wants.
pub(crate) fn i18n_map(pairs: &[(&str, &str)]) -> std::collections::HashMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

/// Embed accent colour by level, matching the framework's default curve
/// but kept here so we can tweak it independently.
pub(crate) fn color_for_level(level: u32) -> u32 {
    match level {
        0..=2 => 0x7A5BE8,   // violet
        3..=6 => 0x5B8DEF,   // blue
        7..=14 => 0x00C2A8,  // teal
        15..=29 => 0xF59F00, // orange
        _ => 0xE02B6B,       // hot pink for the dedicated
    }
}

/// Human-readable process uptime, e.g. `2d 3h 4m 5s`.
pub(crate) fn uptime_string() -> String {
    let secs = super::BOT_START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_secs();
    let (d, h, m, s) = (
        secs / 86_400,
        (secs % 86_400) / 3_600,
        (secs % 3_600) / 60,
        secs % 60,
    );
    if d > 0 {
        format!("{d}d {h}h {m}m {s}s")
    } else if h > 0 {
        format!("{h}h {m}m {s}s")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}

/// Parse `+10`, `-5:30`, `0`, `+03:45` etc. into signed minutes, or `None`.
pub(crate) fn parse_utc_offset(spec: &str) -> Option<i32> {
    let s = spec.trim();
    if s.is_empty() {
        return None;
    }
    let (sign, rest) = match s.as_bytes()[0] {
        b'+' => (1_i32, &s[1..]),
        b'-' => (-1_i32, &s[1..]),
        _ => (1_i32, s),
    };
    let (hh, mm) = match rest.split_once(':') {
        Some((h, m)) => (h, m),
        None => (rest, "0"),
    };
    let hours: i32 = hh.parse().ok()?;
    let minutes: i32 = mm.parse().ok()?;
    if !(0..=23).contains(&hours) || !(0..=59).contains(&minutes) {
        return None;
    }
    let total = sign * (hours * 60 + minutes);
    if !(-12 * 60..=14 * 60).contains(&total) {
        return None;
    }
    Some(total)
}

/// Render a signed minute offset as `+10`, `-5:30`, `+00:00` etc.
pub(crate) fn format_offset(minutes: i32) -> String {
    let sign = if minutes < 0 { '-' } else { '+' };
    let m = minutes.abs();
    let hh = m / 60;
    let mm = m % 60;
    if mm == 0 {
        format!("{sign}{hh}")
    } else {
        format!("{sign}{hh}:{mm:02}")
    }
}

pub(crate) fn parse_dice(spec: &str) -> Option<(u32, u32)> {
    let s = spec.trim().to_ascii_lowercase();
    let (n_str, m_str) = s.split_once('d')?;
    let count: u32 = if n_str.is_empty() {
        1
    } else {
        n_str.parse().ok()?
    };
    let sides: u32 = m_str.parse().ok()?;
    Some((count, sides))
}

/// Parse `10s`, `5m`, `2h`, or a bare number (seconds) into seconds.
pub(crate) fn parse_duration(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num, mult) = match s.chars().last()? {
        's' => (&s[..s.len() - 1], 1),
        'm' => (&s[..s.len() - 1], 60),
        'h' => (&s[..s.len() - 1], 3600),
        c if c.is_ascii_digit() => (s, 1),
        _ => return None,
    };
    // checked_mul: a huge number times 3600 must not wrap around.
    num.parse::<u64>().ok().and_then(|n| n.checked_mul(mult))
}

/// Render a second count as a compact "2h 3m" / "45s" string.
pub(crate) fn human_duration(secs: i64, lang: &str) -> String {
    let secs = secs.max(0);
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    let (hu, mu, su) = if lang == "ru" {
        ("ч", "мин", "сек")
    } else {
        ("h", "m", "s")
    };
    if h > 0 {
        format!("{h}{hu} {m}{mu}")
    } else if m > 0 {
        format!("{m}{mu} {s}{su}")
    } else {
        format!("{s}{su}")
    }
}

/// Turn a `"platform:id"` identity into something readable. We don't have
/// display names across platforms, so we show the platform and a shortened
/// id.
pub(crate) fn pretty_identity(primary: &str) -> String {
    let (platform, id) = primary.split_once(':').unwrap_or(("", primary));
    let short: String = id.chars().take(6).collect();
    let short = if id.chars().count() > 6 {
        format!("{short}...")
    } else {
        short
    };
    if platform.is_empty() {
        short
    } else {
        format!("{} {short}", capitalize(platform))
    }
}

/// Pull a target user id out of a command argument: a Discord mention
/// (`<@123>` / `<@!123>`), a `@name`-stripped token, or a bare id. Only
/// the first whitespace token counts, so trailing words don't leak into
/// the id. Returns `None` for an empty argument so callers default to
/// the invoker.
pub(crate) fn parse_target(arg: &str) -> Option<String> {
    let first = arg.split_whitespace().next()?;
    let t = first
        .trim_start_matches("<@")
        .trim_start_matches('!')
        .trim_end_matches('>')
        .trim_start_matches('@')
        .trim();
    if t.is_empty() {
        None
    } else {
        Some(t.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dice_specs() {
        assert_eq!(parse_dice("3d6"), Some((3, 6)));
        assert_eq!(parse_dice("d20"), Some((1, 20)));
        assert_eq!(parse_dice("  2D10 "), Some((2, 10)));
        assert_eq!(parse_dice("nonsense"), None);
        assert_eq!(parse_dice("3x6"), None);
    }

    #[test]
    fn utc_offsets_round_trip() {
        assert_eq!(parse_utc_offset("+10"), Some(600));
        assert_eq!(parse_utc_offset("-5:30"), Some(-330));
        assert_eq!(parse_utc_offset("0"), Some(0));
        assert_eq!(parse_utc_offset("+14"), Some(840));
        // Out of the -12..=+14 range Telegram/real-world offsets allow.
        assert_eq!(parse_utc_offset("+15"), None);
        assert_eq!(parse_utc_offset("garbage"), None);

        assert_eq!(format_offset(600), "+10");
        assert_eq!(format_offset(-330), "-5:30");
        assert_eq!(format_offset(0), "+0");
    }

    #[test]
    fn durations_parse() {
        assert_eq!(parse_duration("30s"), Some(30));
        assert_eq!(parse_duration("5m"), Some(300));
        assert_eq!(parse_duration("2h"), Some(7200));
        assert_eq!(parse_duration("45"), Some(45));
        assert_eq!(parse_duration(""), None);
        assert_eq!(parse_duration("10x"), None);
    }

    #[test]
    fn durations_render_compact() {
        assert_eq!(human_duration(45, "en"), "45s");
        assert_eq!(human_duration(90, "en"), "1m 30s");
        assert_eq!(human_duration(3_661, "en"), "1h 1m");
        assert_eq!(human_duration(-5, "en"), "0s"); // clamped
    }

    #[test]
    fn identity_is_readable() {
        assert_eq!(pretty_identity("discord:123"), "Discord 123");
        assert_eq!(pretty_identity("telegram:1234567890"), "Telegram 123456...");
        assert_eq!(pretty_identity("bare"), "bare");
    }

    #[test]
    fn level_colour_bands() {
        // Just make sure the bands don't panic and cover the ranges.
        assert_eq!(color_for_level(0), 0x7A5BE8);
        assert_eq!(color_for_level(10), 0x00C2A8);
        assert_eq!(color_for_level(999), 0xE02B6B);
    }
}
