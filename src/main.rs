use std::io::{Error, ErrorKind};
use std::str::FromStr;
use axum::{routing::{get, post}, http::StatusCode, response::IntoResponse, Json, Router, http};
use std::net::SocketAddr;
use axum::http::{HeaderValue, Method};
use axum::response::Response;
use tracing_subscriber;
use tracing::{debug, info, error, Level};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    // initialize tracing
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .compact()
        .with_level(true)
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // enable cors
    let cors = CorsLayer::new()
        .allow_methods([Method::DELETE])
        .allow_origin("http://localhost:63342".parse::<HeaderValue>().unwrap());

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/hello", get(|| async {format!("hello world!")}))
        // `POST /users` goes to `create_user`
        .route("/question", get(get_question))
        .layer(cors);

    // run our app with hyper
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

}

#[derive(Debug)]
struct InvalidId;

impl IntoResponse for InvalidId {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Cannot parse id into integer"),
        )
            .into_response()
    }
}

async fn get_question() -> Result<impl IntoResponse, impl IntoResponse> {
    let question = Question::new(
        QuestionId::from_str("1").unwrap(),
       "First Question".to_string(),
       "How are u?".to_string(),
        Some(vec!("faq".to_string())),
    );

    match question.id.0.parse::<u32>() {
        Ok(_) => Ok((StatusCode::OK, Json(question))),
        Err(e) => {
            error!("cannot parse id due to: {}", e);
            Err(InvalidId)
        },
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>)
           -> Self {
        Question{
            id,
            title,
            content,
            tags,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct QuestionId(String);

impl FromStr for QuestionId {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(Error::new(ErrorKind::InvalidInput, "No id provided"));
        }
        Ok(QuestionId(s.to_string()))
    }
}


