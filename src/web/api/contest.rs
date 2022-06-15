use super::auth;
use crate::sys::schema::*;
use crate::web::prelude::*;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use sea_orm::prelude::*;
use sea_orm::{JoinType, QueryOrder, QuerySelect, Set};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::str::FromStr;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ContestCreatePayload {
    pub nick: String,
    pub description: String,
    pub start: DateTime,
    pub end: DateTime,
}

#[derive(Serialize)]
pub struct ContestCreateFeedback {
    pub id: Uuid,
}

pub async fn contest_create(
    Json(payload): Json<ContestCreatePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    let model = contest::ActiveModel {
        id: Set(Uuid::new_v4().to_string()),
        nick: Set(payload.nick),
        description: Set(payload.description),
        start: Set(payload.start),
        end: Set(payload.end),
    }
    .insert(conn)
    .await?;
    Ok(Json(ContestCreateFeedback {
        id: uuid!(model.id),
    }))
}

#[derive(Deserialize)]
pub struct ContestUpdatePayload {
    pub id: Uuid,
    pub nick: Option<String>,
    pub description: Option<String>,
    pub start: Option<DateTime>,
    pub end: Option<DateTime>,
}

pub async fn contest_update(
    Json(payload): Json<ContestUpdatePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    let model = contest::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    let mut model: contest::ActiveModel = model.into();
    if let Some(nick) = payload.nick {
        model.nick = Set(nick);
    }
    if let Some(description) = payload.description {
        model.description = Set(description);
    }
    if let Some(start) = payload.start {
        model.start = Set(start);
    }
    if let Some(end) = payload.end {
        model.end = Set(end);
    }
    model.update(conn).await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct ContestReadPayload {
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct ContestReadFeedback {
    pub nick: String,
    pub description: String,
    pub start: DateTime,
    pub end: DateTime,
}

pub async fn contest_read(
    Json(payload): Json<ContestReadPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root_or_admin_or_player_of_contest(payload.id, conn, &cookies).await?;
    let model = contest::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    Ok(Json(ContestReadFeedback {
        nick: model.nick,
        description: model.description,
        start: model.start,
        end: model.end,
    }))
}

#[derive(Deserialize)]
pub struct ContestDeletePayload {
    pub id: Uuid,
}

pub async fn contest_delete(
    Json(payload): Json<ContestDeletePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root(conn, &cookies).await?;
    contest::Entity::delete_by_id(payload.id.to_string())
        .exec(conn)
        .await?;
    Ok(Json(()))
}

#[derive(Serialize)]
pub struct ContestModel {
    pub id: Uuid,
    pub nick: String,
    pub description: String,
    pub start: DateTime,
    pub end: DateTime,
}

#[derive(Serialize)]
pub struct ContestListFeedback {
    contests: Vec<ContestModel>,
}

pub async fn contest_list(
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let user_id = auth::check_user(conn, &cookies).await?;
    if auth::test_root(conn, &cookies).await.is_ok() {
        let contests = contest::Entity::find()
            .order_by_asc(contest::Column::Start)
            .all(conn)
            .await?
            .into_iter()
            .map(
                |contest::Model {
                     id,
                     nick,
                     description,
                     start,
                     end,
                 }| ContestModel {
                    id: uuid!(id),
                    nick,
                    description,
                    start,
                    end,
                },
            )
            .collect();
        return Ok(Json(ContestListFeedback { contests }));
    }
    let contests = contest::Entity::find()
        .inner_join(privilege::Entity)
        .filter(privilege::Column::UserId.eq(user_id.to_string()))
        .all(conn)
        .await?
        .into_iter()
        .map(
            |contest::Model {
                 id,
                 nick,
                 description,
                 start,
                 end,
             }| ContestModel {
                id: uuid!(id),
                nick,
                description,
                start,
                end,
            },
        )
        .collect();
    Ok(Json(ContestListFeedback { contests }))
}

#[derive(Deserialize)]
pub struct ContestRanklistPayload {
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct ContestRanklistFeedback {
    pub records: BTreeMap<Uuid, BTreeMap<Uuid, (Uuid, String, DateTime)>>,
}

pub async fn contest_ranklist(
    Json(payload): Json<ContestRanklistPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    auth::check_root_or_admin_or_player_of_contest(payload.id, conn, &cookies).await?;
    let models = record::Entity::find()
        .join(JoinType::InnerJoin, record::Relation::Problem.def())
        .join(JoinType::InnerJoin, problem::Relation::Contest.def())
        .filter(problem::Column::ContestId.eq(payload.id))
        .all(conn)
        .await?;
    let mut records = BTreeMap::<Uuid, BTreeMap<Uuid, (Uuid, String, DateTime)>>::new();
    for model in models {
        let record_id = uuid!(model.id);
        let user_id = uuid!(model.user_id);
        let problem_id = uuid!(model.problem_id);
        if records.get(&user_id).is_none() {
            records.insert(user_id, BTreeMap::new());
        }
        let p = records.get_mut(&user_id).unwrap();
        if p.get(&problem_id).is_some() {
            if p[&problem_id].1 == "accepted" {
                if model.time < p[&problem_id].2 && model.status == "accepted" {
                    *p.get_mut(&problem_id).unwrap() = (record_id, model.status, model.time);
                }
            } else {
                if p[&problem_id].2 < model.time || model.status == "accepted" {
                    *p.get_mut(&problem_id).unwrap() = (record_id, model.status, model.time);
                }
            }
        } else {
            p.insert(problem_id, (record_id, model.status, model.time));
        }
    }
    Ok(Json(ContestRanklistFeedback { records }))
}
