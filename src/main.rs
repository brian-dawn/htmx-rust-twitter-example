use askama::Template;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/tweet", post(create_tweet));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// basic handler that responds with a static string
async fn root() -> Result<Html<String>, StatusCode> {
    #[derive(Template)] // this will generate the code...
    #[template(path = "index.html")] // using the template in this path, relative
    struct IndexTemplate<'a> {
        // the name of the struct can be anything
        name: &'a str, // the field name should match the variable name
                       // in your template
    }

    // Generate askama template
    let index = IndexTemplate { name: "world" };
    let rendered = index.render().map_err(|e| {
        tracing::error!("Failed to render template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(rendered))
}

#[derive(Deserialize, Serialize)]
struct CreateTweet {
    tweet: String,
}
async fn create_tweet(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Form(payload): Form<CreateTweet>,
) -> (StatusCode, Json<String>) {
    tracing::debug!("tweet: {}", payload.tweet);
    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json("hi".to_string()))
}
