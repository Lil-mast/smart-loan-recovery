use actix_identity::Identity;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};

use crate::db::Db;
use crate::error::{AppError, AppResult};
use crate::user::UserManager;

#[derive(Deserialize)]
pub struct DemoLoginReq {
    user_id: String,
}

#[derive(Serialize)]
struct DemoLoginRes {
    user_id: String,
    role: String,
    name: String,
}

pub async fn demo_login(
    data: web::Json<DemoLoginReq>,
    req: HttpRequest,
    db: web::Data<Db>,
) -> AppResult<ActixResult<HttpResponse>> {
    let user_id = data.user_id.trim().to_string();
    let mgr = UserManager::new(&db);
    let user = mgr
        .get_user(&user_id)
        .map_err(AppError::Database)?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Identity::login(&req.extensions(), user.id.clone()).map_err(|_| AppError::AuthRequired)?;

    let role = format!("{:?}", user.role).to_lowercase();

    Ok(Ok(HttpResponse::Ok().json(DemoLoginRes {
        user_id: user.id,
        role,
        name: user.name,
    })))
}
