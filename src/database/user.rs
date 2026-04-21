use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::models::User;
use crate::{DbConnection, models};

type DbError = Box<dyn std::error::Error + Send + Sync>;
use crate::schema::user::dsl::*;

pub async fn find_user_by_id(
    conn: &mut DbConnection,
    uid: Uuid,
) -> Result<Option<models::User>, DbError> {
    let item = user
        .filter(id.eq(uid.to_string()))
        .select(models::User::as_select())
        .first::<models::User>(conn)
        .await
        .optional()?;

    Ok(item)
}

pub async fn insert_new_user(
    conn: &mut DbConnection,
    new_user: &User, // prevent collision with db column imported inside the function
) -> Result<models::User, DbError> {
    let created_user = diesel::insert_into(user)
        .values(new_user)
        .returning(models::User::as_returning())
        .get_result(conn)
        .await
        .expect("Error inserting user");

    Ok(created_user)
}
