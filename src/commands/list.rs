use std::{io::{BufWriter, Write, stdout}, path::Path};

use content_inspector::ContentType;
use image::GenericImageView;
use miette::{Context, IntoDiagnostic, Result};

use super::SEPARATOR;
use crate::{cli::ListArgs, database::{data::ClipboardEntry, init_db, queries::get_all_entries}, utils::{decode_image, get_mimetype, human_bytes, ignore_broken_pipe, truncate}};

#[tracing::instrument()]
fn preview_text(entry: &ClipboardEntry) -> String {
    let mut result = String::with_capacity(entry.content.len());
    String::from_utf8_lossy(&entry.content)
        .split_whitespace()
        .for_each(|w| {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(w);
        });

    result
}

#[tracing::instrument()]
fn preview_binary(entry: &ClipboardEntry) -> String {
    // Human readable representation of the byte count of the binary data
    let byte_count = human_bytes(entry.content_size);

    let Some(mimetype) = entry.mimetype.as_ref() else {
        return format!("[[ binary data {byte_count} ]]");
    };

    if let Some(extra_preview_data) = entry.extra_preview_data.as_ref() {
        format!("[[ binary data {byte_count} {mimetype} {extra_preview_data} ]]")
    } else {
        format!("[[ binary data {byte_count} {mimetype} ]]")
    }
}

#[tracing::instrument()]
fn preview(mut entry: ClipboardEntry, width: usize) -> String {
    // Fallback for entries created with clipvault v1.1.1 and older, which would not
    // have some of the additional preview data stored in the DB
    let content_type = entry.content_type.unwrap_or_else(|| {
        let content_type = content_inspector::inspect(&entry.content);
        entry.content_type = Some(content_type);

        if content_type.is_binary() {
            if let Some((img_mimetype, img)) = decode_image(&entry.content) {
                let (w, h) = img.dimensions();
                entry.extra_preview_data = Some(format!("{w}x{h}"));
                entry.mimetype = Some(img_mimetype.into());
            } else if let Some(content_mimetype) = get_mimetype(&entry.content) {
                entry.mimetype = Some(content_mimetype);
            }
        }

        content_type
    });

    let s = match content_type {
        ContentType::BINARY => preview_binary(&entry),
        ContentType::UTF_8 => preview_text(&entry),
        ContentType::UTF_8_BOM => {
            // Remove BOM so remaining data can be parsed as regular UTF-8
            entry.content.drain(..3);
            preview_text(&entry)
        }
        _ => String::from("[[ Non-UTF-8 text ]]"),
    };
    let truncated = truncate(s, width);

    format!("{}{SEPARATOR}{truncated}", entry.id)
}

#[tracing::instrument(skip(path_db))]
fn execute_inner(path_db: &Path, args: ListArgs, show_output: bool) -> Result<()> {
    let ListArgs {
        max_preview_width,
        reverse,
    } = args;

    let preview_width = if max_preview_width == 0 {
        tracing::debug!("preview width limit disabled");
        usize::MAX
    } else {
        max_preview_width
    };

    // Database only needed to get the entries - avoid locking
    let entries = {
        let conn = init_db(path_db)?;
        let mut entries = get_all_entries(&conn, preview_width)?;
        if reverse {
            entries.reverse();
        }

        entries
    };
    tracing::debug!("entries count: {}", entries.len());

    if entries.is_empty() {
        return Ok(());
    }

    let stdout = stdout();
    let stdout = stdout.lock();

    // [`BufWriter`] for more efficient, buffered writes
    let mut writer = BufWriter::with_capacity(8 * 1024, stdout);

    for entry in entries
        .into_iter()
        .map(|entry| preview(entry, preview_width))
    {
        if show_output {
            writer
                .write(&entry.into_bytes())
                .into_diagnostic()
                .context("failed to write to STDOUT")?;
            writer
                .write(b"\n")
                .into_diagnostic()
                .context("failed to write to STDOUT")?;
        }
    }

    ignore_broken_pipe(writer.flush())
        .into_diagnostic()
        .context("failed to flush STDOUT")?;

    Ok(())
}

#[tracing::instrument(skip(path_db))]
pub fn execute(path_db: &Path, args: ListArgs) -> Result<()> {
    execute_inner(path_db, args, true)
}

#[doc(hidden)]
#[tracing::instrument(skip(path_db))]
pub fn execute_without_output(path_db: &Path, args: ListArgs) -> Result<()> {
    assert!(
        !cfg!(debug_assertions),
        "Not intended to run in production code"
    );
    execute_inner(path_db, args, false)
}
