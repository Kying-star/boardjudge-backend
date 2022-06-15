use super::auth;
use crate::sys::schema::*;
use crate::utils::sha256;
use crate::web::prelude::*;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use sea_orm::prelude::*;
use sea_orm::{QueryOrder, Set};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tower_cookies::Cookies;

#[derive(Deserialize)]
pub struct UserCreatePayload {
    pub name: String,
    pub nick: String,
    pub description: String,
    pub password: String,
    pub banned: bool,
    pub root: bool,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub struct UserCreateFeedback {
    pub id: Uuid,
}

pub async fn user_create(
    Json(payload): Json<UserCreatePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    let model = user::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        name: Set(payload.name),
        nick: Set(payload.nick),
        description: Set(payload.description),
        password: Set(sha256(&payload.password)),
        banned: Set(payload.banned.into()),
        root: Set(payload.root.into()),
    }
    .insert(conn)
    .await?;
    Ok(Json(UserCreateFeedback {
        id: uuid!(model.id),
    }))
}

#[derive(Deserialize)]
pub struct UserUpdatePayload {
    pub id: Uuid,
    pub name: Option<String>,
    pub nick: Option<String>,
    pub description: Option<String>,
    pub password: Option<String>,
    pub banned: Option<bool>,
    pub root: Option<bool>,
}

pub async fn user_update(
    Json(payload): Json<UserUpdatePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    let model = user::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    let mut model: user::ActiveModel = model.into();
    if let Some(name) = payload.name {
        model.name = Set(name);
    }
    if let Some(nick) = payload.nick {
        model.nick = Set(nick);
    }
    if let Some(description) = payload.description {
        model.description = Set(description);
    }
    if let Some(password) = payload.password {
        model.password = Set(sha256(&password));
    }
    if let Some(banned) = payload.banned {
        model.banned = Set(banned.into());
    }
    if let Some(root) = payload.root {
        model.root = Set(root.into());
    }
    model.update(conn).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct UserReadPayload {
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct UserReadFeedback {
    pub name: String,
    pub nick: String,
    pub description: String,
    pub banned: bool,
    pub root: bool,
}

pub async fn user_read(
    Json(payload): Json<UserReadPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    let model = user::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    Ok(Json(UserReadFeedback {
        name: model.name,
        nick: model.nick,
        description: model.description,
        banned: model.banned != 0,
        root: model.root != 0,
    }))
}

#[derive(Deserialize)]
pub struct UserDeletePayload {
    pub id: Uuid,
}

pub async fn user_delete(
    Json(payload): Json<UserDeletePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    user::Entity::delete_by_id(payload.id.to_string())
        .exec(conn)
        .await?;
    Ok(Json(()))
}

#[derive(Serialize)]
pub struct UserModel {
    pub id: Uuid,
    pub name: String,
    pub nick: String,
    pub description: String,
    pub banned: bool,
    pub root: bool,
}

#[derive(Serialize)]
pub struct UserListFeedback {
    pub users: Vec<UserModel>,
}

pub async fn user_list(
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_user(conn, &cookies).await?;
    let users = user::Entity::find()
        .order_by_asc(user::Column::Name)
        .all(conn)
        .await?
        .into_iter()
        .map(
            |user::Model {
                 id,
                 name,
                 nick,
                 description,
                 password: _,
                 banned,
                 root,
             }| UserModel {
                id: uuid!(id),
                name,
                nick,
                description,
                banned: banned != 0,
                root: root != 0,
            },
        )
        .collect();
    Ok(Json(UserListFeedback { users }))
}
