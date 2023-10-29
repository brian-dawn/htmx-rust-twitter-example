// Define the module.

use std::sync::Arc;

use axum::{
    extract::State,
    routing::{get, post},
    Router,
};
use maud::{html, Markup};
use tokio::sync::RwLock;

#[derive(Clone)]
struct AppState {
    //tweets: Arc<RwLock<Vec<Tweet>>>,
    counter: Arc<RwLock<u64>>,
}

pub fn build_routes() -> Router<()> {
    let state = AppState {
        counter: Arc::new(RwLock::new(0)),
    };

    let routes = Router::new()
        .route("/", get(root))
        .route("/submit", post(submit))
        .with_state(state);

    routes
}

/// A simple header.
fn header() -> Markup {
    html! {
        head {
            title { "Maud example" }
            // Include htmx.
            script src="https://unpkg.com/htmx.org/dist/htmx.min.js" {}
        }
    }
}

async fn root() -> Markup {
    html! {

        html {
            (header())

            div {

                button id="button" hx-post="/m/submit" hx-target="#counter" {
                    "Submit"

                }

                div id="counter" {
                    "0"
                }
            }
        }
    }
}

async fn submit(State(state): State<AppState>) -> Markup {
    let mut counter = state.counter.write().await;
    *counter += 1;
    html! {
        (counter)
    }
}
