pub mod channels;

use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::{
    handlers::servers::*,
    middleware::auth_middleware,
    state::AppState,
};

pub fn routes(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/public/{page}", get(get_public_servers))
        .route("/{server_id}/members", get(get_members));

    let protected_routes = Router::new()
        .route("/", get(get_user_servers).post(create_server))
        .route("/{server_id}", get(get_server_by_id))
        .route("/{server_id}/join_public", post(join_public))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    public_routes
        .merge(protected_routes)
        .with_state(state.clone())
        // Подгруппа для каналов
        .nest("/{server_id}/channels", channels::routes(state))
}
