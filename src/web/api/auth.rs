use crate::sys::schema::*;
use crate::utils::sha256;
use crate::web::prelude::*;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub password: String,
}

#[derive(Deserialize)]
pub struct LoginPayload {
    pub name: String,
    pub password: String,
}

#[derive(Serialize)]
#[serde(tag = "type")]
pub enum LoginFeedback {
    Ok {},
    Err { message: String },
}

pub async fn auth_login(
    Json(payload): Json<LoginPayload>,
    Extension(ref conn): Extension<DatabaseConnection>,
    cookies: Cookies,
) -> AppResult<impl IntoResponse> {
    let model = user::Entity::find().one(conn).await?.found()?;
    let id = model.id;
    let expected = model.password;
    if sha256(&payload.password) != expected {
        return Ok(Json(LoginFeedback::Err {
            message: "incorrect password".to_string(),
        }));
    }
    if model.banned == 0 {
        return Ok(Json(LoginFeedback::Err {
            message: "account is banned".to_string(),
        }));
    }
    cookies.add(Cookie::new(
        "session",
        serde_json::to_string(&Session {
            id: uuid!(id),
            password: payload.password,
        })
        .unwrap(),
    ));
    Ok(Json(LoginFeedback::Ok {}))
}

pub async fn auth_logout(cookies: Cookies) -> impl IntoResponse {
    cookies.remove(Cookie::named("session"));
}

pub async fn test_user(conn: &DatabaseConnection, cookies: Cookies) -> Result<Uuid, ()> {
    let session: Session =
        serde_json::from_str(cookies.get("session").ok_or(())?.value()).map_err(|_| ())?;
    let model = user::Entity::find_by_id(session.id.to_string())
        .one(conn)
        .await
        .map_err(|_| ())?
        .ok_or(())?;
    if sha256(&session.password) != model.password {
        return Err(());
    }
    if model.banned == 0 {
        return Err(());
    }
    if model.root != 0 {
        return Ok(session.id);
    }
    Ok(session.id)
}

pub async fn check_user(conn: &DatabaseConnection, cookies: &Cookies) -> AppResult<Uuid> {
    let session: Session = serde_json::from_str(cookies.get("session").allow()?.value())
        .ok()
        .allow()?;
    let model = user::Entity::find_by_id(session.id.to_string())
        .one(conn)
        .await?
        .allow()?;
    if sha256(&session.password) != model.password {
        return Err(AppError::Forbidden(None));
    }
    if model.banned == 0 {
        return Err(AppError::Forbidden(None));
    }
    if model.root != 0 {
        return Ok(session.id);
    }
    Ok(session.id)
}

pub async fn test_root(conn: &DatabaseConnection, cookies: &Cookies) -> Result<Uuid, ()> {
    let session: Session =
        serde_json::from_str(cookies.get("session").ok_or(())?.value()).map_err(|_| ())?;
    let model = user::Entity::find_by_id(session.id.to_string())
        .one(conn)
        .await
        .map_err(|_| ())?
        .ok_or(())?;
    if sha256(&session.password) != model.password {
        return Err(());
    }
    if model.banned == 0 {
        return Err(());
    }
    if model.root != 0 {
        return Ok(session.id);
    }
    Ok(session.id)
}

pub async fn check_root(conn: &DatabaseConnection, cookies: &Cookies) -> AppResult<Uuid> {
    let session: Session = serde_json::from_str(cookies.get("session").allow()?.value())
        .ok()
        .allow()?;
    let model = user::Entity::find_by_id(session.id.to_string())
        .one(conn)
        .await?
        .allow()?;
    if sha256(&session.password) != model.password {
        return Err(AppError::Forbidden(None));
    }
    if model.banned == 0 {
        return Err(AppError::Forbidden(None));
    }
    if model.root != 0 {
        return Ok(session.id);
    }
    Err(AppError::Forbidden(None))
}

