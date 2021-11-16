use crate::{models::Bonus, Client, Result, Stream, NO_QUERY};
use serde::Serialize;

/// Get all bonuses for a given user as an automatically paged Stream.
///
/// Note, do not pass `limit` or `skip` parameters since they are used
/// internally for paging.
///
/// See: [User
/// Bonuses](https://bonusly.docs.apiary.io/#reference/0/users/bonuses)
pub fn for_user<Q>(
    client: &Client,
    user_id: &str,
    page_size: usize,
    params: &'static Q,
) -> Stream<Bonus>
where
    Q: Serialize + ?Sized + std::marker::Sync,
{
    client.get_stream(&format!("/users/{}/bonuses", user_id), page_size, params)
}

/// Get a bonus by its id
///
/// See: [Retrieve a
/// Bonus](https://bonusly.docs.apiary.io/#reference/0/bonuses/retrieve-a-bonus)
pub async fn get(client: &Client, id: &str) -> Result<Bonus> {
    client.get(&format!("/bonuses/{}", id), NO_QUERY).await
}

/// Create a bonus as either a simple or expanded set of parameters.
///
/// See: [Create a
/// Bonus](https://bonusly.docs.apiary.io/#reference/0/bonuses/create-a-bonus),
/// [Create a Bonus with
/// Fields](https://bonusly.docs.apiary.io/#reference/0/bonuses/create-a-bonus-with-separate-fields-for-reason,-hashtag,-receiver-and-amount)
pub async fn create<Q>(client: &Client, params: &'static Q) -> Result<Bonus>
where
    Q: Serialize + ?Sized + std::marker::Sync,
{
    client.post("/bonuses", params).await
}

#[cfg(test)]
mod test {
    use crate::{bonuses, env_var, Client, IntoVec, StreamExt, NO_QUERY};
    use tokio::test;

    #[test]
    async fn for_user() {
        let client = Client::from_env("test.env").expect("client");
        let user_id = &env_var::<String>("BONUSLY_TEST_USER").expect("test user id");

        let bonuses = bonuses::for_user(&client, user_id, 10, NO_QUERY)
            .take(10)
            .into_vec()
            .await
            .expect("bonuses");
        assert_eq!(bonuses.len(), 10);
    }

    #[test]
    async fn get() {
        let client = Client::from_env("test.env").expect("client");
        let _ = bonuses::get(
            &client,
            &env_var::<String>("BONUSLY_TEST_BONUS").expect("test bonus id"),
        )
        .await
        .expect("user");
    }
}
