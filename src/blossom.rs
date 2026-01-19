use base64::Engine;
use nostr_sdk::prelude::*;
use reqwest::Client as HttpClient;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub async fn blossom_mirror_core(
    server_url: &str,
    nsec: &str,
    url: &str,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let _client = crate::utils::get_client_with_signer(nsec, &[], client_nsec).await?;
    let http_client = get_http_client()?;

    let path = url.split('?').next().unwrap_or(url).trim_end_matches('/');
    let filename = path
        .split('/')
        .last()
        .ok_or_else(|| format!("Invalid URL path: {}", url))?;
    let sha256 = filename
        .split('.')
        .next()
        .ok_or_else(|| format!("Invalid filename: {}", filename))?;

    if sha256.len() != 64 || !sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "Could not extract a valid 64-character SHA-256 hex hash from URL: {}",
            url
        ));
    }

    let blossom_url = format!(
        "{}/{}?xs={}",
        server_url.trim_end_matches('/'),
        sha256,
        urlencoding::encode(url)
    );

    let blossom_resp = http_client
        .get(&blossom_url)
        .send()
        .await
        .map_err(|e| format!("Failed to trigger Blossom proxy: {}", e))?;

    if !blossom_resp.status().is_success() {
        return Err(format!(
            "Blossom server returned error: {}",
            blossom_resp.status()
        ));
    }

    Ok(format!("{}/{}", server_url.trim_end_matches('/'), sha256))
}

pub async fn blossom_upload_core(
    server_url: &str,
    nsec: &str,
    file_path: &str,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let nostr_client = crate::utils::get_client_with_signer(nsec, &[], client_nsec).await?;
    let http_client = get_http_client()?;

    let file_content = std::fs::read(file_path).map_err(|e| e.to_string())?;
    let sha256 = hash_sha256(&file_content);

    let head_check = http_client
        .head(format!("{}/{}", server_url.trim_end_matches('/'), sha256))
        .send()
        .await;

    if let Ok(resp) = head_check
        && resp.status().is_success()
    {
        return Ok(format!("{}/{}", server_url.trim_end_matches('/'), sha256));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let tags = vec![
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("t")),
            vec!["upload".to_string()],
        ),
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("x")),
            vec![sha256.clone()],
        ),
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("expiration")),
            vec![(now + 60).to_string()],
        ),
    ];

    let builder = EventBuilder::new(Kind::from(24242), "Upload Blob").tags(tags);
    let event = nostr_client
        .sign_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;
    let auth_b64 = base64::engine::general_purpose::STANDARD
        .encode(serde_json::to_string(&event).map_err(|e| e.to_string())?);

    let response = http_client
        .put(format!("{}/upload", server_url.trim_end_matches('/')))
        .header("Authorization", format!("Nostr {}", auth_b64))
        .body(file_content)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Blossom server returned error: {}",
            response.status()
        ));
    }

    let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    if let Some(url) = result["url"].as_str() {
        Ok(url.to_string())
    } else {
        Ok(format!("{}/{}", server_url.trim_end_matches('/'), sha256))
    }
}

pub async fn blossom_upload_content_core(
    server_url: &str,
    nsec: &str,
    content: String,
    filename: Option<String>,
    media_type: Option<String>,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let nostr_client = crate::utils::get_client_with_signer(nsec, &[], client_nsec).await?;
    let http_client = get_http_client()?;

    let file_content = content.into_bytes();
    let sha256 = hash_sha256(&file_content);

    let head_check = http_client
        .head(format!("{}/{}", server_url.trim_end_matches('/'), sha256))
        .send()
        .await;

    if let Ok(resp) = head_check
        && resp.status().is_success()
    {
        return Ok(format!("{}/{}", server_url.trim_end_matches('/'), sha256));
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut tags = vec![
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("t")),
            vec!["upload".to_string()],
        ),
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("x")),
            vec![sha256.clone()],
        ),
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("expiration")),
            vec![(now + 60).to_string()],
        ),
    ];

    if let Some(name) = filename {
        tags.push(Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("name")),
            vec![name],
        ));
    }

    if let Some(m) = media_type {
        tags.push(Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("m")),
            vec![m],
        ));
    }

    let builder = EventBuilder::new(Kind::from(24242), "Upload Blob").tags(tags);
    let event = nostr_client
        .sign_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;
    let auth_b64 = base64::engine::general_purpose::STANDARD
        .encode(serde_json::to_string(&event).map_err(|e| e.to_string())?);

    let response = http_client
        .put(format!("{}/upload", server_url.trim_end_matches('/')))
        .header("Authorization", format!("Nostr {}", auth_b64))
        .body(file_content)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Blossom server returned error: {}",
            response.status()
        ));
    }

    let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    if let Some(url) = result["url"].as_str() {
        Ok(url.to_string())
    } else {
        Ok(format!("{}/{}", server_url.trim_end_matches('/'), sha256))
    }
}