pub async fn check_root_or_admin_of_contest(
    contest_id: Uuid,
    conn: &DatabaseConnection,
    cookies: &Cookies,
) -> AppResult<Uuid> {
    let session: Session = serde_json::from_str(cookies.get("session").allow()?.value())
        .ok()
        .allow()?;
    let model = user::Entity::find_by_id(session.id.to_string())
        .one(conn)
        .await?
        .allow()?;
    if sha256(&session.password) != model.password {
        return Err(AppError::Forbidden(None));
    }
    if model.banned == 0 {
        return Err(AppError::Forbidden(None));
    }
    if model.root != 0 {
        return Ok(session.id);
    }
    privilege::Entity::find()
        .filter(privilege::Column::ContestId.like(&contest_id.to_string()))
        .filter(privilege::Column::UserId.like(&model.id))
        .filter(privilege::Column::Kind.like("admin"))
        .one(conn)
        .await?
        .allow()?;
    Ok(session.id)
}

pub async fn check_root_or_player_of_contest(
    contest_id: Uuid,
    conn: &DatabaseConnection,
    cookies: &Cookies,
) -> AppResult<Uuid> {
    let session: Session = serde_json::from_str(cookies.get("session").allow()?.value())
        .ok()
        .allow()?;
    let model = user::Entity::find_by_id(session.id.to_string())
        .one(conn)
        .await?
        .allow()?;
    if sha256(&session.password) != model.password {
        return Err(AppError::Forbidden(None));
    }
    if model.banned == 0 {
        return Err(AppError::Forbidden(None));
    }
    if model.root != 0 {
        return Ok(session.id);
    }
    privilege::Entity::find()
        .filter(privilege::Column::ContestId.like(&contest_id.to_string()))
        .filter(privilege::Column::UserId.like(&model.id))
        .filter(privilege::Column::Kind.like("player"))
        .one(conn)
        .await?
        .allow()?;
    Ok(session.id)
}

pub async fn test_root_or_admin_or_player_of_contest(
    contest_id: Uuid,
    conn: &DatabaseConnection,
    cookies: &Cookies,
) -> AppResult<Result<Uuid, Result<Uuid, Uuid>>> {
    let session: Session = serde_json::from_str(cookies.get("session").allow()?.value())
        .ok()
        .allow()?;
    let model = user::Entity::find_by_id(session.id.to_string())
        .one(conn)
        .await?
        .allow()?;
    if sha256(&session.password) != model.password {
        return Err(AppError::Forbidden(None));
    }
    if model.banned == 0 {
        return Err(AppError::Forbidden(None));
    }
    if model.root != 0 {
        return Ok(Ok(session.id));
    }
    let x = privilege::Entity::find()
        .filter(privilege::Column::ContestId.like(&contest_id.to_string()))
        .filter(privilege::Column::UserId.like(&model.id))
        .one(conn)
        .await?
        .allow()?;
    if x.kind == "admin" {
        Ok(Err(Ok(session.id)))
    } else {
        Ok(Err(Err(session.id)))
    }
}

pub async fn check_root_or_admin_or_player_of_contest(
    contest_id: Uuid,
    conn: &DatabaseConnection,
    cookies: &Cookies,
) -> AppResult<Uuid> {
    let session: Session = serde_json::from_str(cookies.get("session").allow()?.value())
        .ok()
        .allow()?;
    let model = user::Entity::find_by_id(session.id.to_string())
        .one(conn)
        .await?
        .allow()?;
    if sha256(&session.password) != model.password {
        return Err(AppError::Forbidden(None));
    }
    if model.banned == 0 {
        return Err(AppError::Forbidden(None));
    }
    if model.root != 0 {
        return Ok(session.id);
    }
    privilege::Entity::find()
        .filter(privilege::Column::ContestId.like(&contest_id.to_string()))
        .filter(privilege::Column::UserId.like(&model.id))
        .one(conn)
        .await?
        .allow()?;
    Ok(session.id)
}
