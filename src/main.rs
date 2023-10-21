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
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Clone)]
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
                id: 0,
                tweet: "hello".to_string(),
                created_at_epoch_ms: 0,
            },
            Tweet {
                id: 1,
                tweet: "world".to_string(),
                created_at_epoch_ms: 1,
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
async fn root(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    #[derive(Template)]
    #[template(path = "index.html")]
    struct IndexTemplate<'a> {
        initial_tweets: &'a Vec<Tweet>,
    }

    let tweets = state.tweets.write().await;

    // Sort the tweets by created_at_epoch_ms
    let mut tweets = tweets.clone();
    tweets.sort_by(|a, b| b.created_at_epoch_ms.cmp(&a.created_at_epoch_ms));

    let index = IndexTemplate {
        initial_tweets: &tweets,
    };
    let rendered = index.render().map_err(|e| {
        tracing::error!("Failed to render template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(rendered))
}

#[derive(Deserialize, Serialize)]
struct CreateTweetRequest {
    tweet: String,
}
async fn create_tweet(
    State(state): State<AppState>,
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Form(payload): Form<CreateTweetRequest>,
) -> Result<Html<String>, StatusCode> {
    tracing::info!("tweet: {}", payload.tweet);

    // insert the new tweet into the database
    let current_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| StatusCode::INTERNAL_SERVER_ERROR)?
        .as_millis();

    // Lock the tweets
    let mut tweets = state.tweets.write().await;

    let tweet = Tweet {
        id: tweets.len(),
        tweet: payload.tweet,
        created_at_epoch_ms: current_time,
    };

    // Insert into tweets...

    // Respond with the new tweet list...
    let tweet_template = TweetTemplate { tweet: &tweet };
    let rendered = tweet_template.render().map_err(|e| {
        tracing::error!("Failed to render template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tweets.push(tweet);

    Ok(Html(rendered))
}

#[derive(Template)]
#[template(path = "tweet.html")]
struct TweetTemplate<'a> {
    tweet: &'a Tweet,
}
