use super::auth;
use crate::sys::schema::*;
use crate::web::prelude::*;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chrono::Utc;
use sea_orm::prelude::*;
use sea_orm::{JoinType, QueryOrder, QuerySelect};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct RecordReadPayload {
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct RecordReadFeedback {
    pub time: DateTime,
    pub user_id: Uuid,
    pub problem_id: Uuid,
    pub code: String,
    pub language: String,
    pub result: String,
    pub status: String,
}

pub async fn record_read(
    Json(payload): Json<RecordReadPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = record::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    let model_problem = problem::Entity::find_by_id(model.problem_id.clone())
        .one(conn)
        .await?;
    let contest_id;
    let user_id;
    if let Some(model_problem) = model_problem {
        contest_id = uuid!(model_problem.contest_id);
        user_id = auth::test_root_or_admin_or_player_of_contest(contest_id, conn, &cookies).await?;
    } else {
        return Err(AppError::Forbidden(None));
    }
    let now = Utc::now().naive_local();
    let flag = contest::Entity::find()
        .filter(contest::Column::Start.gte(now))
        .filter(contest::Column::End.lt(now))
        .one(conn)
        .await
        .map(|x| x.is_some())
        .unwrap_or(false);
    if flag {
        let model_contest = contest::Entity::find_by_id(contest_id.to_string())
            .one(conn)
            .await?;
        if let Some(model_contest) = model_contest {
            let expected = uuid!(model.user_id);
            match user_id {
                Ok(_root_id) => (),
                Err(Ok(_admin_id)) => (),
                Err(Err(player_id)) => {
                    if expected != player_id {
                        return Err(AppError::Forbidden(None));
                    }
                    if now < model_contest.start {
                        return Err(AppError::Forbidden(None));
                    }
                    if model_contest.end < now {
                        return Err(AppError::Forbidden(None));
                    }
                }
            }
        } else {
            return Err(AppError::Forbidden(None));
        }
    }
    Ok(Json(RecordReadFeedback {
        time: model.time,
        user_id: uuid!(model.user_id),
        problem_id: uuid!(model.problem_id),
        code: model.code,
        language: model.language,
        result: model.result,
        status: model.status,
    }))
}

#[derive(Deserialize)]
pub struct RecordDeletePayload {
    pub id: Uuid,
}

pub async fn record_delete(
    Json(payload): Json<RecordDeletePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    record::Entity::delete_by_id(payload.id.to_string())
        .exec(conn)
        .await?;
    Ok(Json(()))
}

#[derive(Serialize)]
pub struct RecordModel {
    pub id: Uuid,
    pub time: DateTime,
    pub user_id: Uuid,
    pub problem_id: Uuid,
    pub language: String,
    pub result: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct RecordListFeedback {
    records: Vec<RecordModel>,
}

pub async fn record_list(
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let user_id = auth::check_user(conn, &cookies).await?;
    if auth::test_root(conn, &cookies).await.is_ok() {
        let records = record::Entity::find()
            .order_by_asc(record::Column::Time)
            .all(conn)
            .await?
            .into_iter()
            .map(
                |record::Model {
                     id,
                     time,
                     user_id,
                     problem_id,
                     code: _,
                     language,
                     result,
                     status,
                 }| RecordModel {
                    id: uuid!(id),
                    time,
                    user_id: uuid!(user_id),
                    problem_id: uuid!(problem_id),
                    language,
                    result,
                    status,
                },
            )
            .collect();
        return Ok(Json(RecordListFeedback { records }));
    }
    let records = record::Entity::find()
        .join(JoinType::InnerJoin, problem::Relation::Record.def())
        .join(JoinType::InnerJoin, contest::Relation::Problem.def())
        .join(JoinType::InnerJoin, privilege::Relation::Contest.def())
        .filter(privilege::Column::UserId.eq(user_id.to_string()))
        .all(conn)
        .await?
        .into_iter()
        .map(
            |record::Model {
                 id,
                 time,
                 user_id,
                 problem_id,
                 code: _,
                 language,
                 result,
                 status,
             }| RecordModel {
                id: uuid!(id),
                time,
                user_id: uuid!(user_id),
                problem_id: uuid!(problem_id),
                language,
                result,
                status,
            },
        )
        .collect();
    Ok(Json(RecordListFeedback { records }))
}
