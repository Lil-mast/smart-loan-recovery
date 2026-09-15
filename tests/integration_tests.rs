use actix_identity::IdentityMiddleware;
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use actix_web::cookie::Key;
use actix_web::{http::StatusCode, test, web, App};
use lendwise_recovery::api::*;
use lendwise_recovery::config::Config;
use lendwise_recovery::db::Db;
use serde_json::json;
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

fn identity_wrap() -> (
    IdentityMiddleware,
    SessionMiddleware<CookieSessionStore>,
) {
    let key = Key::generate();
    (
        IdentityMiddleware::default(),
        SessionMiddleware::builder(CookieSessionStore::default(), key)
            .cookie_secure(false)
            .build(),
    )
}

#[actix_web::test]
async fn test_user_registration() {
    let unique_mail = format!("test.user.{}@example.com", uuid::Uuid::new_v4());
    let db = fresh_test_db();
    let (id_mw, sess_mw) = identity_wrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .wrap(id_mw)
            .wrap(sess_mw)
            .route("/users", web::post().to(register_user)),
    )
    .await;

    let lender_req = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({
            "name": "Ada Okonkwo",
            "role": "lender",
            "organization": "Okoye Credit"
        }))
        .to_request();
    let lender_resp = test::call_service(&app, lender_req).await;
    assert_eq!(lender_resp.status(), StatusCode::OK);
    let lender: serde_json::Value = test::read_body_json(lender_resp).await;
    let lender_id = lender.get("id").and_then(|v| v.as_str()).expect("lender id");

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&json!({
            "name": "Test User",
            "role": "borrower",
            "email": unique_mail,
            "lender_id": lender_id
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("id").is_some());
    assert_eq!(
        body.get("email").and_then(|e| e.as_str()),
        Some(unique_mail.as_str())
    );
    assert_eq!(
        body.get("lender_id").and_then(|e| e.as_str()),
        Some(lender_id)
    );
}

#[actix_web::test]
async fn test_get_users() {
    let db = fresh_test_db();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/users", web::get().to(get_users)),
    )
    .await;

    let req = test::TestRequest::get().uri("/users").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(body.is_empty());
}

#[actix_web::test]
async fn test_invalid_user_registration() {
    let db = fresh_test_db();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db))
            .route("/users", web::post().to(register_user)),
    )
    .await;

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
