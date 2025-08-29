pub mod clear;
pub mod delete;
pub mod get;
pub mod list;
pub mod store;

use miette::{Context, IntoDiagnostic, Result};

pub(super) const SEPARATOR: &str = "\t";

pub(super) fn extract_id(input: String) -> Result<u64> {
    let str = match input.split_once(SEPARATOR) {
        Some((s, _)) => s,
        None => input.trim(),
    };
    str.parse().into_diagnostic().context("failed to parse ID")
}

pub(super) fn wrap_index(len: usize, index: isize) -> usize {
    assert!(len > 0);

    if index < 0 {
        let diff = index.saturating_add_unsigned(len) % len.cast_signed();
        if diff < 0 {
            len - diff.unsigned_abs()
        } else {
            diff.unsigned_abs()
        }
    } else {
        index.unsigned_abs() % len
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use proptest::prelude::*;

    #[test]
    fn test_extract_id() {
        assert!(extract_id("-7".into()).is_err());
        assert!(extract_id("alb".into()).is_err());
        assert!(extract_id("😔".into()).is_err());
        assert!(extract_id("-".into()).is_err());
        assert!(extract_id("one".into()).is_err());
        assert!(extract_id("t1".into()).is_err());
        assert!(extract_id("\t7".into()).is_err());
        assert!(extract_id("13\nabc".into()).is_err());

        assert_eq!(extract_id("2".into()).unwrap(), 2);
        assert_eq!(extract_id("7".into()).unwrap(), 7);
        assert_eq!(extract_id("7".into()).unwrap(), 7);
        assert_eq!(extract_id("18".into()).unwrap(), 18);
        assert_eq!(extract_id("107898".into()).unwrap(), 107_898);

        assert_eq!(extract_id("1\ttesting".into()).unwrap(), 1);
        assert_eq!(extract_id("3\tabc".into()).unwrap(), 3);
        assert_eq!(extract_id("100\tcopy\nthis".into()).unwrap(), 100);

        assert_eq!(extract_id("13\t\t\t12".into()).unwrap(), 13);
        assert_eq!(extract_id("17\t15".into()).unwrap(), 17);
    }

    #[test]
    fn test_wrap_index() {
        // Only one item available
        assert_eq!(wrap_index(1, 5), 0);
        assert_eq!(wrap_index(1, -7), 0);
        assert_eq!(wrap_index(1, isize::MIN), 0);
        assert_eq!(wrap_index(1, isize::MAX), 0);

        assert_eq!(wrap_index(2, -1), 1);
        assert_eq!(wrap_index(2, -2), 0);
        assert_eq!(wrap_index(2, -14), 0);
        assert_eq!(wrap_index(2, 2), 0);
        assert_eq!(wrap_index(2, 4), 0);
        assert_eq!(wrap_index(2, -3), 1);

        assert_eq!(wrap_index(200, 0), 0);
        assert_eq!(wrap_index(200, -1), 199);
        assert_eq!(wrap_index(200, -2), 198);
        assert_eq!(wrap_index(200, 200), 0);
        assert_eq!(wrap_index(200, -200), 0);
        assert_eq!(wrap_index(200, -201), 199);
        assert_eq!(wrap_index(200, -202), 198);
    }

    proptest! {
        #[test]
        fn prop_test_wrap_index(len: usize, index: isize) {
            wrap_index(len, index);
        }
    }
}
