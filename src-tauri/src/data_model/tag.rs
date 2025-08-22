use diesel::Queryable;

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, Queryable)]
#[diesel(table_name = tags)]
pub struct Tag {
    pub id: i64,
    pub name: String,
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Tag {}
