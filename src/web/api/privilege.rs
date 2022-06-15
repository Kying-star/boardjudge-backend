use super::auth;
use crate::sys::schema::*;
use crate::web::prelude::*;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use sea_orm::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct PrivilegeCreatePayload {
    pub user_id: Uuid,
    pub contest_id: Uuid,
    pub kind: String,
}

pub async fn privilege_create(
    Json(payload): Json<PrivilegeCreatePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    privilege::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        user_id: Set(payload.user_id.to_string()),
        contest_id: Set(payload.contest_id.to_string()),
        kind: Set(payload.kind),
    }
    .insert(conn)
    .await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct PrivilegeReadPayload {
    pub user_id: Option<Uuid>,
    pub contest_id: Option<Uuid>,
    pub kind: Option<String>,
}

#[derive(Serialize)]
pub struct PrivilegeModel {
    pub user_id: Uuid,
    pub contest_id: Uuid,
    pub kind: String,
}

#[derive(Serialize)]
pub struct PrivilegeReadFeedback {
    pub privilege: Vec<PrivilegeModel>,
}

pub async fn privilege_read(
    Json(payload): Json<PrivilegeReadPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    let mut select = privilege::Entity::find();
    if let Some(user_id) = payload.user_id {
        select = select.filter(privilege::Column::UserId.eq(user_id.to_string()));
    }
    if let Some(contest_id) = payload.contest_id {
        select = select.filter(privilege::Column::UserId.eq(contest_id.to_string()));
    }
    if let Some(kind) = payload.kind {
        select = select.filter(privilege::Column::UserId.eq(kind.to_string()));
    }
    let model = select.all(conn).await?;
    Ok(Json(PrivilegeReadFeedback {
        privilege: model
            .into_iter()
            .map(
                |privilege::Model {
                     id: _,
                     user_id,
                     contest_id,
                     kind,
                 }| PrivilegeModel {
                    user_id: uuid!(user_id),
                    contest_id: uuid!(contest_id),
                    kind,
                },
            )
            .collect(),
    }))
}

#[derive(Deserialize)]
pub struct PrivilegeDeletePayload {
    pub user_id: Uuid,
    pub contest_id: Uuid,
    pub kind: String,
}

pub async fn privilege_delete(
    Json(payload): Json<PrivilegeDeletePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    privilege::Entity::delete_many()
        .filter(privilege::Column::UserId.eq(payload.user_id.to_string()))
        .filter(privilege::Column::UserId.eq(payload.contest_id.to_string()))
        .filter(privilege::Column::UserId.eq(payload.kind))
        .exec(conn)
        .await?;
    Ok(Json(()))
}
