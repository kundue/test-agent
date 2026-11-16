use axum::{
    Router,
    body::Body,
    extract::Request,
    http::{HeaderValue, StatusCode, header::CONTENT_TYPE},
    response::Response,
    routing::get,
};

pub fn app() -> Router {
    Router::new().route("/ping", get(ping))
}

async fn ping(request: Request<Body>) -> Response<Body> {
    let accept = request
        .headers()
        .get("accept")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let accept = accept.trim();

    if accept.is_empty() || accepts(accept, "application/json") {
        return response(StatusCode::OK, "application/json", r#"{"ping":"pong"}"#);
    }

    if accepts(accept, "application/xml") || accepts(accept, "text/xml") {
        return response(StatusCode::OK, "application/xml", "<ping>pong</ping>");
    }

    Response::builder()
        .status(StatusCode::NOT_ACCEPTABLE)
        .body(Body::empty())
        .expect("a valid 406 response")
}

fn accepts(header: &str, media_type: &str) -> bool {
    header.split(',').any(|entry| {
        entry
            .split(';')
            .next()
            .is_some_and(|value| value.trim().eq_ignore_ascii_case(media_type))
    })
}

fn response(status: StatusCode, content_type: &'static str, body: &'static str) -> Response<Body> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, HeaderValue::from_static(content_type))
        .body(Body::from(body))
        .expect("a valid ping response")
}

#[cfg(test)]
mod tests {
    use super::app;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::util::ServiceExt;

    async fn get_ping(accept: Option<&str>) -> axum::response::Response {
        let mut request = Request::builder().uri("/ping");
        if let Some(value) = accept {
            request = request.header("accept", value);
        }
        app().oneshot(request.body(axum::body::Body::empty()).unwrap())
            .await
            .unwrap()
    }

    async fn body(response: axum::response::Response) -> String {
        String::from_utf8(response.into_body().collect().await.unwrap().to_bytes().to_vec())
            .unwrap()
    }

    #[tokio::test]
    async fn ac_01_get_ping_returns_success() {
        let response = get_ping(Some("application/json")).await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn ac_02_json_accept_returns_json_body_and_content_type() {
        let response = get_ping(Some("application/json")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(body(response).await, r#"{"ping":"pong"}"#);
    }

    #[tokio::test]
    async fn ac_03_missing_accept_defaults_to_json() {
        let response = get_ping(None).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(body(response).await, r#"{"ping":"pong"}"#);
    }

    #[tokio::test]
    async fn ac_04_empty_accept_defaults_to_json() {
        let response = get_ping(Some("")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(body(response).await, r#"{"ping":"pong"}"#);
    }

    #[tokio::test]
    async fn ac_05_application_xml_accept_returns_xml() {
        let response = get_ping(Some("application/xml")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "application/xml");
        assert_eq!(body(response).await, "<ping>pong</ping>");
    }

    #[tokio::test]
    async fn ac_06_text_xml_accept_returns_xml() {
        let response = get_ping(Some("text/xml")).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()["content-type"], "application/xml");
        assert_eq!(body(response).await, "<ping>pong</ping>");
    }

    #[tokio::test]
    async fn ac_07_unsupported_accept_returns_not_acceptable() {
        let response = get_ping(Some("image/png")).await;
        assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
    }
}
