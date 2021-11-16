use crate::{models::User, Client, Result, Stream, NO_QUERY};
use serde::Serialize;

/// Get all users as an automatically paged Stream.
///
/// Note, do not pass `limit` or `skip` parameters since they are used
/// internally for paging.
///
/// See: [List
/// Users](https://bonusly.docs.apiary.io/#reference/0/users/list-users)
pub fn all<Q>(client: &Client, page_size: usize, params: &'static Q) -> Stream<User>
where
    Q: Serialize + ?Sized + std::marker::Sync,
{
    client.get_stream("/users", page_size, params)
}

/// Get a specific user by their id
///
/// See: [Retrieve a
/// User](https://bonusly.docs.apiary.io/#reference/0/users/retrieve-a-user)
pub async fn get(client: &Client, id: &str) -> Result<User> {
    client.get(&format!("/users/{}", id), NO_QUERY).await
}

#[cfg(test)]
mod test {
    use crate::{env_var, users, Client, IntoVec, StreamExt, NO_QUERY};
    use tokio::test;

    #[test]
    async fn all() {
        let client = Client::from_env("test.env").expect("client");
        let users = users::all(&client, 10, NO_QUERY)
            .take(10)
            .into_vec()
            .await
            .expect("users");
        assert_eq!(users.len(), 10);
    }

    #[test]
    async fn get() {
        let client = Client::from_env("test.env").expect("client");
        let _ = users::get(
            &client,
            &env_var::<String>("BONUSLY_TEST_USER").expect("test user id"),
        )
        .await
        .expect("user");
    }
}
