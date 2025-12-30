SELECT
    id,
    last_updated,
    substr (content, 1, ?) AS content,
    octet_length (content) AS content_size,
    content_type,
    mimetype,
    extra_preview_data
FROM clipboard
ORDER BY last_updated DESC
