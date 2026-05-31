use futures::{TryStreamExt, stream};
use mattermost_api::{client::Mattermost, errors::ApiError};
use serde::Serialize;

use crate::models;

pub async fn fetch_all_active_users(api: &Mattermost) -> Result<Vec<models::MMUser>, ApiError> {
    Ok(stream::try_unfold(0, move |page| async move {
        let page_s = page.to_string();
        let params = [
            ("page", page_s.as_str()),
            ("active", "true"), // 有効な(無効化されていない)ユーザのみを表示
        ];

        // GET /api/v4/users
        let res: Vec<models::MMUser> = api.query("GET", "users", Some(&params), None).await?;

        if res.is_empty() {
            Ok::<_, ApiError>(None)
        } else {
            Ok(Some((res, page + 1)))
        }
    })
    .try_collect::<Vec<Vec<_>>>()
    .await?
    .into_iter()
    .flatten()
    .collect())
}

pub async fn get_my_info(api: &Mattermost) -> Result<models::MMUser, ApiError> {
    api.query("GET", "users/me", None, None).await
}

pub async fn get_or_create_dm_channel_id<T: AsRef<str> + Serialize>(
    api: &Mattermost,
    ids: &[T; 2],
) -> Result<models::Channel, ApiError> {
    // to_stringは確実に成功するとみなせそう
    api.query(
        "POST",
        "channels/direct",
        None,
        Some(&serde_json::to_string(ids).unwrap()),
    )
    .await
}

pub async fn set_user_inactive(api: &Mattermost, id: &str) -> Result<(), ApiError> {
    api.query("DELETE", format!("users/{}", id).as_str(), None, None)
        .await
}
