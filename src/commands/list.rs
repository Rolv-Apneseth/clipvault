use std::{
    io::{BufWriter, Cursor, Write, stdout},
    path::Path,
};

use content_inspector::ContentType;
use image::{DynamicImage, GenericImageView, ImageReader};
use miette::{Context, IntoDiagnostic, Result};
use mime_sniffer::MimeTypeSniffer;

use super::SEPARATOR;

use crate::{
    cli::ListArgs,
    database::{init_db, queries::get_all_entries},
    utils::{human_bytes, ignore_broken_pipe, truncate},
};

fn decode_image(data: &[u8]) -> Option<(&'static str, DynamicImage)> {
    let img_reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let mimetype = img_reader.format()?.to_mime_type();
    let img = img_reader.decode().ok()?;

    Some((mimetype, img))
}

fn get_mimetype(data: &[u8]) -> Option<String> {
    data.sniff_mime_type().map(String::from)
}

#[tracing::instrument(skip(data))]
fn preview_text(data: &[u8], width: usize) -> String {
    let mut result = String::with_capacity(data.len());
    String::from_utf8_lossy(data)
        .split_whitespace()
        .for_each(|w| {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(w);
        });

    truncate(&result, width).into_owned()
}

#[tracing::instrument(skip(data))]
fn preview_binary(data: &[u8], width: usize) -> String {
    // Early return if data won't be visible in preview anyway
    if width < "[[ binary data ".len() {
        return truncate("[[ binary data ", width).into_owned();
    };

    // Human readable representation of the byte count of the binary data
    let byte_count = human_bytes(data.len());

    // More details for image types
    let result = if let Some((mimetype, img)) = decode_image(data) {
        let (w, h) = img.dimensions();
        format!("[[ binary data {byte_count} {mimetype} {w}x{h} ]]")
    }
    // Try and parse mime-type for other binary data
    else if let Some(mimetype) = get_mimetype(data) {
        format!("[[ binary data {byte_count} {mimetype} ]]")
    } else {
        format!("[[ binary data {byte_count} ]]")
    };

    truncate(&result, width).into_owned()
}

#[tracing::instrument(skip(data))]
fn preview(id: u64, data: &[u8], width: usize) -> String {
    let data_type = content_inspector::inspect(data);
    let s = match data_type {
        ContentType::BINARY => preview_binary(data, width),
        ContentType::UTF_8 | ContentType::UTF_8_BOM => preview_text(data, width),
        _ => "[[ Non-UTF-8 text ]]".into(),
    };

    format!("{id}{SEPARATOR}{s}")
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
        .map(|entry| preview(entry.id, &entry.content, preview_width))
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
