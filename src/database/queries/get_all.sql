SELECT id, substr (content, 1, ?) AS content, last_updated
FROM clipboard
ORDER BY last_updated DESC
