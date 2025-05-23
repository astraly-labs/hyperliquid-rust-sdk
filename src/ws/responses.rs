use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponse {
    pub channel: String,
    pub data: PostResponseData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponseData {
    pub id: u64,
    pub response: PostResponseDataResponse,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponseDataResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub payload: PostResponsePayload,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponsePayload {
    pub status: String,
    pub response: PostResponsePayloadResponse,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponsePayloadResponse {
    #[serde(rename = "type")]
    pub response_type: String,
    pub data: Option<PostResponsePayloadData>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponsePayloadData {
    pub statuses: Vec<PostResponseStatus>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct PostResponseStatus {
    pub error: Option<String>,
    pub filled: Option<FilledStatus>,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilledStatus {
    pub total_sz: String,
    pub avg_px: String,
    pub oid: u64,
}
