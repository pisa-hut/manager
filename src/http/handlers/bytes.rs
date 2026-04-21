//! Shared helpers for serving byte blobs (ETag, sha256).

use axum::{
    body::Bytes,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
};
use sha2::{Digest, Sha256};

pub fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Build an HTTP response for a byte blob with ETag. Honors `If-None-Match`
/// by returning 304 when the client's tag matches.
pub fn build_blob_response(
    req_headers: &HeaderMap,
    content: Vec<u8>,
    sha256: &str,
) -> axum::response::Response {
    let etag = format!("\"{sha256}\"");

    if let Some(inm) = req_headers.get(header::IF_NONE_MATCH)
        && inm.to_str().map(|v| v.trim() == etag).unwrap_or(false)
    {
        let mut resp = StatusCode::NOT_MODIFIED.into_response();
        resp.headers_mut()
            .insert(header::ETAG, HeaderValue::from_str(&etag).unwrap());
        return resp;
    }

    let mut headers = HeaderMap::new();
    headers.insert(header::ETAG, HeaderValue::from_str(&etag).unwrap());
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    (headers, Bytes::from(content)).into_response()
}
