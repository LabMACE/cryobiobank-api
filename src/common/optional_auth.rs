use axum::http::header::AUTHORIZATION;

#[derive(Debug, Clone)]
pub struct OptionalAuth {
    pub is_authenticated: bool,
    pub has_admin_role: bool,
}

impl OptionalAuth {
    pub fn new(is_authenticated: bool, has_admin_role: bool) -> Self {
        Self {
            is_authenticated,
            has_admin_role,
        }
    }
    
    pub fn unauthenticated() -> Self {
        Self {
            is_authenticated: false,
            has_admin_role: false,
        }
    }
    
    pub fn admin() -> Self {
        Self {
            is_authenticated: true,
            has_admin_role: true,
        }
    }
}

impl OptionalAuth {
    /// Simple header-based authentication check
    /// Returns authenticated admin if Bearer token is present, unauthenticated otherwise
    /// For production use, the Keycloak middleware should validate tokens before they reach this code
    pub fn from_headers(headers: &axum::http::HeaderMap) -> Self {
        let auth_header = match headers.get(AUTHORIZATION) {
            Some(header) => header,
            None => return OptionalAuth::unauthenticated(),
        };

        let auth_str = match auth_header.to_str() {
            Ok(str) => str,
            Err(_) => return OptionalAuth::unauthenticated(),
        };

        if auth_str.starts_with("Bearer ") {
            // If we have a Bearer token, assume it's been validated by Keycloak middleware
            // and the user has admin permissions
            OptionalAuth::admin()
        } else {
            OptionalAuth::unauthenticated()
        }
    }
}