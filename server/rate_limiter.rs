use ores_rl_lib_core::{transition, Decision, LimitPolicy, LimitState, TransitionError};
use std::{
    fmt,
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    Middleware(String),
    RateLimit(String),
    PoisonedRateLimitState,
}

impl fmt::Display for PlatformError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Middleware(message) => write!(formatter, "middleware policy error: {message}"),
            Self::RateLimit(message) => write!(formatter, "rate-limit policy error: {message}"),
            Self::PoisonedRateLimitState => {
                formatter.write_str("rate-limit state lock is poisoned")
            }
        }
    }
}

impl std::error::Error for PlatformError {}

impl From<TransitionError> for PlatformError {
    fn from(value: TransitionError) -> Self {
        Self::RateLimit(value.to_string())
    }
}

#[derive(Clone)]
pub struct SharedRateLimiter {
    policy: LimitPolicy,
    state: Arc<Mutex<LimitState>>,
    started_at: Instant,
}

impl fmt::Debug for SharedRateLimiter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SharedRateLimiter")
            .field("policy", &self.policy)
            .finish_non_exhaustive()
    }
}

impl SharedRateLimiter {
    #[must_use]
    pub fn new(policy: LimitPolicy) -> Self {
        Self {
            policy,
            state: Arc::new(Mutex::new(LimitState::Empty)),
            started_at: Instant::now(),
        }
    }

    pub fn decide_at(&self, now_ms: u64, cost: u64) -> Result<Decision, PlatformError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| PlatformError::PoisonedRateLimitState)?;
        let (next, decision) = transition(self.policy, *state, now_ms, cost)?;
        *state = next;
        Ok(decision)
    }

    pub(super) fn decide(&self, cost: u64) -> Result<Decision, PlatformError> {
        let now_ms = u64::try_from(self.started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
        self.decide_at(now_ms, cost)
    }
}

#[cfg(test)]
mod refinement_tests {
    use super::*;
    use std::sync::Barrier;
    #[test]
    fn refinement_cloned_callers_share_one_admission_budget() {
        for _ in 0..16 {
            let limiter = SharedRateLimiter::new(LimitPolicy::fixed_window(3, 1000));
            let gate = Arc::new(Barrier::new(16));
            let workers: Vec<_> = (0..16)
                .map(|_| {
                    let limiter = limiter.clone();
                    let gate = gate.clone();
                    std::thread::spawn(move || {
                        gate.wait();
                        limiter.decide_at(0, 1).unwrap()
                    })
                })
                .collect();
            let mut allowed = 0;
            let mut denied = 0;
            for worker in workers {
                match worker.join().unwrap() {
                    Decision::Allow { .. } => allowed += 1,
                    Decision::Deny { .. } => denied += 1,
                    other => panic!("unexpected admission {other:?}"),
                }
            }
            assert_eq!((allowed, denied), (3, 13));
            assert!(matches!(
                limiter.decide_at(999, 1).unwrap(),
                Decision::Deny { .. }
            ));
            assert!(matches!(
                limiter.decide_at(1000, 1).unwrap(),
                Decision::Allow { .. }
            ));
        }
    }
    #[test]
    fn refinement_poisoned_state_never_becomes_allow() {
        let limiter = SharedRateLimiter::new(LimitPolicy::fixed_window(3, 1000));
        let state = limiter.state.clone();
        assert!(std::thread::spawn(move || {
            let _guard = state.lock().unwrap();
            panic!("intentional poison");
        })
        .join()
        .is_err());
        assert!(matches!(
            limiter.decide_at(0, 1),
            Err(PlatformError::PoisonedRateLimitState)
        ));
    }
}
