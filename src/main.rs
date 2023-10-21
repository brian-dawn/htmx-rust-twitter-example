use askama::Template;
use axum::{
    extract::{Query, State},
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
        Tweet {
            id: 2,
            tweet: "Just had the best cup of coffee ☕️ #MorningRituals".to_string(),
            created_at_epoch_ms: 2,
        },
        Tweet {
            id: 3,
            tweet: "Watching the sunset and reflecting on life 🌇 #DeepThoughts".to_string(),
            created_at_epoch_ms: 3,
        },
        Tweet {
            id: 4,
            tweet: "Anyone else excited for the new season of The Crown? 📺 #BingeWatching".to_string(),
            created_at_epoch_ms: 4,
        },
        Tweet {
            id: 5,
            tweet: "Had an amazing hike today! Nature truly heals 🌲🚶‍♂️ #NatureLover".to_string(),
            created_at_epoch_ms: 5,
        },
        Tweet {
            id: 6,
            tweet: "Grateful for my friends and family. They've always got my back ❤️ #Blessed".to_string(),
            created_at_epoch_ms: 6,
        },
        Tweet {
            id: 7,
            tweet: "Workout complete! 💪 Feeling stronger every day. #FitnessGoals".to_string(),
            created_at_epoch_ms: 7,
        },
        Tweet {
            id: 8,
            tweet: "I've been reading a lot about mindfulness lately. It's truly life-changing. #MindfulLiving".to_string(),
            created_at_epoch_ms: 8,
        },
        Tweet {
            id: 9,
            tweet: "The new album from Imagine Dragons is 🔥! #MusicLover".to_string(),
            created_at_epoch_ms: 9,
        },
        Tweet {
            id: 10,
            tweet: "Travel plans for 2023: Japan, Greece, and Canada. Can't wait! ✈️ #Wanderlust".to_string(),
            created_at_epoch_ms: 10,
        },
        Tweet {
            id: 11,
            tweet: "The book 'Atomic Habits' is a game changer. Highly recommend! 📚 #BookRecommendations".to_string(),
            created_at_epoch_ms: 11,
        },
        Tweet {
            id: 12,
            tweet: "Spending some quality time with my cat 🐱. Pets truly are a source of joy. #CatLover".to_string(),
            created_at_epoch_ms: 12,
        },
        Tweet {
            id: 13,
            tweet: "I'm considering taking a digital detox for a week. Has anyone tried it? #DigitalDetox".to_string(),
            created_at_epoch_ms: 13,
        },
        Tweet {
            id: 14,
            tweet: "I've recently taken up pottery. It's such a therapeutic hobby! 🏺#NewHobbies".to_string(),
            created_at_epoch_ms: 14,
        }

        ])),
    };

    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/tweet", post(create_tweet))
        .route("/tweet", get(get_lazy_tweets))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

// basic handler that responds with a static string
async fn root(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let index = IndexTemplate {};
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

    let tweet_template = TweetTemplate { tweet: &tweet };
    let rendered = tweet_template.render().map_err(|e| {
        tracing::error!("Failed to render template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tweets.push(tweet);

    // Simulate network delay...
    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

    Ok(Html(rendered))
}

#[derive(Template)]
#[template(path = "tweet.html")]
struct TweetTemplate<'a> {
    tweet: &'a Tweet,
}

#[derive(Template)]
#[template(path = "lazy_tweets.html")]
struct LazyTweetsTemplate<'a> {
    tweets: &'a [Tweet],
    page: usize,
    more_tweets: bool,
}

#[derive(Deserialize)]
struct LazyTweetsQueryParams {
    page: usize,
    size: usize,
}

async fn get_lazy_tweets(
    State(state): State<AppState>,
    // Get query params...
    query: Query<LazyTweetsQueryParams>,
) -> Result<Html<String>, StatusCode> {
    let tweets = state.tweets.read().await;

    // Sort the tweets by created_at_epoch_ms
    let mut tweets = tweets.clone();
    tweets.sort_by(|a, b| b.created_at_epoch_ms.cmp(&a.created_at_epoch_ms));

    // Only show the tweets for the current page.
    let tweets = tweets
        .into_iter()
        .skip(query.page * query.size)
        .take(query.size)
        .collect::<Vec<_>>();

    let tweets_template = LazyTweetsTemplate {
        tweets: &tweets,
        page: query.page,
        more_tweets: tweets.len() > query.size * query.page,
    };
    let rendered = tweets_template.render().map_err(|e| {
        tracing::error!("Failed to render template: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Simulate network delay...
    tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

    Ok(Html(rendered))
}
