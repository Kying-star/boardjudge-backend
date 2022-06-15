use super::auth;
use crate::config;
use crate::sys::schema::*;
use crate::web::prelude::*;
use axum::extract::Multipart;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct TestdataUploadPayload {
    pub id: Uuid,
    pub name: String,
}

pub async fn testdata_upload(
    mut multipart: Multipart,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let mut payload: Option<TestdataUploadPayload> = None;
    let mut file: Option<Vec<u8>> = None;
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().good()?.to_string();
        if name == "payload" {
            let data = field.bytes().await.good()?;
            let s = std::str::from_utf8(&data).good()?;
            payload = Some(serde_json::from_str(s).good()?);
        } else if name == "file" {
            file = Some(field.bytes().await.good()?.to_vec());
        }
    }
    let payload = payload.found()?;
    let file = file.found()?;
    let model = problem::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    auth::check_root_or_admin_of_contest(uuid!(model.contest_id), conn, &cookies).await?;
    tokio::fs::write(
        format!(
            "{}/{}/{}/{}",
            config().judger.root,
            "testdata",
            payload.id,
            payload.name
        ),
        file,
    )
    .await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct TestdataDownloadPayload {
    pub id: Uuid,
    pub name: String,
}

pub async fn testdata_download(
    Json(payload): Json<TestdataDownloadPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = problem::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    auth::check_root_or_admin_or_player_of_contest(uuid!(model.contest_id), conn, &cookies).await?;
    let data = tokio::fs::read(format!(
        "{}/{}/{}/{}",
        config().judger.root,
        "testdata",
        payload.id,
        payload.name
    ))
    .await?;
    Ok(Json(data))
}

#[derive(Deserialize)]
pub struct TestdataDeletePayload {
    pub id: Uuid,
    pub name: String,
}

pub async fn testdata_delete(
    Json(payload): Json<TestdataDeletePayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = problem::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    auth::check_root_or_admin_or_player_of_contest(uuid!(model.contest_id), conn, &cookies).await?;
    tokio::fs::remove_file(format!(
        "{}/{}/{}/{}",
        config().judger.root,
        "testdata",
        payload.id,
        payload.name
    ))
    .await?;
    Ok(Json(()))
}

#[derive(Deserialize)]
pub struct TestdataListPayload {
    pub id: Uuid,
}

#[derive(Serialize)]
pub struct TestdataModel {
    pub name: String,
}

#[derive(Serialize)]
pub struct TestdataListFeedback {
    pub testdata: Vec<TestdataModel>,
}

pub async fn testdata_list(
    Json(payload): Json<TestdataListPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = problem::Entity::find_by_id(payload.id.to_string())
        .one(conn)
        .await?
        .found()?;
    auth::check_root_or_admin_or_player_of_contest(uuid!(model.contest_id), conn, &cookies).await?;
    if let Ok(mut dir) = tokio::fs::read_dir(format!(
        "{}/{}/{}",
        config().judger.root,
        "testdata",
        payload.id,
    ))
    .await
    {
        let mut testdata = vec![];
        while let Ok(Some(r)) = dir.next_entry().await {
            testdata.push(TestdataModel {
                name: r.file_name().to_str().unwrap().to_string(),
            });
        }
        Ok(Json(TestdataListFeedback { testdata }))
    } else {
        Ok(Json(TestdataListFeedback { testdata: vec![] }))
    }
}
