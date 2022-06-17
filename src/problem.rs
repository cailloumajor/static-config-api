use serde::{Serialize, Serializer};
use trillium::{conn_try, Conn, KnownHeaderName, Status};

struct StatusCode(Status);

impl Serialize for StatusCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value: u16 = self.0 as u16;
        serializer.serialize_u16(value)
    }
}

/// Represents problem details as of [RFC7807](https://datatracker.ietf.org/doc/html/rfc7807).
#[derive(Serialize)]
pub struct ProblemDetails {
    #[serde(rename = "type")]
    type_uri: String,
    title: String,
    status: StatusCode,
    detail: String,
}

impl ProblemDetails {
    pub fn new(problem_type: &str, title: &str, status: Status, detail: &str) -> Self {
        let mut type_uri = String::from("/problem/");
        type_uri.push_str(problem_type);
        Self {
            type_uri,
            title: title.into(),
            status: StatusCode(status),
            detail: detail.into(),
        }
    }
}

/// Extension trait that adds methods to [`trillium::Conn`].
pub trait ProblemDetailsConnExt {
    fn with_problem_details(self, details: &ProblemDetails) -> Self;
}

impl ProblemDetailsConnExt for Conn {
    fn with_problem_details(self, details: &ProblemDetails) -> Self {
        let body = conn_try!(serde_json::to_string(details), self);
        self.with_status(details.status.0)
            .with_header(KnownHeaderName::ContentType, "application/problem+json")
            .with_body(body)
            .halt()
    }
}

#[cfg(test)]
mod tests {
    use trillium_testing::prelude::*;

    use super::*;

    async fn handler(conn: Conn) -> Conn {
        conn.with_problem_details(&ProblemDetails::new(
            "test-problem",
            "A test problem",
            Status::ImATeapot,
            "Test problem details",
        ))
    }

    #[test]
    fn with_problem_details() {
        assert_response!(
            get("/").on(&handler),
            Status::ImATeapot,
            r#"{"type":"/problem/test-problem","title":"A test problem","status":418,"detail":"Test problem details"}"#,
            "content-type" => "application/problem+json"
        );
    }
}