use actix_web::{web, App, HttpServer};
use source_control_rest_interface::endpoints::organization::create::create_organization;

#[actix_web::main]
async fn main() -> io::Result<()> {
    HttpServer::new(move || App::new().route("/organization", web::post().to(create_organization)))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
