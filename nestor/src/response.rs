use failure::Error;

#[derive(Debug)]
pub enum Response {
    Say(String),
    Act(String),
    Notice(String),
    None,
}

pub enum Outcome {
    Success(Response),
    Failure(Error),
    Forward(String),
}

pub trait IntoResponse {
    fn into_response(self) -> Response;
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::Notice(self.to_string())
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::Notice(self)
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::None
    }
}

impl<T> IntoResponse for Option<T>
where
    T: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Some(inner) => inner.into_response(),
            None => Response::None,
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

pub trait IntoOutcome {
    fn into_outcome(self) -> Outcome;
}

impl<T> IntoOutcome for T
where
    T: IntoResponse,
{
    fn into_outcome(self) -> Outcome {
        let response = self.into_response();
        Outcome::Success(response)
    }
}

impl IntoOutcome for Outcome {
    fn into_outcome(self) -> Outcome {
        self
    }
}

impl<T, E> IntoOutcome for Result<T, E>
where
    T: IntoResponse,
    E: Into<Error>,
{
    fn into_outcome(self) -> Outcome {
        match self {
            Ok(inner) => Outcome::Success(inner.into_response()),
            Err(err) => Outcome::Failure(err.into()),
        }
    }
}
