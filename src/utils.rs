use std::{
    borrow::Cow,
    time::{SystemTime, UNIX_EPOCH},
};

use unicode_segmentation::UnicodeSegmentation;

/// Returns the given number of bytes as a human-readable string representation.
pub fn human_bytes(mut bytes: usize) -> String {
    let unit = if bytes < 1_000 {
        "B"
    } else if bytes < 1_000_000 {
        bytes /= 1_000;
        "kB"
    } else if bytes < 1_000_000_000 {
        bytes /= 1_000_000;
        "MB"
    } else {
        bytes /= 1_000_000_000;
        "GB"
    };

    format!("{bytes}{unit}")
}

/// Truncates a string to the given number of characters.
pub fn truncate(s: &str, max_graphemes: usize) -> Cow<'_, str> {
    let graphemes = s.graphemes(true).collect::<Vec<_>>();

    if graphemes.is_empty() || max_graphemes >= graphemes.len() {
        return Cow::from(s);
    }

    if max_graphemes <= 1 {
        return graphemes[..max_graphemes].join("").into();
    }

    let max = max_graphemes - 1;
    Cow::Owned(format!("{}…", graphemes[..max].join("")))
}

/// Current Unix timestamp in seconds - based on system time.
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should go forward - problem with system clock")
        .as_secs()
}

/// Ignore broken pipe IO errors.
///
/// See <https://users.rust-lang.org/t/broken-pipe-when-attempt-to-write-to-stdout/111186>
pub fn ignore_broken_pipe(res: std::io::Result<()>) -> std::io::Result<()> {
    match res {
        Err(e) if e.kind() != std::io::ErrorKind::BrokenPipe => Err(e),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod test {
    use miette::miette;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_truncate() {
        // Character count only for basic Latin text - should be equivalent
        assert_eq!(truncate("abc", 0).chars().count(), 0);
        assert_eq!(truncate("", 17).chars().count(), 0);
        assert_eq!(truncate(";lakdf", 1).chars().count(), 1);
        assert_eq!(truncate("ioiek", 2).chars().count(), 2);
        assert_eq!(truncate("zcxvsd", 3).chars().count(), 3);
        assert_eq!(truncate("/.l", 5).chars().count(), 3);
        assert_eq!(
            truncate("alksdfaksldfklaslkfasfkskladfalk", 7)
                .chars()
                .count(),
            7
        );
        assert_eq!(
            truncate("alksdfaksldfklaslkfasfkskladfalklsdkfks", 18)
                .chars()
                .count(),
            18
        );

        // Graphemes
        assert_eq!(truncate("😀😀", 5).graphemes(true).count(), 2);
        assert_eq!(truncate("😀🫡😀", 2).graphemes(true).count(), 2);
        assert_eq!(truncate("ᚅ ᚆ ᚇ", 4).graphemes(true).count(), 4);

        assert_eq!(truncate("ƀ Ɓ Ƃ ƃ Ƅ ƅ Ɔ Ƈ ƈ Ɖ Ɗ Ƌ ƌ", 4), "ƀ Ɓ…");
        assert_eq!(truncate("й к л м н о п р с т у ф", 8), "й к л м…");
        assert_eq!(truncate("ڠ ڡ ڢ ڣ ڤ ڥ ڦ ڧ ڨ", 16), "ڠ ڡ ڢ ڣ ڤ ڥ ڦ ڧ…");
        assert_eq!(truncate("ᛦ ᛧ ᛨ ᛩ ᛪ ᛫ ᛬ ᛭ ᛮ ᛯ ᛰ ", 6), "ᛦ ᛧ ᛨ…");
        assert_eq!(truncate("ᚅ ᚆ ᚇ", 5), "ᚅ ᚆ ᚇ");
        assert_eq!(truncate("ㄱ ㄲ ㄳ ㄴ ㄵ ㄶ ㄷ ㄸ ㄹ", 4), "ㄱ ㄲ…");
        assert_eq!(truncate("ポ マ ミ ム", 6), "ポ マ ミ…");
        assert_eq!(truncate("🫰🫰🏿🫰🏻🫰🏽🫰🏼🫰🏾", 6), "🫰🫰🏿🫰🏻🫰🏽🫰🏼🫰🏾");
        assert_eq!(truncate("👍👍🏾👍🏼👍🏿👍🏽👍🏻", 5), "👍👍🏾👍🏼👍🏿…");
    }

    #[test]
    fn test_human_bytes() {
        assert_eq!(human_bytes(0), String::from("0B"));
        assert_eq!(human_bytes(10), String::from("10B"));
        assert_eq!(human_bytes(1_000), String::from("1kB"));
        assert_eq!(human_bytes(9_999), String::from("9kB"));
        assert_eq!(human_bytes(999_999), String::from("999kB"));
        assert_eq!(human_bytes(1_000_000), String::from("1MB"));
        assert_eq!(human_bytes(8_200_000), String::from("8MB"));
        assert_eq!(human_bytes(175_500_000), String::from("175MB"));
        assert_eq!(human_bytes(1_000_000_000), String::from("1GB"));
        assert_eq!(human_bytes(2_000_000_000), String::from("2GB"));
    }

    #[test]
    fn test_ignore_broken_pipe() {
        use std::io::{Error, ErrorKind};

        assert!(ignore_broken_pipe(Err(Error::new(ErrorKind::NotFound, miette!("")))).is_err());
        assert!(
            ignore_broken_pipe(Err(Error::new(ErrorKind::AlreadyExists, miette!("")))).is_err()
        );
        assert!(ignore_broken_pipe(Err(Error::new(ErrorKind::BrokenPipe, miette!("")))).is_ok());
    }
}
