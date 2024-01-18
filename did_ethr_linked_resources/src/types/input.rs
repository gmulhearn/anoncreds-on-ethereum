#[derive(Debug, Clone)]
pub struct ResourceInput {
    pub resource_name: String,
    pub resource_type: String,
    pub resource_version_id: String,
    pub media_type: String,
    pub content: Vec<u8>,
}

impl Default for ResourceInput {
    fn default() -> Self {
        Self {
            resource_name: Default::default(),
            resource_type: Default::default(),
            resource_version_id: Default::default(),
            media_type: Default::default(),
            content: Default::default(),
        }
    }
}
