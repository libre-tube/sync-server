use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::database::DbError;
use crate::models::Account;
use crate::{DbConnection, models};

use crate::schema::account::dsl::*;

pub async fn find_account_by_id(
    conn: &mut DbConnection,
    id_: &str,
) -> Result<Option<models::Account>, DbError> {
    let item = account
        .filter(id.eq(id_.to_string()))
        .select(models::Account::as_select())
        .first::<models::Account>(conn)
        .await
        .optional()?;

    Ok(item)
}

pub async fn find_account_by_name_hash(
    conn: &mut DbConnection,
    name_hash_: &str,
) -> Result<Option<models::Account>, DbError> {
    let item = account
        .filter(name_hash.eq(name_hash_.to_string()))
        .select(models::Account::as_select())
        .first::<models::Account>(conn)
        .await
        .optional()?;

    Ok(item)
}

pub async fn insert_new_account(
    conn: &mut DbConnection,
    new_account: &Account,
) -> Result<models::Account, DbError> {
    let created_account = diesel::insert_into(account)
        .values(new_account)
        .returning(models::Account::as_returning())
        .get_result(conn)
        .await?;

    Ok(created_account)
}

pub async fn delete_existing_account(
    conn: &mut DbConnection,
    account_id: &str, // prevent collision with db column imported inside the function
) -> Result<(), DbError> {
    diesel::delete(account.filter(id.eq(account_id.to_string())))
        .execute(conn)
        .await?;

    Ok(())
}
