use axum::{
    Router, middleware,
    routing::{delete, get, post},
};

use crate::{handlers::users::sessions::*, middleware::auth_middleware, state::AppState};

pub fn routes(state: AppState) -> Router {
    let public_routes = Router::new().route("/refresh", post(refresh_token));

    let protected_routes = Router::new()
        .route("/", get(get_sessions))
        .route("/other", delete(delete_all_other_sessions))
        .route("/{session_id}", delete(delete_session_by_id))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    public_routes.merge(protected_routes).with_state(state)
}
