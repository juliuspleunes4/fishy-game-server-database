use crate::domain::LoginResponse;
use crate::domain::User;
use crate::service::user::UserService;
use rocket::get;
use rocket::post;
use rocket::response::status;
use rocket::routes;
use rocket::serde::json::Json;
use rocket::State;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Arc;
use utoipa::ToSchema;

/// Request body for creating a user.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

/// Request body for changing a password.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ChangePasswordRequest {
    pub username: String,
    pub new_password: String,
}

/// Request body for requesting a players username.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct RetreiveUsernameRequest {
    pub email: String,
}

// Utoipa is the crate that generates swagger documentation for your endpoints.
// The documentation for each endpoint is combined in docs.rs
// Make sure to add your endpoint in docs.rs when you write new endpoints.
#[utoipa::path(
    post,
    path = "/account/register",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Creates a user. The email and username should be unique.",
    operation_id = "createUser",
    tag = "Users"
)]
#[post("/register", data = "<payload>")]
async fn create_user(
    payload: Json<CreateUserRequest>,
    user_service: &State<Arc<dyn UserService>>,
) -> Json<LoginResponse> {
    match user_service
        .create(
            payload.username.clone(),
            payload.email.clone(),
            payload.password.clone(),
        )
        .await
    {
        Ok(res) => Json(res),
        Err(_) => Json(LoginResponse {
            code: 401,
            jwt: String::from(""),
        }),
    }
}

#[utoipa::path(
    post,
    path = "/account/retreive_username",
    request_body = RetreiveUsernameRequest,
    responses(
        (status = 201, description = "Username send successfull", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Sends the username of the account the email belongs to to the mail address",
    operation_id = "retreiveUsername",
    tag = "Users"
)]
#[post("/retreive_username", data = "<payload>")]
async fn retreive_username(
    payload: Json<RetreiveUsernameRequest>,
    user_service: &State<Arc<dyn UserService>>,
) -> Json<bool> {
    match user_service.retreive_username(payload.email.clone()).await {
        Ok(res) => Json(res),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/account/change_password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 201, description = "Changed password", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Changes a users password",
    operation_id = "changePassword",
    tag = "Users"
)]
#[post("/change_password", data = "<payload>")]
async fn change_password(
    payload: Json<ChangePasswordRequest>,
    user_service: &State<Arc<dyn UserService>>,
) -> Json<bool> {
    // TODO: We need a way to verify a user is actually changing the password of it's own account
    match user_service
        .change_password(payload.username.clone(), payload.new_password.clone())
        .await
    {
        Ok(res) => Json(res),
        Err(_) => Json(false),
    }
}

/// Response for recieving user information.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct GetUserResponse {
    name: String,
    email: String,
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 201, description = "User recieved successfully", body = GetUserResponse),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Recieve user details.",
    operation_id = "getUser",
    tag = "Users",
    security(
        ("jwt_auth" = [])
    )
)]
#[get("/")]
async fn get_user(user: User) -> Result<Json<GetUserResponse>, status::Custom<String>> {
    Ok(Json(GetUserResponse {
        email: user.email,
        name: user.name,
    }))
}

// Combine all the user routes.
pub fn user_routes() -> Vec<rocket::Route> {
    routes![create_user, retreive_username, change_password, get_user]
}
