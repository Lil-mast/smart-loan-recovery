use actix_web::{test, App, http::StatusCode};
use actix_web::web;
use lendwise_recovery::config::Config;
use serde_json::json;
use lendwise_recovery::api::*;
use lendwise_recovery::db::Db;
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn fresh_test_db() -> Db {
    let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let mut path = std::env::temp_dir();
    path.push(format!("lendwise_test_{}.db", counter));
    let path_str = path.to_str().unwrap().to_string();
    let _ = std::fs::remove_file(&path_str);
    Db::new_with_path(&path_str).expect("Failed to create test database")
}

#[actix_web::test]
async fn test_user_registration() {
    let unique_mail = format!("test.user.{}@example.com", uuid::Uuid::new_v4());
    let db = fresh_test_db();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/users", web::post().to(register_user))
    ).await;

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({
            "name": "Test User",
            "role": "borrower",
            "email": unique_mail,
            "lender_name": "Demo Lender"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("id").is_some());
    assert_eq!(body.get("email").and_then(|e| e.as_str()), Some(unique_mail.as_str()));
}

#[actix_web::test]
async fn test_get_users() {
    let db = fresh_test_db();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/users", web::get().to(get_users))
    ).await;

    let req = test::TestRequest::get()
        .uri("/users")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let _body: Vec<serde_json::Value> = test::read_body_json(resp).await;
}

#[actix_web::test]
async fn test_invalid_user_registration() {
    let db = fresh_test_db();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/users", web::post().to(register_user))
    ).await;

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({
            "name": "Test User",
            "role": "invalid_role"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_server_startup() {
    let config = Config::from_env().expect("Failed to load config");
    assert_eq!(config.server_host, "127.0.0.1");
    assert_eq!(config.server_port, 3000);
    assert!(!config.session_secret.is_empty());
}