SELECT (freelist_count * page_size) AS freelist_size
FROM pragma_freelist_count, pragma_page_size
