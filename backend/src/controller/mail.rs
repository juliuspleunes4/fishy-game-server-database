use rocket::{post, routes, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::service::mail::MailService;

/// Request body for creating a mail.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct CreateMailRequest {
    pub mail_id: Uuid,
    pub sender_id: Uuid,
    pub receiver_ids: Vec<Uuid>,
    pub title: String,
    pub message: String,
}

/// Request body for deleting a mail.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct DeleteMailRequest {
    pub user_id: Uuid,
    pub mail_id: Uuid,
}

/// Request body for reading a mail.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ReadMailRequest {
    pub user_id: Uuid,
    pub mail_id: Uuid,
    pub read: bool,
}

/// Request body for archiving a mail.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct ArchiveMailRequest {
    pub user_id: Uuid,
    pub mail_id: Uuid,
    pub archived: bool,
}

#[utoipa::path(
    post,
    path = "/mail/create",
    request_body = CreateMailRequest,
    responses(
        (status = 201, description = "Mail created successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Creates a mail",
    operation_id = "createMail",
    tag = "Mails"
)]
#[post("/create", data = "<payload>")]
async fn create_mail(
    payload: Json<CreateMailRequest>,
    mail_service: &State<Arc<dyn MailService>>,
) -> Json<bool> {
    match mail_service
        .create(
            payload.mail_id,
            payload.sender_id,
            payload.receiver_ids.clone(),
            payload.title.clone(),
            payload.message.clone(),
        )
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/mail/delete",
    request_body = DeleteMailRequest,
    responses(
        (status = 201, description = "Mail successfully deleted", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Deletes a mail",
    operation_id = "deleteMail",
    tag = "Mails"
)]
#[post("/delete", data = "<payload>")]
async fn delete_mail(
    payload: Json<DeleteMailRequest>,
    mail_service: &State<Arc<dyn MailService>>,
) -> Json<bool> {
    match mail_service.delete(payload.user_id, payload.mail_id).await {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/mail/change_read_state",
    request_body = ReadMailRequest,
    responses(
        (status = 201, description = "Mail read state changed successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Changes the read state",
    operation_id = "readStateMail",
    tag = "Mails"
)]
#[post("/change_read_state", data = "<payload>")]
async fn change_read_state(
    payload: Json<ReadMailRequest>,
    mail_service: &State<Arc<dyn MailService>>,
) -> Json<bool> {
    match mail_service
        .change_read_state(payload.user_id, payload.mail_id, payload.read)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

#[utoipa::path(
    post,
    path = "/mail/archive_state",
    request_body = ArchiveMailRequest,
    responses(
        (status = 201, description = "Mail archive state changed successfully", body = bool),
        (status = 400, description = "Invalid input data"),
        (status = 500, description = "Internal server error")
    ),
    description = "Changes the archived state",
    operation_id = "archiveStateMail",
    tag = "Mails"
)]
#[post("/archive_state", data = "<payload>")]
async fn change_archive_state(
    payload: Json<ArchiveMailRequest>,
    mail_service: &State<Arc<dyn MailService>>,
) -> Json<bool> {
    match mail_service
        .change_archive_state(payload.user_id, payload.mail_id, payload.archived)
        .await
    {
        Ok(()) => Json(true),
        Err(_) => Json(false),
    }
}

// Combine all the user routes.
pub fn mail_routes() -> Vec<rocket::Route> {
    routes![
        create_mail,
        delete_mail,
        change_read_state,
        change_archive_state
    ]
}
