use rusqlite::Row;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClipboardEntry {
    pub id: u64,
    pub content: Vec<u8>,
    pub last_updated: u64,
}

impl<'stmt> TryFrom<&Row<'stmt>> for ClipboardEntry {
    type Error = rusqlite::Error;
    fn try_from(row: &Row) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: row.get(0)?,
            content: row.get(1)?,
            last_updated: row.get(2)?,
        })
    }
}

impl Ord for ClipboardEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.last_updated.cmp(&other.last_updated)
    }
}

impl PartialOrd for ClipboardEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
