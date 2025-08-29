use std::{
    borrow::Cow,
    time::{SystemTime, UNIX_EPOCH},
};

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
pub fn truncate(s: &str, max_chars: usize) -> Cow<'_, str> {
    if max_chars <= 1 {
        return s[..max_chars].into();
    }

    if max_chars >= s.chars().count() {
        return s.into();
    }

    match s.char_indices().nth(max_chars.saturating_sub(1)) {
        None => Cow::from(s),
        Some((idx, _)) => Cow::Owned(format!("{}…", &s[..idx].trim_end())),
    }
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
        assert_eq!(truncate("abc", 0).chars().count(), 0);
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
