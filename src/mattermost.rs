use futures::{TryStreamExt, stream};
use mattermost_api::{client::Mattermost, errors::ApiError};

use crate::models;

pub async fn fetch_all_active_users(api: &Mattermost) -> Result<Vec<models::MMUser>, ApiError> {
    Ok(stream::try_unfold(0, move |page| async move {
        let page_s = page.to_string();
         let query_params = [
            ("page", page_s.as_str()),
            ("active", "true"), // 有効な(無効化されていない)ユーザのみを表示
        ];

        // GET /api/v4/users
        let res: Vec<models::MMUser> = api.query("GET", "users", Some(&query_params), None).await?;
        
        if res.is_empty() {
            Ok::<_, ApiError>(None)
        } else {
            Ok(Some((res, page + 1)))
        }
    }).try_collect::<Vec<Vec<_>>>().await?.into_iter().flatten().collect())
}
