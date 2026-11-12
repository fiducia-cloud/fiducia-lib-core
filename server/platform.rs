//! Shared request-serving policy for every Fiducia Rust web/API binary.
//!
//! The portable request lifecycle comes from `ores-middleware`. Its built-in
//! limiter is installed in audit mode because the authoritative admission state
//! below is implemented by `ores-rate-limit/ores-rl-lib-core`.

use std::any::type_name;

use axum::{
    extract::{Request, State},
    http::{header::RETRY_AFTER, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use ores_middleware::{default_config, descriptor, RuntimeEnvironment};
use ores_rl_lib_core::{ConsistencyMode, Decision, LimitPolicy};

use crate::config::ServiceRole;

#[path = "rate_limiter.rs"]
mod rate_limiter;
pub use rate_limiter::{PlatformError, SharedRateLimiter};

/// Immutable dependency and policy evidence shared by all four standard
/// Fiducia Rust servers.
#[derive(Debug, Clone)]
pub struct ServiceProfile {
    pub role: ServiceRole,
    pub service_name: String,
    pub interface_type: &'static str,
    pub middleware_contract_version: &'static str,
    pub middleware_capabilities: Vec<String>,
    pub rate_limit_policy: LimitPolicy,
}

impl ServiceProfile {
    pub fn new(service_name: impl Into<String>, role: ServiceRole) -> Result<Self, PlatformError> {
        let service_name = service_name.into();
        let middleware = descriptor();
        let rate_limit_policy = policy_for(role)
            .validate()
            .map_err(|error| PlatformError::RateLimit(error.to_string()))?;
        Ok(Self {
            role,
            service_name,
            interface_type: type_name::<fiducia_interfaces::SupportRequirementsSupportPlan>(),
            middleware_contract_version: ores_middleware::CONTRACT_VERSION,
            middleware_capabilities: middleware.capabilities,
            rate_limit_policy,
        })
    }
}

/// Install the standard ORES request lifecycle plus the ORES rate-limit state
/// machine. Health/version probes remain outside admission quotas.
pub fn install_axum(
    router: Router,
    service_name: impl Into<String>,
    role: ServiceRole,
    environment: &str,
) -> Result<Router, PlatformError> {
    let service_name = service_name.into();
    let profile = ServiceProfile::new(service_name.clone(), role)?;
    let mut config = default_config(service_name);
    config.environment = runtime_environment(environment);
    let router = ores_middleware::frameworks::axum_audit::install_with_config(router, config)
        .map_err(|error| PlatformError::Middleware(error.to_string()))?;
    Ok(router.layer(middleware::from_fn_with_state(
        SharedRateLimiter::new(profile.rate_limit_policy),
        enforce_rate_limit,
    )))
}

fn runtime_environment(value: &str) -> RuntimeEnvironment {
    match value.trim().to_ascii_lowercase().as_str() {
        "test" => RuntimeEnvironment::Test,
        "stage" | "staging" => RuntimeEnvironment::Staging,
        "prod" | "production" => RuntimeEnvironment::Production,
        _ => RuntimeEnvironment::Development,
    }
}

const fn policy_for(role: ServiceRole) -> LimitPolicy {
    let capacity = match role {
        ServiceRole::Web => 240,
        ServiceRole::Api => 120,
        ServiceRole::AdminWeb => 60,
        ServiceRole::AdminApi => 60,
        ServiceRole::Worker => 600,
    };
    LimitPolicy::fixed_window(capacity, 60_000).with_consistency(ConsistencyMode::Bounded)
}

async fn enforce_rate_limit(
    State(limiter): State<SharedRateLimiter>,
    request: Request,
    next: Next,
) -> Response {
    if matches!(request.uri().path(), "/healthz" | "/readyz" | "/version") {
        return next.run(request).await;
    }

    match limiter.decide(1) {
        Ok(Decision::Allow { .. } | Decision::Bypass { .. }) => next.run(request).await,
        Ok(Decision::Deny { retry_after_ms, .. }) => {
            let mut response = StatusCode::TOO_MANY_REQUESTS.into_response();
            let seconds = retry_after_ms.div_ceil(1_000).max(1);
            if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
                response.headers_mut().insert(RETRY_AFTER, value);
            }
            response
        }
        Err(error) => {
            tracing::warn!(error = %error, "shared rate limiter failed closed");
            StatusCode::SERVICE_UNAVAILABLE.into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_bind_all_three_external_contracts() {
        let profile = ServiceProfile::new("fiducia-api-server", ServiceRole::Api).unwrap();
        assert!(profile.interface_type.contains("fiducia_interfaces"));
        assert!(!profile.middleware_contract_version.is_empty());
        assert!(profile
            .middleware_capabilities
            .iter()
            .any(|value| value == "request-context"));
        assert_eq!(profile.rate_limit_policy.capacity, 120);
    }

    #[test]
    fn shared_limiter_enforces_ores_rate_limit_transition() {
        let limiter = SharedRateLimiter::new(LimitPolicy::fixed_window(2, 1_000));
        assert!(matches!(
            limiter.decide_at(0, 1).unwrap(),
            Decision::Allow { .. }
        ));
        assert!(matches!(
            limiter.decide_at(1, 1).unwrap(),
            Decision::Allow { .. }
        ));
        assert!(matches!(
            limiter.decide_at(2, 1).unwrap(),
            Decision::Deny { .. }
        ));
    }
    #[test]
    fn real_concurrent_admission_has_one_linearized_winner() {
        use std::sync::{Arc, Barrier};
        use std::thread;

        // All requests consume one unit in the same fixed window. The
        // production decide_at method must keep read/decide/commit under one
        // mutex; it is not enough for the abstract checker alone to pass.
        const CALLERS: usize = 32;
        let limiter = Arc::new(SharedRateLimiter::new(LimitPolicy::fixed_window(1, 60_000)));
        let barrier = Arc::new(Barrier::new(CALLERS + 1));
        let mut threads = Vec::new();
        for _ in 0..CALLERS {
            let limiter = limiter.clone();
            let barrier = barrier.clone();
            threads.push(thread::spawn(move || {
                barrier.wait();
                limiter
                    .decide_at(1, 1)
                    .expect("valid rate-limit transition")
            }));
        }
        barrier.wait();
        let mut allowed = 0;
        let mut denied = 0;
        for handle in threads {
            match handle.join().expect("admission thread") {
                Decision::Allow { .. } => allowed += 1,
                Decision::Deny { .. } => denied += 1,
                Decision::Bypass { .. } => panic!("fixed-window policy bypassed quota"),
            }
        }
        assert_eq!(allowed, 1, "capacity-one window overadmitted");
        assert_eq!(denied, CALLERS - 1);
        assert!(matches!(
            limiter.decide_at(2, 1).unwrap(),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn cloned_limiter_handles_share_the_same_quota_authority() {
        let limiter = SharedRateLimiter::new(LimitPolicy::fixed_window(1, 60_000));
        let clone = limiter.clone();
        assert!(matches!(
            clone.decide_at(0, 1).unwrap(),
            Decision::Allow { .. }
        ));
        assert!(matches!(
            limiter.decide_at(1, 1).unwrap(),
            Decision::Deny { .. }
        ));
    }
}
