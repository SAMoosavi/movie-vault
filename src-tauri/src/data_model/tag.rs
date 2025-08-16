#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub color: String,
}

impl Ord for Tag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name
            .cmp(&other.name)
            .then_with(|| self.color.cmp(&other.color))
    }
}

impl PartialOrd for Tag {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.color == other.color
    }
}

impl Eq for Tag {}
