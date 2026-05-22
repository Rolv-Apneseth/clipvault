use std::{io::{Read, stdin}, path::Path};

use content_inspector::ContentType;
use image::GenericImageView;
use miette::{Context, IntoDiagnostic, Result, bail};
use tracing::instrument;

use crate::{cli::StoreArgs, database::{data::ClipboardEntry, init_db, queries::{delete_all_entries, delete_entries_older_than, trim_entries, upsert_entry}}, utils::{decode_image, get_mimetype, now}, wayland::wlr_toplevel::get_active_toplevel};

#[instrument(skip(path_db, args))]
pub fn execute(path_db: &Path, args: StoreArgs) -> Result<()> {
    execute_with_source(path_db, args, stdin())
}

#[doc(hidden)]
#[instrument(skip(source))]
pub fn execute_with_source(path_db: &Path, args: StoreArgs, mut source: impl Read) -> Result<()> {
    let StoreArgs {
        max_entries,
        max_entry_age: max_age,
        max_entry_length: max_bytes,
        min_entry_length: min_bytes,
        store_sensitive,
        ignore_pattern,
        window_ignore_pattern,
    } = args;

    // Min conflicts with max
    if min_bytes > max_bytes {
        bail!("minimum entry length ({min_bytes}) exceeds maximum entry length ({max_bytes})")
    }

    // Set by `wl-clipboard`
    if let Ok(s) = std::env::var("CLIPBOARD_STATE") {
        tracing::debug!("CLIPBOARD_STATE={s}");
        match s.as_str() {
            // Clipboard contains a sensitive value - skip if not storing sensitive values
            // As of writing, the latest release of `wl-clipboard` does not include the changes for
            // marking sensitive values using x-kde-passwordManagerHint.
            "sensitive" if !store_sensitive => {
                tracing::trace!("sensitive - not storing");
                return Ok(());
            }
            // Clipboard explicitly cleared - clear history as well.
            // As of writing, "clear" is not yet used by `wl-clipboard`.
            "clear" => {
                tracing::debug!("explicitly cleared clipboard");
                return delete_all_entries(&init_db(path_db)?);
            }
            // Clipboard is empty - nothing to store
            "nil" => return Ok(()),
            _ => {}
        }
    };

    // Read input using given source - this should be STDIN for production code
    let buf = {
        let mut buf = vec![];
        source
            .read_to_end(&mut buf)
            .into_diagnostic()
            .context("failed to read from STDIN")?;
        buf
    };
    drop(source);

    if buf.is_empty() {
        tracing::trace!("no content to store");
        return Ok(());
    }

    // Ignore content outside of the min and max byte constraints
    let gt_max = buf.len() > max_bytes && max_bytes != 0;
    let lt_min = buf.len() < min_bytes;
    if gt_max || lt_min {
        tracing::debug!(
            "content length ({}) is outside the bounds {min_bytes}->{max_bytes}",
            buf.len()
        );
        return Ok(());
    }

    // Ignore purely whitespace content
    if buf.trim_ascii().is_empty() {
        tracing::debug!("only ASCII whitespace content");
        return Ok(());
    }

    // Check user-provided ignore patterns
    if let Some(regexes) = ignore_pattern
        && !regexes.is_empty()
        && matches!(
            content_inspector::inspect(&buf),
            ContentType::UTF_8 | ContentType::UTF_8_BOM
        )
        && regexes
            .iter()
            .any(|re| re.is_match(&String::from_utf8_lossy(&buf)))
    {
        tracing::debug!("content matched an ignore pattern");
        return Ok(());
    }

    // Check user-provided window ignore patterns
    if let Some(regexes) = window_ignore_pattern
        && !regexes.is_empty()
    {
        match get_active_toplevel() {
            Ok(Some(active_top_window)) => {
                let title = active_top_window
                    .title
                    .unwrap_or_else(|| String::from("[untitled]"));
                let app_id = active_top_window
                    .app_id
                    .unwrap_or_else(|| String::from("[unknown]"));
                let haystack = format!("{app_id}: {title}");

                tracing::debug!("focused window: ({haystack})");
                if regexes.iter().any(|re| re.is_match(&haystack)) {
                    tracing::debug!("Focused window ({haystack}) matched an ignore pattern");
                    return Ok(());
                } else {
                    tracing::trace!("Focused window ({haystack}) did not match any ignore pattern");
                }
            }
            Ok(None) => tracing::warn!("No focused window found"),
            Err(e) => tracing::error!("Failed to get the active toplevel: {e}"),
        }
    }

    // Only get DB connection after parsing STDIN - avoid locking
    let conn = &init_db(path_db)?;

    // Delete old entries
    let max_age = max_age.as_secs();
    if max_age != 0 {
        let timestamp = now() - max_age;
        delete_entries_older_than(conn, timestamp)?;
    }

    // Setup additional data to be stored for the entry
    let entry = {
        // Inspect the content type
        let content_type = content_inspector::inspect(&buf);

        // Store extra information for images and other binary data
        let (mut mimetype, mut extra_preview_data) = (None, None);
        if content_type.is_binary() {
            // Resolution and mimetype for images
            if let Some((img_mimetype, img)) = decode_image(&buf) {
                let (w, h) = img.dimensions();
                extra_preview_data = Some(format!("{w}x{h}"));
                mimetype = Some(img_mimetype.into());
            }
            // Only mimetype for other binary data (if detected)
            else if let Some(content_mimetype) = get_mimetype(&buf) {
                mimetype = Some(content_mimetype);
            }
        };

        ClipboardEntry {
            content: buf,
            content_type: Some(content_type),
            mimetype,
            extra_preview_data,
            ..Default::default()
        }
    };

    // Upsert new entry
    upsert_entry(conn, entry)?;

    // Trim entries if over limit
    if max_entries != 0 {
        trim_entries(conn, max_entries)?;
    }

    Ok(())
}
