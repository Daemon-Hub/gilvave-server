use axum::{
    Router, middleware,
    routing::{delete, get, post},
};

use crate::{
    handlers::files::{delete_file, download_file, get_file_info, upload_file},
    middleware::auth_middleware,
    state::AppState,
};

/// Маршруты файлов
pub fn routes(state: AppState) -> Router {
    let public_routes = Router::new()
        .route("/{file_id}", get(download_file))
        .route("/{file_id}/download", get(download_file))
        .route("/{file_id}/info", get(get_file_info));

    let protected_routes = Router::new()
        .route("/", post(upload_file))
        .route("/{file_id}", delete(delete_file))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state)
}
