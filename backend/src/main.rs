use crate::controller::authentication::authentication_routes;
use crate::controller::shop::shop_routes;
use crate::controller::stats::stats_routes;
use crate::controller::user::*;
use crate::docs::ApiDoc;
use crate::domain::User;
use crate::repository::friends::FriendRepositoryImpl;
use crate::repository::user::UserRepositoryImpl;
use crate::service::authentication::*;
use crate::service::shop::ShopService;
use crate::service::shop::ShopServiceImpl;
use crate::service::user::UserService;
use crate::service::user::UserServiceImpl;
use crate::AuthenticationService;
use controller::data::data_routes;
use controller::effects::routes as effects_routes;
use controller::friends::friend_routes;
use controller::inventory::inventory_routes;
use controller::mail::mail_routes;
use dotenv::dotenv;
use repository::data::DataRepositoryImpl;
use repository::effects::EffectsRepositoryImpl;
use repository::inventory::InventoryRepositoryImpl;
use repository::mail::MailRepositoryImpl;
use repository::stats::StatsRepositoryImpl;
use rocket::http::Status;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Outcome;
use rocket::Config;
use rocket::Request;
use rocket::State;
use rocket_cors::AllowedOrigins;
use rocket_cors::{AllowedHeaders, CorsOptions};
use sea_orm::Database;
use service::data::DataService;
use service::data::DataServiceImpl;
use service::effects::EffectsService;
use service::effects::EffectsServiceImpl;
use service::friends::FriendService;
use service::friends::FriendServiceImpl;
use service::inventory::InventoryService;
use service::inventory::InventoryServiceImpl;
use service::mail::MailService;
use service::mail::MailServiceImpl;
use service::stats::StatsService;
use service::stats::StatsServiceImpl;
use std::env;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

extern crate rocket;

// Import all the different layers that make up the backend.
pub mod controller;
pub mod docs;
pub mod domain;
pub mod entity;
pub mod repository;
pub mod service;
pub mod utils;

// This is responsible for protecting the endpoints with JWT.
//
// If an endpoint requires a User as argument, this piece of code is called.
// It reades the header if a JWT is given.
// it reades the user_id from the jwt and recieves the user from the database.
// Then that user is being parsed into the endpoint.
//
// If the JWT is not in the header, it returns access denied.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        // Recieve the authentication service from rocket for recieving and validating the jwt.
        let authentication_service = request
            .guard::<&State<Arc<dyn AuthenticationService>>>()
            .await
            .unwrap();

        match request.headers().get_one("Authorization") {
            None => Outcome::Error((Status::Unauthorized, ())),
            Some(autherisation_header) => match autherisation_header.strip_prefix("Bearer ") {
                None => Outcome::Error((Status::Unauthorized, ())),
                Some(jwt) => match authentication_service.verify_jwt(jwt).await {
                    Ok(Some(user)) => request::Outcome::Success(user.clone()),
                    Ok(None) => Outcome::Error((Status::Unauthorized, ())),
                    Err(_) => Outcome::Error((Status::Unauthorized, ())),
                },
            },
        }
    }
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    // Read the database path from enviousment variables.
    // You can set them in bash like so:
    // ```bash
    // export DATABASE_URL="postgres://username:password@host:port/database_name"
    // ```
    //
    // And same for the secret key that is being used to sign the JWT.
    //
    // ```bash
    // export SECRET_KEY="My secret key :)"
    // ```
    // the program will panic if these are not set.
    println!("Starting backend...");
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY must be set for generating JWT");
    let port = env::var("PORT").expect("Connection port must be provided in the ENV");
    let listening_ip = env::var("IP_ADDR").expect("Listening ip must be provided in the ENV");

    // Connect to postgres database.
    let db = Database::connect(&database_url)
        .await
        .expect("Failed to create SeaORM database connection");

    // Alow request from any origin.
    // You should customize this if you want to make your backend more secure.
    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::all(), // Allow all origins
        allowed_headers: AllowedHeaders::some(&["Authorization", "Content-Type"]),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Failed to create CORS configuration");

    // Build the repository layers and service layers.
    let user_repository = UserRepositoryImpl::new();
    let data_repository = DataRepositoryImpl::new();
    let effects_repository = EffectsRepositoryImpl::new();
    let friends_repository = FriendRepositoryImpl::new();
    let stats_repository = StatsRepositoryImpl::new();
    let mail_repository = MailRepositoryImpl::new();
    let inventory_repository = InventoryRepositoryImpl::new();

    let user_service: Arc<dyn UserService> = Arc::new(UserServiceImpl::new(
        db.clone(),
        user_repository.clone(),
        stats_repository.clone(),
        inventory_repository.clone(),
        secret_key.clone(),
    ));

    let authentication_service: Arc<dyn AuthenticationService> = Arc::new(
        AuthenticationServiceImpl::new(db.clone(), user_repository.clone(), secret_key.clone()),
    );

    let data_service: Arc<dyn DataService> =
        Arc::new(DataServiceImpl::new(db.clone(), data_repository.clone()));

    let friend_service: Arc<dyn FriendService> = Arc::new(FriendServiceImpl::new(
        db.clone(),
        friends_repository.clone(),
    ));

    let stats_service: Arc<dyn StatsService> =
        Arc::new(StatsServiceImpl::new(db.clone(), stats_repository.clone()));

    let mail_service: Arc<dyn MailService> =
        Arc::new(MailServiceImpl::new(db.clone(), mail_repository.clone()));

    let inventory_service: Arc<dyn InventoryService> = Arc::new(InventoryServiceImpl::new(
        db.clone(),
        inventory_repository.clone(),
    ));

    let effects_service: Arc<dyn EffectsService> =
        Arc::new(EffectsServiceImpl::new(db.clone(), effects_repository.clone()));

    let shop_service: Arc<dyn ShopService> = Arc::new(ShopServiceImpl::new(
        db.clone(),
        stats_repository.clone(),
        inventory_repository.clone(),
    ));

    // Add here more repositories and services when your backend grows.

    // Set rocket configuration.
    let config = Config {
        port: port
            .parse()
            .expect(&format!("could not parse port: {:?}", port)),
        address: listening_ip
            .parse()
            .expect(&format!("Could not parse listening ip: {:?}", listening_ip)),
        ..Config::debug_default()
    };

    // And combine everything into your beautiful backend.
    let _rocket = rocket::custom(config)
        // Here the service layers are given as arguments to the endpoints.
        // Add more service layers when you backend grows.
        .manage(user_service)
        .manage(authentication_service)
        .manage(stats_service)
        .manage(mail_service)
        .manage(inventory_service)
        .manage(data_service)
        .manage(friend_service)
        .manage(effects_service)
        .manage(shop_service)
        // expose swagger ui.
        // Go to http://localhost:8000/docs to view your endpoint documentation.
        .mount(
            "/",
            SwaggerUi::new("/docs/<_..>").url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        // Mount all your routes here.
        .mount("/account", user_routes())
        .mount("/login", authentication_routes())
        .mount("/stats", stats_routes())
        .mount("/mail", mail_routes())
        .mount("/inventory", inventory_routes())
        .mount("/data", data_routes())
        .mount("/friend", friend_routes())
        .mount("/effects", effects_routes())
        .mount("/shop", shop_routes())
        .attach(cors)
        .launch()
        .await?;

    Ok(())
}
