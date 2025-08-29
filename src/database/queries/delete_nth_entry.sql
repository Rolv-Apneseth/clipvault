DELETE
FROM clipboard
WHERE
    id = (SELECT id FROM clipboard ORDER BY last_updated DESC LIMIT 1 OFFSET ?)
