use askama::Template;
use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Form, Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
struct Tweet {
    id: usize,
    tweet: String,
    created_at_epoch_ms: u128,
}

#[derive(Clone)]
struct AppState {
    tweets: Arc<RwLock<Vec<Tweet>>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    tracing_subscriber::fmt::init();

    let state = AppState {
        tweets: Arc::new(RwLock::new(vec![
            Tweet {
                id: 1,
                tweet: "hello".to_string(),
                created_at_epoch_ms: 0,
            },
            Tweet {
                id: 2,
                tweet: "world".to_string(),
                created_at_epoch_ms: 0,
            },
        ])),
    };

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/tweet", post(create_tweet))
        .with_state(state);

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
    State(state): State<AppState>,
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Form(payload): Form<CreateTweet>,
) -> (StatusCode, Json<String>) {
    tracing::debug!("tweet: {}", payload.tweet);

    // insert the new tweet into the database
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    // Lock the tweets
    let mut tweets = state.tweets.write().await;

    let tweet = Tweet {
        id: tweets.len(),
        tweet: payload.tweet,
        created_at_epoch_ms: current_time,
    };

    let tweet_str = format!("{:?}", &tweet);

    // Insert into tweets...
    tweets.push(tweet);

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(format!("{:?}", tweet_str)))
}
