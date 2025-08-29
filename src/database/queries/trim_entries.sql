DELETE
FROM clipboard
WHERE id IN (SELECT id FROM clipboard ORDER BY last_updated ASC LIMIT ?)
