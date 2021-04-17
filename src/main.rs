use newsletter::app::load_configuration;
use newsletter::app::setup_tracing;
use newsletter::app::NewsletterApp;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    setup_tracing("newsletter".into(), "info".into());

    let configuration =
        load_configuration().unwrap_or_else(|e| panic!("error loading configuration: {}", e));

    let app = NewsletterApp::from(configuration).await?;
    app.server?.await
}
