pub mod sites;
pub mod areas;
pub mod isolates;
pub mod samples;
pub mod dna;

use axum::Router;
use sea_orm::DatabaseConnection;

/// Public API router - serves only non-private records, no authentication required
pub fn router(db: DatabaseConnection) -> Router {
    Router::new()
        .nest("/sites", sites::router(db.clone()))
        .nest("/areas", areas::router(db.clone()))
        .nest("/isolates", isolates::router(db.clone()))
        .nest("/samples", samples::router(db.clone()))
        .nest("/dna", dna::router(db.clone()))
}