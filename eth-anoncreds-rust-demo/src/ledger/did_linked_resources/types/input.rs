pub struct ResourceInput {
    pub resource_name: String,
    pub resource_type: String,
    pub resource_version_id: String,
    pub media_type: String,
    pub content: Vec<u8>,
}