fn hash_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

pub async fn blossom_get_blob_core(server_url: &str, hash: &str) -> Result<Vec<u8>, String> {
    let http_client = get_http_client()?;
    let response = http_client
        .get(format!("{}/{}", server_url.trim_end_matches('/'), hash))
        .header("Accept-Encoding", "identity")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Blossom server returned error: {}",
            response.status()
        ));
    }

    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    Ok(bytes.to_vec())
}

pub async fn blossom_delete_core(
    server_url: &str,
    nsec: &str,
    hash: &str,
    client_nsec: Option<&str>,
) -> Result<(), String> {
    let nostr_client = crate::utils::get_client_with_signer(nsec, &[], client_nsec).await?;
    let http_client = get_http_client()?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let tags = vec![
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("t")),
            vec!["delete".to_string()],
        ),
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("x")),
            vec![hash.to_string()],
        ),
        Tag::custom(
            TagKind::Custom(std::borrow::Cow::Borrowed("expiration")),
            vec![(now + 60).to_string()],
        ),
    ];

    let builder = EventBuilder::new(Kind::from(24242), "Delete Blob").tags(tags);
    let event = nostr_client
        .sign_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;
    let auth_b64 = base64::engine::general_purpose::STANDARD
        .encode(serde_json::to_string(&event).map_err(|e| e.to_string())?);

    let response = http_client
        .delete(format!("{}/{}", server_url.trim_end_matches('/'), hash))
        .header("Authorization", format!("Nostr {}", auth_b64))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Blossom server returned error: {}",
            response.status()
        ));
    }

    Ok(())
}

pub async fn blossom_list_core(
    server_url: &str,
    pubkey: &str,
    nsec: Option<&str>,
    client_nsec: Option<&str>,
) -> Result<serde_json::Value, String> {
    let http_client = get_http_client()?;
    let url = format!("{}/list/{}", server_url.trim_end_matches('/'), pubkey);

    let mut request = http_client.get(&url);

    if let Some(n) = nsec {
        let nostr_client = crate::utils::get_client_with_signer(n, &[], client_nsec).await?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let tags = vec![
            Tag::custom(
                TagKind::Custom(std::borrow::Cow::Borrowed("t")),
                vec!["list".to_string()],
            ),
            Tag::custom(
                TagKind::Custom(std::borrow::Cow::Borrowed("expiration")),
                vec![(now + 60).to_string()],
            ),
        ];

        let builder = EventBuilder::new(Kind::from(24242), "List Blobs").tags(tags);
        let event = nostr_client
            .sign_event_builder(builder)
            .await
            .map_err(|e| e.to_string())?;
        let auth_b64 = base64::engine::general_purpose::STANDARD
            .encode(serde_json::to_string(&event).map_err(|e| e.to_string())?);
        request = request.header("Authorization", format!("Nostr {}", auth_b64));
    }

    let response = request.send().await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!(
            "Blossom server returned error: {}",
            response.status()
        ));
    }

    let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
    Ok(result)
}

fn get_http_client() -> Result<HttpClient, String> {
    HttpClient::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(300))
        .tcp_keepalive(Some(Duration::from_secs(60)))
        .pool_idle_timeout(Some(Duration::from_secs(90)))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}
