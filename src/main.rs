use std::net::SocketAddr;

use axum::{Router, routing::get};

mod error;
mod handlers;
mod models;

#[derive(Clone)]
pub struct AppState {
    db: toasty::Db,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let db = toasty::Db::builder()
        .models(toasty::models!(models::Family))
        .connect("sqlite:./families.db")
        .await?;

    db.push_schema().await?;

    let state = AppState { db };

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{addr}");
    axum::serve(listener, app(state)).await?;

    Ok(())
}

fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(handlers::health))
        .route(
            "/families",
            axum::routing::get(handlers::list_families).post(handlers::create_family),
        )
        .route(
            "/families/{id}",
            axum::routing::get(handlers::get_family)
                .put(handlers::update_family)
                .delete(handlers::delete_family),
        )
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode, header},
    };
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use super::*;

    async fn test_app() -> AppState {
        let db = toasty::Db::builder()
            .models(toasty::models!(models::Family))
            .connect("sqlite::memory:")
            .await
            .unwrap();

        db.push_schema().await.unwrap();

        AppState { db }
    }

    async fn send(app: &Router, req: Request<Body>) -> (StatusCode, Value) {
        let response = app.clone().oneshot(req).await.unwrap();
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json = if body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body).unwrap()
        };
        (status, json)
    }

    fn json_request(method: &str, uri: &str, body: Value) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string()))
            .unwrap()
    }

    fn plain_request(method: &str, uri: &str) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap()
    }

    async fn create_family(app: &Router, id: u64) {
        let req = json_request(
            "POST",
            "/families",
            json!({ "name": format!("Family {id}"), "summary": "s" }),
        );
        let (status, _) = send(app, req).await;
        assert_eq!(status, StatusCode::CREATED);
    }

    #[tokio::test]
    async fn health_responds_ok() {
        let state = test_app().await;
        let app = app(state);

        let (status, body) = send(&app, plain_request("GET", "/")).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, json!({ "status": "ok" }));
    }

    #[tokio::test]
    async fn creates_and_lists_families() {
        let state = test_app().await;
        let app = app(state);

        let (status, body) = send(
            &app,
            json_request(
                "POST",
                "/families",
                json!({ "name": "The Simpsons", "summary": "Springfield" }),
            ),
        )
        .await;

        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(body["name"], "The Simpsons");
        assert_eq!(body["summary"], "Springfield");

        let (status, body) = send(&app, plain_request("GET", "/families")).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.as_array().unwrap().len(), 1);
        assert_eq!(body[0]["id"], 1);
    }

    #[tokio::test]
    async fn gets_family_by_id() {
        let state = test_app().await;
        let app = app(state);
        create_family(&app, 1).await;

        let (status, body) = send(&app, plain_request("GET", "/families/1")).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["name"], "Family 1");
    }

    #[tokio::test]
    async fn get_returns_404_for_missing_family() {
        let state = test_app().await;
        let app = app(state);

        let (status, _) = send(&app, plain_request("GET", "/families/42")).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn updates_family() {
        let state = test_app().await;
        let app = app(state);
        create_family(&app, 1).await;

        let (status, body) = send(
            &app,
            json_request(
                "PUT",
                "/families/1",
                json!({ "name": "Simpsons 2", "summary": null }),
            ),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["name"], "Simpsons 2");
        assert!(body["summary"].is_null());
    }

    #[tokio::test]
    async fn soft_delete_hides_family_but_keeps_row() {
        let state = test_app().await;
        let app = app(state.clone());
        create_family(&app, 1).await;

        let (status, _) = send(&app, plain_request("DELETE", "/families/1")).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        let (status, body) = send(&app, plain_request("GET", "/families")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.as_array().unwrap().len(), 0);

        let (status, _) = send(&app, plain_request("GET", "/families/1")).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let (status, _) = send(&app, plain_request("DELETE", "/families/1")).await;
        assert_eq!(status, StatusCode::NO_CONTENT);

        let mut db = state.db.clone();
        let all: Vec<models::Family> = models::Family::all().exec(&mut db).await.unwrap();
        assert_eq!(all.len(), 1);
        assert!(all[0].deleted_at.is_some());
    }

    #[tokio::test]
    async fn delete_returns_404_for_missing_family() {
        let state = test_app().await;
        let app = app(state);

        let (status, _) = send(&app, plain_request("DELETE", "/families/42")).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}
