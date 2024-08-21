use axum::{extract::{Path, State}, response::IntoResponse, http::StatusCode};

#[tracing::instrument(skip(state))]
pub async fn get_user_id(
  State(state): State<super::State>,
  Path(name): Path<String>,
) -> impl IntoResponse {
  match state.get_user_id(name.as_str()) {
    Err(err) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      err.to_string()
    ).into_response(),
    Ok(user_id) => (
      StatusCode::OK,
      [("X-Anilist-User-Name", name), ("X-Anilist-User-Id", user_id.clone())],
      user_id
    ).into_response()
  }
}

#[tracing::instrument(skip(state))]
pub async fn get_user_media(
  State(state): State<super::State>,
  Path(name): Path<String>,
) -> impl IntoResponse {
  match state.generate(name.as_str()) {
    Err(err) => (
      StatusCode::BAD_GATEWAY,
      err.to_string()
    ).into_response(),
    Ok(custom_list) => axum::response::Json(custom_list).into_response()
  }
}
