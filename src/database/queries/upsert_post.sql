INSERT
INTO clipboard (
    last_updated, content, content_type, mimetype, extra_preview_data
)
VALUES (?, ?, ?, ?, ?)
ON CONFLICT (content) DO UPDATE SET last_updated = excluded.last_updated,
mimetype = excluded.mimetype,
extra_preview_data = excluded.extra_preview_data,
content_type = excluded.content_type
