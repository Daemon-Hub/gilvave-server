use axum::{
    Router, middleware,
    routing::{get, patch},
};

use crate::{
    handlers::servers::channels::*,
    middleware::auth_middleware,
    state::AppState,
};

pub fn routes(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/", get(get_all).post(create));

    let protected_routes = Router::new()
        .route("/{channel_id}/name", patch(update_name))
        .route("/{channel_id}/position", patch(update_position))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    public_routes.merge(protected_routes).with_state(state)
}
