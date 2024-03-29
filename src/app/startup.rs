use std::convert::TryInto;
use std::net::TcpListener;

use actix_web::dev::Server;
use actix_web::{
    web,
    App,
    HttpServer,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use url::Url;

use crate::app::configuration::{
    DatabaseSettings,
    EmailClientSettings,
    Settings,
};
use crate::domain::{
    AppBaseUrl,
    SubscriberEmail,
};
use crate::email_client::EmailClient;
use crate::routes::*;

pub struct NewsletterApp {
    pub server: Result<Server, std::io::Error>,
    pub port: u16,
}

impl NewsletterApp {
    pub async fn from(configuration: Settings) -> Result<NewsletterApp, std::io::Error> {
        let tcp_listener = TcpListener::bind(configuration.application.binding_address())?;
        let port = tcp_listener.local_addr().unwrap().port();
        let postgres_pool =
            web::Data::new(NewsletterApp::postgres_pool(configuration.database).await);
        let email_client = web::Data::new(NewsletterApp::email_client(configuration.email_client));
        let app_base_url = web::Data::new(AppBaseUrl(configuration.application.base_url));

        // HttpServer handles all transport level concerns
        let server = HttpServer::new(move || {
            // App is where all the application logic lives: routing, middlewares, request
            // handlers, etc.
            App::new()
                .wrap(TracingLogger::default())
                // route takes two parameters: path and route
                // path -> template string
                // route -> Route Struct implements the Route trait.
                // The route combines the handler with a set of guards:
                // the check method verifies the guards conditions are met
                // and eventually call the handler
                .route("/health_check", web::get().to(health_check))
                // we need to clone the input connection  because the current closure will be called
                // multiple times (in fact it is of type Fn not FnOnce) and the input connection
                // would not be available anymore at the next call otherwise.
                .route("/subscriptions", web::post().to(subscribe))
                .route("/subscriptions/confirm", web::get().to(confirm))
                .route("/newsletters", web::post().to(newsletters))
                .app_data(postgres_pool.clone())
                .app_data(email_client.clone())
                .app_data(app_base_url.clone())
        })
        .backlog(configuration.application.max_pending_connections)
        .listen(tcp_listener)
        .map(HttpServer::run);
        Ok(NewsletterApp { server, port })
    }

    pub async fn postgres_pool(database_config: DatabaseSettings) -> PgPool {
        PgPoolOptions::new()
            .connect_timeout(std::time::Duration::from_secs(
                database_config.connect_timeout_seconds,
            ))
            .max_connections(database_config.max_db_connections)
            .connect_with(database_config.database_connection_options())
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "Error: {:?} creating Postgresql pool with settings:{:?}",
                    e, database_config
                )
            })
    }

    fn email_client(client_config: EmailClientSettings) -> EmailClient {
        let base_url = Url::parse(&client_config.base_url).unwrap_or_else(|e| {
            panic!("Error: {} parsing base url: {}", e, client_config.base_url)
        });

        let sender_email: SubscriberEmail = client_config
            .sender_email
            .try_into()
            .unwrap_or_else(|e| panic!("Error: {} parsing sender email from config", e));

        EmailClient::new(
            base_url,
            sender_email,
            client_config.token,
            client_config.timeout_secs,
        )
        .unwrap_or_else(|e| panic!("Error: {} creating EmailClient", e))
    }
}
