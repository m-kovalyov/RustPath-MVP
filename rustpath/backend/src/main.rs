mod domain;
mod error;
mod repository;
mod runner;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::IntoResponse,
    routing::{get, post, put},
};
use domain::{Course, Lesson, LessonSummary, Progress};
use error::AppError;
use repository::{LearningRepository, PgLearningRepository};
use runner::{CodeEvaluator, IsolatedRunner};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, Semaphore};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    course: Arc<Course>,
    repository: Arc<dyn LearningRepository>,
    evaluator: Arc<dyn CodeEvaluator>,
    cookie_secure: bool,
    origin: String,
    slots: Arc<Semaphore>,
    rate: Arc<Mutex<HashMap<Uuid, (Instant, u32)>>>,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rustpath_api=info,tower_http=info".into()),
        )
        .init();
    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    sqlx::migrate!().run(&db).await?;
    let state = AppState {
        repository: Arc::new(PgLearningRepository(db.clone())),
        db,
        course: Arc::new(Course::load()),
        evaluator: Arc::new(IsolatedRunner {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(35))
                .build()?,
            url: std::env::var("RUNNER_URL")?,
            secret: std::env::var("RUNNER_SECRET")?,
        }),
        cookie_secure: std::env::var("COOKIE_SECURE").as_deref() == Ok("true"),
        origin: std::env::var("APP_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".into()),
        slots: Arc::new(Semaphore::new(2)),
        rate: Arc::new(Mutex::new(HashMap::new())),
    };
    let app = router(state);
    let address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".into());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!(%address, "RustPath API listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}
fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/session", post(session))
        .route("/api/course", get(course))
        .route("/api/progress", get(progress))
        .route("/api/bookmarks", get(bookmarks))
        .route(
            "/api/lessons/{id}/bookmark",
            put(add_bookmark).delete(remove_bookmark),
        )
        .route("/api/lessons/{id}/answer", post(answer))
        .route("/api/lessons/{id}", get(lesson))
        .route("/api/lessons/{id}/draft", get(draft).put(save_draft))
        .route("/api/lessons/{id}/submit", post(submit))
        .route("/api/session", put(rotate_session))
        .layer(DefaultBodyLimit::max(24 * 1024))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}
