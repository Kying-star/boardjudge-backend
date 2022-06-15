use super::auth;
use crate::judger::Judger;
use crate::sys::schema::*;
use crate::web::prelude::*;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chrono::Utc;
use sea_orm::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct SubmitPayload {
    pub problem_id: Uuid,
    pub code: String,
    pub language: String,
}

#[derive(Serialize)]
pub struct SubmitFeedback {
    pub id: Uuid,
}

pub async fn submit(
    Json(payload): Json<SubmitPayload>,
    Extension(conn): Extension<DatabaseConnection>,
    Extension(judger): Extension<Judger>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = problem::Entity::find_by_id(payload.problem_id.to_string())
        .one(&conn)
        .await?
        .found()?;
    let user_id =
        auth::check_root_or_admin_of_contest(uuid!(model.contest_id), &conn, &cookies).await?;
    let record_id = Uuid::new_v4();
    let modell = record::ActiveModel {
        id: Set(record_id.to_string()),
        time: Set(Utc::now().naive_local()),
        user_id: Set(user_id.to_string()),
        problem_id: Set(payload.problem_id.to_string()),
        code: Set(payload.code.clone()),
        language: Set(payload.language.clone()),
        result: Set("{}".to_string()),
        status: Set("waiting".to_string()),
    }
    .insert(&conn)
    .await?;
    tokio::spawn(async move {
        let (status, result) = judger
            .judge(crate::judger::Judge {
                record_id,
                problem_id: payload.problem_id,
                time_limit: model.limit_time,
                memory_limit: model.limit_memory,
                language: payload.language,
                code: payload.code,
            })
            .await;
        let mut modell: record::ActiveModel = modell.into();
        modell.status = Set(Into::<&'static str>::into(status).to_string());
        modell.result = Set(serde_json::to_string(&result).unwrap());
        modell.save(&conn).await.unwrap();
    });
    Ok(Json(SubmitFeedback { id: record_id }))
}
