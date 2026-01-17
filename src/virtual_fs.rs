use crate::models::{VirtualFilesystem, VirtualNode, FilesystemRequest, EventResponse};
use crate::utils::get_client_with_signer;
use nostr_sdk::prelude::*;
use base64::{Engine as _, engine::general_purpose};
use serde_cbor;
use chrono;

pub const KIND_VIRTUAL_FS: Kind = Kind::Custom(31925);

pub async fn publish_filesystem(
    nsec: &str,
    relays: &[String],
    req: FilesystemRequest,
    client_nsec: Option<&str>,
) -> Result<String, String> {
    let client = get_client_with_signer(nsec, relays, client_nsec).await?;

    let content_bytes = serde_cbor::to_vec(&req.filesystem)
        .map_err(|e| format!("Failed to encode filesystem: {}", e))?;

    let content_base64 = general_purpose::STANDARD.encode(content_bytes);

    let tags = vec![
        Tag::identifier(req.identifier),
        Tag::custom(TagKind::Custom("name".into()), vec![req.name]),
        Tag::custom(TagKind::Custom("version".into()), vec![req.filesystem.metadata.version]),
        Tag::custom(TagKind::Custom("max_depth".into()), vec![req.filesystem.metadata.max_depth.to_string()]),
        Tag::custom(TagKind::Custom("encoding".into()), vec!["cbor".to_string()]),
        Tag::custom(TagKind::Custom("size".into()), vec![req.filesystem.metadata.total_size.to_string()]),
        Tag::custom(TagKind::Custom("node_count".into()), vec![req.filesystem.metadata.total_nodes.to_string()]),
    ];

    let builder = EventBuilder::new(KIND_VIRTUAL_FS, content_base64).tags(tags);

    let output = client
        .send_event_builder(builder)
        .await
        .map_err(|e| e.to_string())?;

    Ok(output.id().to_string())
}

pub async fn fetch_filesystems(
    pubkey_str: &str,
    relays: &[String],
) -> Result<Vec<EventResponse>, String> {
    // For fetching public events, we use a random key if no nsec is provided
    let keys = Keys::generate();
    let client = Client::new(keys);
    for relay in relays {
        if let Ok(url) = RelayUrl::parse(relay) {
            client.add_relay(url).await.map_err(|e| e.to_string())?;
        }
    }
    client.connect().await;

    let pubkey = if pubkey_str.is_empty() {
         return Err("Public key is required for fetching".to_string());
    } else {
        PublicKey::from_hex(pubkey_str)
            .or_else(|_| PublicKey::from_bech32(pubkey_str))
            .map_err(|e| format!("Invalid public key: {}", e))?
    };

    let filter = Filter::new()
        .author(pubkey)
        .kind(KIND_VIRTUAL_FS);

    let events = client
        .fetch_events(filter, std::time::Duration::from_secs(10))
        .await
        .map_err(|e| e.to_string())?;

    let mut responses = Vec::new();
    for event in events.into_iter() {
        responses.push(EventResponse {
            id: event.id.to_string(),
            pubkey: event.pubkey.to_string(),
            created_at: event.created_at.as_secs() as i64,
            kind: event.kind.as_u16(),
            tags: event.tags.iter().map(|t| t.clone().to_vec()).collect(),
            content: event.content.to_string(),
            is_private: false,
            bech32: Some(event.id.to_bech32().unwrap_or_default()),
            private_tags: None,
            source: None,
        });
    }

    Ok(responses)
}

