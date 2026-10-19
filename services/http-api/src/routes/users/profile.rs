use axum::{
    Router, middleware,
    routing::{get, patch},
};

use crate::{
    handlers::users::profile::*,
    middleware::auth_middleware,
    state::AppState,
};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/me", get(get_profile))
        .route("/me/avatar", patch(update_avatar))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}
