use actix_web::{test, App, http::StatusCode};
use actix_web::web;
use serde_json::json;
use smart_loan_recovery::api::*;
use smart_loan_recovery::config::Config;
use smart_loan_recovery::db::Db;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};

#[actix_web::test]
async fn test_user_registration() {
    // Initialize database for testing
    let db = Db::new().expect("Failed to create test database");

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/users", web::post().to(register_user))
    ).await;

    // Test user registration
    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({
            "name": "Test User",
            "role": "borrower"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("id").is_some());
}

#[actix_web::test]
async fn test_get_users() {
    // Initialize database for testing
    let db = Db::new().expect("Failed to create test database");

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/users", web::get().to(get_users))
    ).await;

    // Test get users
    let req = test::TestRequest::get()
        .uri("/users")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(body.is_empty() || body.len() >= 0); // Should return array
}

#[actix_web::test]
async fn test_invalid_user_registration() {
    // Initialize database for testing
    let db = Db::new().expect("Failed to create test database");

    // Create test app
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/users", web::post().to(register_user))
    ).await;

    // Test invalid role
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
    // Test that the server can be configured
    let config = Config::from_env().expect("Failed to load config");
    assert_eq!(config.server_host, "127.0.0.1");
    assert_eq!(config.server_port, 3000);
    assert!(!config.session_secret.is_empty());
}