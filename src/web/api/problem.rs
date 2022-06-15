use super::auth;
use crate::sys::schema::*;
use crate::web::prelude::*;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use sea_orm::prelude::*;
use sea_orm::{JoinType, QueryOrder, QuerySelect, Set};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ProblemCreatePayload {
    pub nick: String,
    pub description: String,
    pub limit_time: u32,
    pub limit_memory: u32,
    pub contest_id: Uuid,
}

#[derive(Serialize)]
pub struct ProblemCreateFeedback {
    pub id: Uuid,
}

pub async fn problem_create(
    Json(payload): Json<ProblemCreatePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root_or_admin_of_contest(payload.contest_id, conn, &cookies).await?;
    let model = problem::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        nick: Set(payload.nick),
        description: Set(payload.description),
        limit_time: Set(payload.limit_time),
        limit_memory: Set(payload.limit_memory),
        contest_id: Set(payload.contest_id.to_string()),
    }
    .insert(conn)
    .await?;
    Ok(Json(ProblemCreateFeedback {
        id: uuid!(model.id),
    }))
}

#[derive(Deserialize)]
pub struct ProblemUpdatePayload {
    pub id: Uuid,
    pub nick: Option<String>,
    pub description: Option<String>,
    pub limit_time: Option<u32>,
    pub limit_memory: Option<u32>,
}

pub async fn problem_update(
    Json(payload): Json<ProblemUpdatePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = problem::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    auth::check_root_or_admin_of_contest(uuid!(model.contest_id), conn, &cookies).await?;
    let mut model: problem::ActiveModel = model.into();
    if let Some(nick) = payload.nick {
        model.nick = Set(nick);
    }
    if let Some(description) = payload.description {
        model.description = Set(description);
    }
    if let Some(limit_time) = payload.limit_time {
        model.limit_time = Set(limit_time);
    }
    if let Some(limit_memory) = payload.limit_memory {
        model.limit_memory = Set(limit_memory);
    }
    model.update(conn).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct ProblemReadPayload {
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct ProblemReadFeedback {
    pub nick: String,
    pub description: String,
    pub limit_time: u32,
    pub limit_memory: u32,
    pub contest_id: Uuid,
}

pub async fn problem_read(
    Json(payload): Json<ProblemReadPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = problem::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    auth::check_root_or_admin_or_player_of_contest(uuid!(model.contest_id), conn, &cookies).await?;
    Ok(Json(ProblemReadFeedback {
        nick: model.nick,
        description: model.description,
        limit_time: model.limit_time,
        limit_memory: model.limit_memory,
        contest_id: uuid!(model.contest_id),
    }))
}

#[derive(Deserialize)]
pub struct ProblemDeletePayload {
    pub id: Uuid,
}

pub async fn problem_delete(
    Json(payload): Json<ProblemDeletePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = problem::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    auth::check_root_or_admin_or_player_of_contest(uuid!(model.contest_id), conn, &cookies).await?;
    model.delete(conn).await?;
    Ok(Json(()))
}

#[derive(Serialize)]
pub struct ProblemModel {
    pub id: Uuid,
    pub nick: String,
    pub description: String,
    pub limit_time: u32,
    pub limit_memory: u32,
    pub contest_id: Uuid,
}

#[derive(Serialize)]
pub struct ProblemListFeedback {
    pub problems: Vec<ProblemModel>,
}

pub async fn problem_list(
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let user_id = auth::check_user(conn, &cookies).await?;
    if auth::test_root(conn, &cookies).await.is_ok() {
        let problems = problem::Entity::find()
            .order_by_asc(problem::Column::Nick)
            .all(conn)
            .await?
            .into_iter()
            .map(
                |problem::Model {
                     id,
                     nick,
                     description,
                     limit_time,
                     limit_memory,
                     contest_id,
                 }| ProblemModel {
                    id: uuid!(id),
                    nick,
                    description,
                    limit_time,
                    limit_memory,
                    contest_id: uuid!(contest_id),
                },
            )
            .collect();
        return Ok(Json(ProblemListFeedback { problems }));
    }
    let problems = problem::Entity::find()
        .join(JoinType::InnerJoin, contest::Relation::Problem.def())
        .join(JoinType::InnerJoin, privilege::Relation::Contest.def())
        .filter(privilege::Column::UserId.eq(user_id.to_string()))
        .all(conn)
        .await?
        .into_iter()
        .map(
            |problem::Model {
                 id,
                 nick,
                 description,
                 limit_time,
                 limit_memory,
                 contest_id,
             }| ProblemModel {
                id: uuid!(id),
                nick,
                description,
                limit_time,
                limit_memory,
                contest_id: uuid!(contest_id),
            },
        )
        .collect();
    Ok(Json(ProblemListFeedback { problems }))
}
