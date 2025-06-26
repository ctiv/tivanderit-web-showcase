#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use sqlx::migrate::Migrator;
    use tivanderit::app::*;
    use tower_http::compression::CompressionLayer;

    use ssr::db;

    let mut conn = db().await.expect("couldn't connect to DB");

    let migrations_path = std::env::var("MIGRATIONS_PATH").expect("MIGRATIONS_PATH must be set - aborting startup because the migration is required");

    let m = Migrator::new(std::path::Path::new(&migrations_path)).await.expect("could't find the migrations");
    m.run(&mut conn).await.expect("Couldn't run migrations");
   
    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options)
        .layer(CompressionLayer::new());

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
