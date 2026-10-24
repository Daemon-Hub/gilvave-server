use axum::{
    Router, middleware,
    routing::{get, post},
};

use crate::{handlers::users::auth::*, middleware::auth_middleware, state::AppState};

pub fn routes(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/register/validate", post(register_validate))
        .route("/register/confirm", get(register_confirm))
        .route("/login", post(login));

    let protected_routes =
        Router::new()
            .route("/logout", post(logout))
            .route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

    public_routes.merge(protected_routes).with_state(state)
}
