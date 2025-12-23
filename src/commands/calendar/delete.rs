use crate::client::ApiClient;
use crate::error::Result;

pub async fn delete_event(
    client: &ApiClient,
    calendar_id: &str,
    event_id: &str,
) -> Result<()> {
    let path = format!(
        "/calendars/{}/events/{}",
        urlencoding::encode(calendar_id),
        urlencoding::encode(event_id)
    );
    client.delete(&path).await
}