async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}
async fn health(State(s): State<AppState>) -> Result<Json<serde_json::Value>, AppError> {
    sqlx::query("SELECT 1").execute(&s.db).await?;
    Ok(Json(json!({"status":"ok"})))
}
fn hash_token(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}
fn cookie(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|c| c.trim().strip_prefix("rustpath_session="))
}
async fn learner(s: &AppState, headers: &HeaderMap) -> Result<Uuid, AppError> {
    let token = cookie(headers)
        .filter(|t| t.len() == 64)
        .ok_or(AppError::Unauthorized)?;
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM learners WHERE token_hash=$1")
        .bind(hash_token(token))
        .fetch_optional(&s.db)
        .await?
        .ok_or(AppError::Unauthorized)
}
fn origin_matches(origin: &str, headers: &HeaderMap) -> bool {
    headers
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        == Some(origin)
}
fn check_origin(s: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    if !origin_matches(&s.origin, headers) {
        return Err(AppError::BadRequest("Недопустимый Origin запроса.".into()));
    }
    Ok(())
}
fn session_cookie(token: &str, secure: bool) -> HeaderValue {
    HeaderValue::from_str(&format!(
        "rustpath_session={token}; HttpOnly; SameSite=Strict; Path=/api; Max-Age=31536000{}",
        if secure { "; Secure" } else { "" }
    ))
    .expect("safe cookie")
}
async fn session(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    check_origin(&s, &headers)?;
    if let Ok(id) = learner(&s, &headers).await {
        return Ok((HeaderMap::new(), Json(json!({"learner_id": id}))));
    }
    let id = Uuid::new_v4();
    let token = format!(
        "{:032x}{:032x}",
        rand::random::<u128>(),
        rand::random::<u128>()
    );
    sqlx::query("INSERT INTO learners(id,token_hash) VALUES($1,$2)")
        .bind(id)
        .bind(hash_token(&token))
        .execute(&s.db)
        .await?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::SET_COOKIE, session_cookie(&token, s.cookie_secure));
    Ok((response_headers, Json(json!({"learner_id": id}))))
}
async fn rotate_session(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    check_origin(&s, &headers)?;
    let id = learner(&s, &headers).await?;
    let token = format!(
        "{:032x}{:032x}",
        rand::random::<u128>(),
        rand::random::<u128>()
    );
    sqlx::query("UPDATE learners SET token_hash=$1 WHERE id=$2")
        .bind(hash_token(&token))
        .bind(id)
        .execute(&s.db)
        .await?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::SET_COOKIE, session_cookie(&token, s.cookie_secure));
    Ok((response_headers, StatusCode::NO_CONTENT))
}
async fn course(State(s): State<AppState>) -> Json<serde_json::Value> {
    Json(
        json!({"version":s.course.version, "tracks":s.course.tracks, "modules": s.course.modules, "lessons": s.course.lessons.iter().map(LessonSummary::from).collect::<Vec<_>>(), "roadmap": s.course.roadmap}),
    )
}
async fn progress(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Progress>, AppError> {
    let id = learner(&s, &headers).await?;
    Ok(Json(
        s.repository.progress(id, s.course.lessons.len()).await?,
    ))
}
async fn accessible_lesson<'a>(
    s: &'a AppState,
    headers: &HeaderMap,
    id: &str,
) -> Result<(Uuid, &'a Lesson), AppError> {
    let l = s.course.lesson(id).ok_or(AppError::NotFound)?;
    let learner = learner(s, headers).await?;
    let p = s
        .repository
        .progress(learner, s.course.lessons.len())
        .await?;
    if !s.course.unlocked(id, &p.completed) {
        return Err(AppError::Locked);
    }
    Ok((learner, l))
}
async fn lesson(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Lesson>, AppError> {
    let (_, l) = accessible_lesson(&s, &headers, &id).await?;
    Ok(Json(l.clone()))
}
async fn draft(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (learner, _) = accessible_lesson(&s, &headers, &id).await?;
    let code: Option<String> =
        sqlx::query_scalar("SELECT code FROM drafts WHERE learner_id=$1 AND lesson_id=$2")
            .bind(learner)
            .bind(id)
            .fetch_optional(&s.db)
            .await?;
    Ok(Json(json!({"code":code})))
}
#[derive(Deserialize)]
struct CodeInput {
    code: String,
}
fn validate_code(code: &str) -> Result<(), AppError> {
    if code.is_empty() || code.len() > 16 * 1024 {
        return Err(AppError::BadRequest(
            "Код должен занимать от 1 до 16384 байт.".into(),
        ));
    }
    // Do not append tests to source with attributes able to disable/replace the harness.
    // This is only an integrity guard; isolation is provided by the runner, never by this filter.
    if code.contains("#!")
        || code.contains("#[")
        || code.contains("include!")
        || code.contains("include_str!")
        || code.contains("include_bytes!")
    {
        return Err(AppError::BadRequest(
            "В учебных заданиях атрибуты и include-макросы недоступны.".into(),
        ));
    }
    Ok(())
}
async fn save_draft(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<CodeInput>,
) -> Result<StatusCode, AppError> {
    check_origin(&s, &headers)?;
    let (learner, _) = accessible_lesson(&s, &headers, &id).await?;
    if input.code.len() > 16 * 1024 {
        return Err(AppError::BadRequest("Черновик слишком большой.".into()));
    }
    sqlx::query("INSERT INTO drafts(learner_id,lesson_id,code) VALUES($1,$2,$3) ON CONFLICT(learner_id,lesson_id) DO UPDATE SET code=excluded.code, updated_at=now()")
        .bind(learner).bind(id).bind(input.code).execute(&s.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn submit(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<CodeInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_origin(&s, &headers)?;
    validate_code(&input.code)?;
    let (learner, l) = accessible_lesson(&s, &headers, &id).await?;
    if l.kind != "code" {
        return Err(AppError::BadRequest(
            "Для этого урока нужен ответ на вопрос, а не код.".into(),
        ));
    }
    limit_attempt(&s, learner).await?;
    let _permit = s
        .slots
        .clone()
        .try_acquire_owned()
        .map_err(|_| AppError::Runner)?;
    let result = s.evaluator.evaluate(&input.code, &l.tests).await?;
    let awarded = s
        .repository
        .complete(learner, &id, l.xp, result.passed)
        .await?;
    let progress = s
        .repository
        .progress(learner, s.course.lessons.len())
        .await?;
    Ok(Json(
        json!({"evaluation":result,"awarded_xp":awarded,"progress":progress}),
    ))
}
async fn limit_attempt(s: &AppState, learner: Uuid) -> Result<(), AppError> {
    let mut rates = s.rate.lock().await;
    rates.retain(|_, (time, _)| time.elapsed() < Duration::from_secs(60));
    let entry = rates.entry(learner).or_insert((Instant::now(), 0));
    if entry.1 >= 10 {
        return Err(AppError::RateLimited);
    }
    entry.1 += 1;
    Ok(())
}
#[derive(Deserialize)]
struct AnswerInput {
    option: usize,
}
async fn answer(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(input): Json<AnswerInput>,
) -> Result<Json<serde_json::Value>, AppError> {
    check_origin(&s, &headers)?;
    let (learner, l) = accessible_lesson(&s, &headers, &id).await?;
    let quiz = l
        .quiz
        .as_ref()
        .filter(|_| l.kind == "quiz")
        .ok_or_else(|| AppError::BadRequest("Этот урок проверяется кодом.".into()))?;
    if input.option >= quiz.options.len() {
        return Err(AppError::BadRequest("Недопустимый вариант ответа.".into()));
    }
    limit_attempt(&s, learner).await?;
    let passed = input.option == quiz.correct;
    let awarded = s.repository.complete(learner, &id, l.xp, passed).await?;
    let progress = s
        .repository
        .progress(learner, s.course.lessons.len())
        .await?;
    Ok(Json(
        json!({"evaluation":{"passed":passed,"output":quiz.explanation,"duration_ms":0},"awarded_xp":awarded,"progress":progress}),
    ))
}
async fn bookmarks(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<String>>, AppError> {
    let user = learner(&s, &headers).await?;
    Ok(Json(
        sqlx::query_scalar(
            "SELECT lesson_id FROM bookmarks WHERE learner_id=$1 ORDER BY created_at",
        )
        .bind(user)
        .fetch_all(&s.db)
        .await?,
    ))
}
async fn bookmark_user(s: &AppState, headers: &HeaderMap, id: &str) -> Result<Uuid, AppError> {
    check_origin(s, headers)?;
    s.course.lesson(id).ok_or(AppError::NotFound)?;
    learner(s, headers).await
}
async fn add_bookmark(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let user = bookmark_user(&s, &headers, &id).await?;
    sqlx::query("INSERT INTO bookmarks(learner_id,lesson_id) VALUES($1,$2) ON CONFLICT DO NOTHING")
        .bind(user)
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn remove_bookmark(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    let user = bookmark_user(&s, &headers, &id).await?;
    sqlx::query("DELETE FROM bookmarks WHERE learner_id=$1 AND lesson_id=$2")
        .bind(user)
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn csrf_origin_is_exact() {
        let mut headers = HeaderMap::new();
        assert!(!origin_matches("http://localhost:8080", &headers));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("http://localhost:8080.evil.test"),
        );
        assert!(!origin_matches("http://localhost:8080", &headers));
        headers.insert(
            header::ORIGIN,
            HeaderValue::from_static("http://localhost:8080"),
        );
        assert!(origin_matches("http://localhost:8080", &headers));
    }
    #[test]
    fn code_size_and_harness_attributes_are_guarded() {
        assert!(validate_code("").is_err());
        assert!(validate_code(&"x".repeat(16385)).is_err());
        assert!(validate_code("#![cfg(any())]").is_err());
        assert!(validate_code("pub fn answer() -> i32 { 42 }").is_ok());
    }
    #[test]
    fn session_cookie_is_private() {
        let c = session_cookie(&"a".repeat(64), true);
        assert!(c.to_str().unwrap().contains("HttpOnly; SameSite=Strict"));
        assert!(c.to_str().unwrap().ends_with("; Secure"));
    }
}

#[cfg(test)]
mod integration {
    use super::*;
    use async_trait::async_trait;
    use axum::{body::Body, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    struct FakeEvaluator;
    #[async_trait]
    impl CodeEvaluator for FakeEvaluator {
        async fn evaluate(&self, code: &str, tests: &str) -> Result<domain::Evaluation, AppError> {
            assert!(tests.contains("assert_eq!"));
            Ok(domain::Evaluation {
                passed: code.contains("correct"),
                output: "injected test evaluator; no user code execution".into(),
                duration_ms: 1,
            })
        }
    }
    async fn request(
        app: &Router,
        method: &str,
        path: &str,
        cookie: Option<&str>,
        body: Option<&str>,
        origin: bool,
    ) -> (StatusCode, HeaderMap, serde_json::Value) {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header("Content-Type", "application/json");
        if origin {
            builder = builder.header("Origin", "http://localhost:8080");
        }
        if let Some(cookie) = cookie {
            builder = builder.header("Cookie", cookie);
        }
        let response = app
            .clone()
            .oneshot(
                builder
                    .body(Body::from(body.unwrap_or("").to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let value = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
        (status, headers, value)
    }
    #[tokio::test]
    #[ignore = "requires dedicated INTEGRATION_DATABASE_URL"]
    async fn postgres_http_flow_and_isolation() {
        let db = PgPoolOptions::new()
            .max_connections(5)
            .connect(&std::env::var("INTEGRATION_DATABASE_URL").expect("dedicated test DB URL"))
            .await
            .unwrap();
        sqlx::migrate!().run(&db).await.unwrap();
        let app = router(AppState {
            repository: Arc::new(PgLearningRepository(db.clone())),
            db: db.clone(),
            course: Arc::new(Course::load()),
            evaluator: Arc::new(FakeEvaluator),
            cookie_secure: false,
            origin: "http://localhost:8080".into(),
            slots: Arc::new(Semaphore::new(2)),
            rate: Arc::new(Mutex::new(HashMap::new())),
        });
        assert_eq!(
            request(&app, "POST", "/api/session", None, None, false)
                .await
                .0,
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            request(&app, "GET", "/api/progress", None, None, false)
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
        let (status, headers, guest) =
            request(&app, "POST", "/api/session", None, None, true).await;
        assert_eq!(status, StatusCode::OK);
        let cookie = headers[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        let (_, headers2, guest2) = request(&app, "POST", "/api/session", None, None, true).await;
        let cookie2 = headers2[header::SET_COOKIE]
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();
        assert_eq!(
            request(
                &app,
                "GET",
                "/api/lessons/arithmetic",
                Some(&cookie),
                None,
                false
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
        let (status, _, l) = request(
            &app,
            "GET",
            "/api/lessons/hello",
            Some(&cookie),
            None,
            false,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(l.get("tests").is_none());
        assert!(l.get("solution").is_none());
        assert_eq!(
            request(
                &app,
                "PUT",
                "/api/lessons/hello/draft",
                Some(&cookie),
                Some(r#"{"code":"draft"}"#),
                true
            )
            .await
            .0,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            request(
                &app,
                "GET",
                "/api/lessons/hello/draft",
                Some(&cookie),
                None,
                false
            )
            .await
            .2["code"],
            "draft"
        );
        assert!(
            request(
                &app,
                "GET",
                "/api/lessons/hello/draft",
                Some(&cookie2),
                None,
                false
            )
            .await
            .2["code"]
                .is_null()
        );
        let wrong = request(
            &app,
            "POST",
            "/api/lessons/hello/submit",
            Some(&cookie),
            Some(r#"{"code":"wrong"}"#),
            true,
        )
        .await;
        assert_eq!(wrong.0, StatusCode::OK);
        assert_eq!(wrong.2["awarded_xp"], 0);
        assert_eq!(wrong.2["progress"]["completed"], json!([]));
        let good = request(
            &app,
            "POST",
            "/api/lessons/hello/submit",
            Some(&cookie),
            Some(r#"{"code":"correct"}"#),
            true,
        )
        .await;
        assert_eq!(good.2["awarded_xp"], 50);
        assert_eq!(good.2["progress"]["completed"], json!(["hello"]));
        let repeated = request(
            &app,
            "POST",
            "/api/lessons/hello/submit",
            Some(&cookie),
            Some(r#"{"code":"correct"}"#),
            true,
        )
        .await;
        assert_eq!(repeated.2["awarded_xp"], 0);
        assert_eq!(repeated.2["progress"]["xp"], 50);
        assert_eq!(
            request(
                &app,
                "GET",
                "/api/lessons/arithmetic",
                Some(&cookie),
                None,
                false
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(
            request(
                &app,
                "GET",
                "/api/lessons/arithmetic",
                Some(&cookie2),
                None,
                false
            )
            .await
            .0,
            StatusCode::FORBIDDEN
        );
        let q = request(
            &app,
            "GET",
            "/api/lessons/intro-rust",
            Some(&cookie),
            None,
            false,
        )
        .await;
        assert!(q.2["quiz"].get("correct").is_none());
        assert_eq!(
            request(
                &app,
                "POST",
                "/api/lessons/intro-rust/answer",
                Some(&cookie),
                Some(r#"{"option":99}"#),
                true
            )
            .await
            .0,
            StatusCode::BAD_REQUEST
        );
        let wrong_quiz = request(
            &app,
            "POST",
            "/api/lessons/intro-rust/answer",
            Some(&cookie),
            Some(r#"{"option":1}"#),
            true,
        )
        .await;
        assert_eq!(wrong_quiz.2["awarded_xp"], 0);
        let right_quiz = request(
            &app,
            "POST",
            "/api/lessons/intro-rust/answer",
            Some(&cookie),
            Some(r#"{"option":0}"#),
            true,
        )
        .await;
        assert_eq!(right_quiz.2["awarded_xp"], 20);
        assert_eq!(right_quiz.2["progress"]["xp"], 70);
        assert_eq!(
            request(
                &app,
                "POST",
                "/api/lessons/intro-rust/answer",
                Some(&cookie),
                Some(r#"{"option":0}"#),
                true
            )
            .await
            .2["awarded_xp"],
            0
        );
        assert_eq!(
            request(
                &app,
                "GET",
                "/api/lessons/intro-main",
                Some(&cookie),
                None,
                false
            )
            .await
            .0,
            StatusCode::OK
        );
        assert_eq!(
            request(
                &app,
                "PUT",
                "/api/lessons/adv-min/bookmark",
                Some(&cookie),
                None,
                true
            )
            .await
            .0,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            request(&app, "GET", "/api/bookmarks", Some(&cookie), None, false)
                .await
                .2,
            json!(["adv-min"])
        );
        assert_eq!(
            request(&app, "GET", "/api/bookmarks", Some(&cookie2), None, false)
                .await
                .2,
            json!([])
        );
        assert_eq!(
            request(
                &app,
                "DELETE",
                "/api/lessons/adv-min/bookmark",
                Some(&cookie),
                None,
                true
            )
            .await
            .0,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            request(&app, "GET", "/api/bookmarks", Some(&cookie), None, false)
                .await
                .2,
            json!([])
        );
        let guest_id = Uuid::parse_str(guest["learner_id"].as_str().unwrap()).unwrap();
        let guest2_id = Uuid::parse_str(guest2["learner_id"].as_str().unwrap()).unwrap();
        sqlx::query("DELETE FROM learners WHERE id=$1 OR id=$2")
            .bind(guest_id)
            .bind(guest2_id)
            .execute(&db)
            .await
            .unwrap();
    }
}
