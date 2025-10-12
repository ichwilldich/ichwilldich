use centaurus::{
  bail,
  error::{ErrorReportStatusExt, Result},
};
use entity::user;
use http::StatusCode;
use sea_orm::prelude::*;
use tracing::instrument;

pub struct UserTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> UserTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  #[allow(unused)]
  #[instrument(skip(self))]
  pub async fn get_user(&self, id: Uuid) -> Result<entity::user::Model> {
    let res = entity::user::Entity::find_by_id(id).one(self.db).await?;

    res
      .ok_or(sea_orm::DbErr::RecordNotFound("Not Found".into()))
      .status(StatusCode::NOT_FOUND)
  }

  #[instrument(skip(self))]
  pub async fn get_user_by_name(&self, name: String) -> Result<entity::user::Model> {
    let res = entity::user::Entity::find()
      .filter(user::Column::Name.eq(name))
      .one(self.db)
      .await?;

    res
      .ok_or(sea_orm::DbErr::RecordNotFound("Not Found".into()))
      .status(StatusCode::NOT_FOUND)
  }

  #[instrument(skip(self))]
  pub async fn create_user(&self, user: user::Model) -> Result<()> {
    let user: user::ActiveModel = user.into();

    user.insert(self.db).await?;

    Ok(())
  }

  #[instrument(skip(self))]
  pub async fn list_users(&self) -> Result<Vec<user::Model>> {
    let res = entity::user::Entity::find().all(self.db).await?;

    Ok(res)
  }

  #[instrument(skip(self))]
  pub async fn delete_user(&self, id: Uuid) -> Result<()> {
    let result = entity::user::Entity::delete_by_id(id).exec(self.db).await?;
    if result.rows_affected == 0 {
      bail!(NOT_FOUND, "User not found");
    }

    Ok(())
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[tokio::test]
  async fn test_user_table() {
    let db = crate::db::test::test_db().await;
    let user_table = db.user();

    let user_id = Uuid::new_v4();
    let user_name = "test_user".to_string();
    let user_password = "hashed_password".to_string();
    let user_salt = "salt".to_string();
    let user = user::Model {
      id: user_id,
      name: user_name.clone(),
      password: user_password.clone(),
      salt: user_salt.clone(),
    };

    // Test create_user
    user_table
      .create_user(user.clone())
      .await
      .expect("Failed to create user");

    // Test get_user_by_name
    let fetched_user = user_table
      .get_user_by_name(user_name.clone())
      .await
      .expect("Failed to fetch user by name");
    assert_eq!(fetched_user.id, user_id);
    assert_eq!(fetched_user.name, user_name);
    assert_eq!(fetched_user.password, user_password);
    assert_eq!(fetched_user.salt, user_salt);

    // Test get_user
    let fetched_user_by_id = user_table
      .get_user(user_id)
      .await
      .expect("Failed to fetch user by id");
    assert_eq!(fetched_user_by_id.id, user_id);
    assert_eq!(fetched_user_by_id.name, user_name);
    assert_eq!(fetched_user_by_id.password, user_password);
    assert_eq!(fetched_user_by_id.salt, user_salt);

    // Test list_users
    let users = user_table.list_users().await.expect("Failed to list users");
    assert_eq!(users.len(), 1);

    // Test delete_user
    user_table
      .delete_user(user_id)
      .await
      .expect("Failed to delete user");
    let users_after_delete = user_table
      .list_users()
      .await
      .expect("Failed to list users after delete");
    assert_eq!(users_after_delete.len(), 0);
  }
}
