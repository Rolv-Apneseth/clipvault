use std::{
    io::{Cursor, Write, stdout},
    path::Path,
};

use content_inspector::ContentType;
use image::{GenericImageView, ImageReader};
use miette::{Context, IntoDiagnostic, Result};
use mime_sniffer::MimeTypeSniffer;

use super::SEPARATOR;

use crate::{
    cli::ListArgs,
    database::{init_db, queries::get_all_entries},
    utils::{human_bytes, ignore_broken_pipe, truncate},
};

fn preview_image(data: &[u8]) -> Option<String> {
    let img_reader = ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .ok()?;
    let format = img_reader.format()?;
    let img = img_reader.decode().ok()?;
    let (width, height) = img.dimensions();

    Some(format!(
        "[[ binary data {} {} {width}x{height} ]]",
        human_bytes(data.len()),
        format.to_mime_type(),
    ))
}

fn get_mimemtype(data: &[u8]) -> Option<String> {
    data.sniff_mime_type().map(String::from)
}

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
fn preview(id: u64, data: &[u8], width: usize) -> String {
    let data_type = content_inspector::inspect(data);
    let s = match data_type {
        ContentType::BINARY => {
            // More details for image types
            if let Some(img_msg) = preview_image(data) {
                img_msg
            }
            // Try and parse mime-type for other binary data
            else if let Some(mimetype) = get_mimemtype(data) {
                format!("[[ binary data {mimetype}]]")
            } else {
                "[[ binary data ]]".into()
            }
        }
        ContentType::UTF_8 | ContentType::UTF_8_BOM => preview_text(data, width),
        _ => "[[ Non-UTF-8 text ]]".into(),
    };

    format!("{id}{SEPARATOR}{s}")
}

#[tracing::instrument(skip(path_db))]
pub fn execute(path_db: &Path, args: ListArgs) -> Result<()> {
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

    // Combine previews into a single string so that all the output can be written to STDOUT at the same time
    let output = entries
        .into_iter()
        .map(|entry| preview(entry.id, &entry.content, preview_width))
        .collect::<Vec<_>>()
        .join("\n");

    let mut stdout = stdout().lock();
    ignore_broken_pipe(writeln!(&mut stdout, "{output}",))
        .into_diagnostic()
        .context("failed to write to STDOUT")?;
    ignore_broken_pipe(stdout.flush())
        .into_diagnostic()
        .context("failed to flush STDOUT")?;

    Ok(())
}
