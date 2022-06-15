pub mod api;
pub mod prelude;

use crate::judger::Judger;
use axum::extract::Extension;
use axum::routing::{get, post, put};
use axum::Router;
use sea_orm::DatabaseConnection;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;

pub fn router(conn: DatabaseConnection, judger: Judger) -> Router {
    use self::api::auth::*;
    use self::api::contest::*;
    use self::api::privilege::*;
    use self::api::problem::*;
    use self::api::record::*;
    use self::api::submit::*;
    use self::api::testdata::*;
    use self::api::user::*;
    Router::new()
        .route("/api/auth/login", put(auth_login))
        .route("/api/auth/logout", put(auth_logout))
        .route(
            "/api/contest",
            get(contest_read)
                .post(contest_create)
                .patch(contest_update)
                .delete(contest_delete),
        )
        .route("/api/contest/list", get(contest_list))
        .route("/api/contest/ranklist", get(contest_ranklist))
        .route(
            "/api/privilege",
            get(privilege_read)
                .post(privilege_create)
                .delete(privilege_delete),
        )
        .route(
            "/api/problem",
            get(problem_read)
                .post(problem_create)
                .patch(problem_update)
                .delete(problem_delete),
        )
        .route("/api/problem/list", get(problem_list))
        .route("/api/record", get(record_read).delete(record_delete))
        .route("/api/record/list", get(record_list))
        .route(
            "/api/user",
            get(user_read)
                .post(user_create)
                .patch(user_update)
                .delete(user_delete),
        )
        .route("/api/user/list", get(user_list))
        .route("/api/submit", post(submit))
        .route(
            "/api/testdata",
            get(testdata_download)
                .post(testdata_upload)
                .delete(testdata_delete),
        )
        .route("/api/testdata/list", get(testdata_list))
        .layer(ServiceBuilder::new().layer(Extension(conn)))
        .layer(ServiceBuilder::new().layer(Extension(judger)))
        .layer(CookieManagerLayer::new())
}
