INSERT
INTO clipboard (content, last_updated)
VALUES (?, ?)
ON CONFLICT (content) DO UPDATE SET last_updated = excluded.last_updated