pub fn decode_filesystem(content_base64: &str) -> Result<VirtualFilesystem, String> {
    let content_bytes = general_purpose::STANDARD
        .decode(content_base64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    serde_cbor::from_slice(&content_bytes)
        .map_err(|e| format!("Failed to decode CBOR: {}", e))
}

impl VirtualNode {
    pub fn validate_depth(&self, max_depth: u8) -> Result<(), String> {
        match self {
            VirtualNode::File { depth, .. } => {
                if *depth > max_depth {
                    return Err(format!("Max depth {} exceeded at file", max_depth));
                }
                Ok(())
            }
            VirtualNode::Directory { depth, children, .. } => {
                if *depth > max_depth {
                    return Err(format!("Max depth {} exceeded at directory", max_depth));
                }
                for child in children {
                    child.validate_depth(max_depth)?;
                }
                Ok(())
            }
        }
    }

    pub fn find_node(&self, path: &[&str]) -> Option<&VirtualNode> {
        if path.is_empty() {
            return Some(self);
        }

        match self {
            VirtualNode::Directory { children, .. } => {
                let first = path[0];
                let rest = &path[1..];
                for child in children {
                    match child {
                        VirtualNode::File { name, .. } | VirtualNode::Directory { name, .. } => {
                            if name == first {
                                return child.find_node(rest);
                            }
                        }
                    }
                }
                None
            }
            VirtualNode::File { .. } => None,
        }
    }

    pub fn add_child(&mut self, node: VirtualNode) -> Result<(), String> {
        match self {
            VirtualNode::Directory { children, depth, .. } => {
                let child_depth = match &node {
                    VirtualNode::File { depth, .. } => *depth,
                    VirtualNode::Directory { depth, .. } => *depth,
                };

                if child_depth != *depth + 1 {
                    return Err(format!("Invalid child depth: expected {}, got {}", *depth + 1, child_depth));
                }

                children.push(node);
                Ok(())
            }
            VirtualNode::File { .. } => Err("Cannot add child to a file".to_string()),
        }
    }
}

impl VirtualFilesystem {
    pub fn create_directory_recursive(&mut self, path: &str) -> Result<(), String> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            return Ok(());
        }

        if parts.len() > self.metadata.max_depth as usize {
            return Err(format!("Path exceeds maximum depth of {}", self.metadata.max_depth));
        }

        let mut current_node = &mut self.root;
        let mut current_depth = 0;

        for part in parts {
            current_depth += 1;

            let found_index = match current_node {
                VirtualNode::Directory { children, .. } => {
                    children.iter().position(|child| match child {
                        VirtualNode::Directory { name, .. } => name == part,
                        _ => false,
                    })
                }
                _ => return Err("Encountered a file while traversing path".to_string()),
            };

            if let Some(index) = found_index {
                current_node = match current_node {
                    VirtualNode::Directory { children, .. } => &mut children[index],
                    _ => unreachable!(),
                };
            } else {
                let new_dir = VirtualNode::Directory {
                    name: part.to_string(),
                    depth: current_depth,
                    metadata: crate::models::DirectoryNodeMetadata {
                        created: chrono::Utc::now().timestamp(),
                        modified: chrono::Utc::now().timestamp(),
                        permissions: "755".to_string(),
                    },
                    children: Vec::new(),
                };

                match current_node {
                    VirtualNode::Directory { children, .. } => {
                        children.push(new_dir);
                        let last_idx = children.len() - 1;
                        current_node = &mut children[last_idx];
                    },
                    _ => unreachable!(),
                }
                self.metadata.total_nodes += 1;
            }
        }

        Ok(())
    }

    pub fn add_file(&mut self, path: &str, file: VirtualNode) -> Result<(), String> {
        let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if parts.is_empty() {
            // Add to root
            self.root.add_child(file)?;
            self.metadata.total_nodes += 1;
            return Ok(());
        }

        self.create_directory_recursive(path)?;

        let mut current_node = &mut self.root;
        for part in parts {
            current_node = match current_node {
                VirtualNode::Directory { children, .. } => {
                    let idx = children.iter().position(|c| match c {
                        VirtualNode::Directory { name, .. } => name == part,
                        _ => false,
                    }).ok_or("Failed to navigate to created directory")?;
                    &mut children[idx]
                },
                _ => unreachable!(),
            };
        }

        current_node.add_child(file)?;
        self.metadata.total_nodes += 1;
        Ok(())
    }
}
