use axum::{
    Router, middleware,
    routing::{delete, get, post},
};

use crate::{handlers::users::friends::*, middleware::auth_middleware, state::AppState};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/", get(get_friends))
        .route("/{friend_id}", delete(remove_friend))
        .route("/requests", get(get_requests).post(send_request))
        .route("/requests/{user_id}", delete(decline_request))
        .route("/requests/{user_id}/accept", post(accept_request))
        .route("/requests/{user_id}/decline", post(decline_request))
        .route("/blocks", get(get_blocked_users).post(block_user))
        .route(
            "/blocks/{user_id}",
            post(block_user_by_id).delete(unblock_user),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}
