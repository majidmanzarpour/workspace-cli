use crate::client::ApiClient;
use crate::error::Result;
use super::types::{SpaceReadState, ThreadReadState, UnreadResult, UnreadSpace, Space};
use super::spaces::{list_spaces, ListSpacesParams};
use super::messages::{list_messages, ListMessagesParams};
use futures::future::join_all;

pub async fn get_space_read_state(client: &ApiClient, space_name: &str) -> Result<SpaceReadState> {
    let space = if space_name.starts_with("spaces/") {
        space_name.to_string()
    } else {
        format!("spaces/{}", space_name)
    };
    let path = format!("/users/me/{}/spaceReadState", space);
    client.get(&path).await
}

pub async fn get_thread_read_state(client: &ApiClient, space_name: &str, thread_name: &str) -> Result<ThreadReadState> {
    let space = if space_name.starts_with("spaces/") {
        space_name.to_string()
    } else {
        format!("spaces/{}", space_name)
    };
    let thread = if thread_name.contains("/threads/") {
        thread_name.rsplit("/threads/").next().unwrap_or(thread_name).to_string()
    } else {
        thread_name.to_string()
    };
    let path = format!("/users/me/{}/threads/{}/threadReadState", space, thread);
    client.get(&path).await
}

fn parse_since_to_cutoff(since: &str) -> Option<String> {
    if since == "all" { return None; }
    let days: u64 = if since.ends_with('d') {
        since.trim_end_matches('d').parse().unwrap_or(7)
    } else {
        since.parse().unwrap_or(7)
    };
    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    Some(cutoff.to_rfc3339())
}

pub async fn get_unread_messages(client: &ApiClient, limit_per_space: u32, space_type_filter: Option<&str>, since: &str) -> Result<UnreadResult> {
    // Step 1: List spaces with server-side spaceType filter
    let api_filter = match space_type_filter {
        Some("all") | None => None,
        Some(t) => Some(format!("spaceType = \"{}\"", t)),
    };
    let spaces_response = list_spaces(client, ListSpacesParams {
        page_size: 1000,
        page_token: None,
        filter: api_filter,
    }).await?;

    // Step 2: Filter out bot DMs, unnamed spaces, inactive spaces, and old spaces
    let since_cutoff = parse_since_to_cutoff(since);
    let spaces: Vec<&Space> = spaces_response.spaces.iter()
        .filter(|s| s.name.is_some())
        .filter(|s| !s.single_user_bot_dm.unwrap_or(false))
        .filter(|s| s.last_active_time.is_some()) // No activity = no unread
        .filter(|s| {
            // Skip spaces with no activity since cutoff
            match (&since_cutoff, &s.last_active_time) {
                (Some(cutoff), Some(active)) => active.as_str() >= cutoff.as_str(),
                _ => true,
            }
        })
        .collect();

    eprintln!("Checking {} spaces for unread messages...", spaces.len());

    // Step 3: Fetch read states concurrently (batches of 50)
    let mut unread_spaces: Vec<UnreadSpace> = Vec::new();
    let mut total_messages = 0usize;

    for chunk in spaces.chunks(50) {
        // Concurrently fetch read states
        let read_state_futures: Vec<_> = chunk.iter().map(|space| {
            let space_name = space.name.as_ref().unwrap().clone();
            async move {
                let rs = get_space_read_state(client, &space_name).await;
                (space_name, rs)
            }
        }).collect();

        let read_states = join_all(read_state_futures).await;

        // Step 4: Compare lastActiveTime vs lastReadTime — skip already-read spaces
        let mut needs_messages = Vec::new();
        for (space_name, rs_result) in &read_states {
            if let Ok(rs) = rs_result {
                if let Some(ref last_read) = rs.last_read_time {
                    if last_read.is_empty() { continue; }

                    // Find space metadata
                    let space_meta = chunk.iter().find(|s| s.name.as_deref() == Some(space_name.as_str()));

                    // If lastActiveTime available, skip spaces where lastActiveTime <= lastReadTime
                    if let Some(meta) = space_meta {
                        if let Some(ref last_active) = meta.last_active_time {
                            if last_active <= last_read {
                                continue; // Already read — skip message fetch
                            }
                        }
                    }

                    needs_messages.push((space_name.clone(), last_read.clone(), space_meta.cloned()));
                }
            }
        }

        if needs_messages.is_empty() { continue; }

        // Step 5: Concurrently fetch messages only for potentially unread spaces
        let msg_futures: Vec<_> = needs_messages.iter().map(|(space_name, last_read, _)| {
            let filter = format!("createTime > \"{}\"", last_read);
            let params = ListMessagesParams {
                space_name: space_name.clone(),
                page_size: limit_per_space,
                page_token: None,
                order_by: Some("createTime DESC".to_string()),
                filter: Some(filter),
            };
            async move {
                list_messages(client, params).await
            }
        }).collect();

        let msg_results = join_all(msg_futures).await;

        for (i, msg_result) in msg_results.into_iter().enumerate() {
            if let Ok(response) = msg_result {
                if !response.messages.is_empty() {
                    let (space_name, last_read, space_meta) = &needs_messages[i];
                    let count = response.messages.len();
                    total_messages += count;
                    unread_spaces.push(UnreadSpace {
                        space_name: Some(space_name.clone()),
                        display_name: space_meta.as_ref().and_then(|s| s.display_name.clone()),
                        space_type: space_meta.as_ref().and_then(|s| s.space_type.clone()),
                        last_read_time: Some(last_read.clone()),
                        messages: response.messages,
                    });
                }
            }
        }
    }

    let total_spaces = unread_spaces.len();
    Ok(UnreadResult {
        spaces: unread_spaces,
        total_unread_spaces: total_spaces,
        total_unread_messages: total_messages,
    })
}
