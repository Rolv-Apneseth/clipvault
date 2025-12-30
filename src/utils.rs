use std::{io::Cursor, time::{SystemTime, UNIX_EPOCH}};

use image::{DynamicImage, ImageReader};
use mime_sniffer::MimeTypeSniffer;
use unicode_segmentation::UnicodeSegmentation;

/// Returns the given number of bytes as a human-readable string representation.
#[must_use]
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
#[must_use]
pub fn truncate(mut s: String, max_graphemes: usize) -> String {
    if max_graphemes == 0 {
        s.clear();
        return s;
    } else if max_graphemes == 1 {
        s.drain(1..);
        return s;
    };

    let graphemes = s.graphemes(true).collect::<Vec<_>>();
    if graphemes.is_empty() || max_graphemes >= graphemes.len() {
        return s;
    }

    let max = max_graphemes - 1;
    graphemes.into_iter().take(max).chain(["…"]).collect()
}

/// Current Unix timestamp in seconds - based on system time.
#[must_use]
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

/// Attempt to parse image data from a byte slice.
///
/// Returns a tuple in the format `(mimetype, dynamic_image)`.
#[must_use]
pub fn decode_image(data: &[u8]) -> Option<(&'static str, DynamicImage)> {
    let img_reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let mimetype = img_reader.format()?.to_mime_type();
    let img = img_reader.decode().ok()?;

    Some((mimetype, img))
}

/// Guess the mimetype of data contained in a byte slice using `mime_sniffer`.
pub fn get_mimetype(data: &[u8]) -> Option<String> {
    data.sniff_mime_type().map(String::from)
}

#[cfg(test)]
mod test {
    use miette::miette;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_truncate() {
        // Character count only for basic Latin text - should be equivalent
        assert_eq!(truncate("abc".into(), 0).chars().count(), 0);
        assert_eq!(truncate("".into(), 17).chars().count(), 0);
        assert_eq!(truncate("d".into(), 1).chars().count(), 1);
        assert_eq!(truncate(";lakdf".into(), 1).chars().count(), 1);
        assert_eq!(truncate("ioiek".into(), 2).chars().count(), 2);
        assert_eq!(truncate("zcxvsd".into(), 3).chars().count(), 3);
        assert_eq!(truncate("/.l".into(), 5).chars().count(), 3);
        assert_eq!(
            truncate("alksdfaksldfklaslkfasfkskladfalk".into(), 7)
                .chars()
                .count(),
            7
        );
        assert_eq!(
            truncate("alksdfaksldfklaslkfasfkskladfalklsdkfks".into(), 18)
                .chars()
                .count(),
            18
        );
        assert_eq!(truncate("o".into(), 1), "o");

        // Graphemes
        assert_eq!(truncate("😀😀".into(), 5).graphemes(true).count(), 2);
        assert_eq!(truncate("😀🫡😀".into(), 2).graphemes(true).count(), 2);
        assert_eq!(truncate("ᚅ ᚆ ᚇ".into(), 4).graphemes(true).count(), 4);

        assert_eq!(truncate("ƀ Ɓ Ƃ ƃ Ƅ ƅ Ɔ Ƈ ƈ Ɖ Ɗ Ƌ ƌ".into(), 4), "ƀ Ɓ…");
        assert_eq!(truncate("й к л м н о п р с т у ф".into(), 8), "й к л м…");
        assert_eq!(truncate("ڠ ڡ ڢ ڣ ڤ ڥ ڦ ڧ ڨ".into(), 16), "ڠ ڡ ڢ ڣ ڤ ڥ ڦ ڧ…");
        assert_eq!(truncate("ᛦ ᛧ ᛨ ᛩ ᛪ ᛫ ᛬ ᛭ ᛮ ᛯ ᛰ ".into(), 6), "ᛦ ᛧ ᛨ…");
        assert_eq!(truncate("ᚅ ᚆ ᚇ".into(), 5), "ᚅ ᚆ ᚇ");
        assert_eq!(truncate("ㄱ ㄲ ㄳ ㄴ ㄵ ㄶ ㄷ ㄸ ㄹ".into(), 4), "ㄱ ㄲ…");
        assert_eq!(truncate("ポ マ ミ ム".into(), 6), "ポ マ ミ…");
        assert_eq!(truncate("🫰🫰🏿🫰🏻🫰🏽🫰🏼🫰🏾".into(), 6), "🫰🫰🏿🫰🏻🫰🏽🫰🏼🫰🏾");
        assert_eq!(truncate("👍👍🏾👍🏼👍🏿👍🏽👍🏻".into(), 5), "👍👍🏾👍🏼👍🏿…");
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
