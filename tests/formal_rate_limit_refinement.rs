//! Observable refinement of formal/safety-interleavings/check.rs.
//! Uses the real SharedRateLimiter and pinned ores-rl-lib-core transition.
//! Fixed timestamp isolates admission atomicity; refill/clock policy is out of scope.
#![cfg(feature = "server")]
use fiducia_lib_core::server::SharedRateLimiter;
use ores_rl_lib_core::{Decision, LimitPolicy};
use std::sync::{Arc, Barrier};
use std::thread;

fn parallel_decisions(capacity: u64, contenders: usize) -> (usize, usize) {
    let limiter = SharedRateLimiter::new(LimitPolicy::fixed_window(capacity, 60_000));
    let start = Arc::new(Barrier::new(contenders + 1));
    let mut workers = Vec::new();
    for _ in 0..contenders {
        let limiter = limiter.clone();
        let start = start.clone();
        workers.push(thread::spawn(move || {
            start.wait();
            limiter
                .decide_at(0, 1)
                .expect("valid fixed-window transition")
        }));
    }
    start.wait();

    let mut allowed = 0;
    let mut denied = 0;
    for worker in workers {
        match worker.join().expect("no limiter panic") {
            Decision::Allow { .. } => allowed += 1,
            Decision::Deny { .. } => denied += 1,
            Decision::Bypass { .. } => panic!("fixed-window policy must not bypass"),
        }
    }
    (allowed, denied)
}

#[test]
fn overlapping_callers_cannot_double_spend_capacity_one() {
    for _ in 0..12 {
        assert_eq!(parallel_decisions(1, 16), (1, 15));
    }
}

#[test]
fn overlapping_callers_saturate_capacity_two_exactly() {
    for _ in 0..12 {
        assert_eq!(parallel_decisions(2, 16), (2, 14));
    }
}

#[test]
fn cloned_handles_share_the_same_admission_counter() {
    let a = SharedRateLimiter::new(LimitPolicy::fixed_window(1, 60_000));
    let b = a.clone();
    assert!(matches!(a.decide_at(0, 1).unwrap(), Decision::Allow { .. }));
    assert!(matches!(b.decide_at(0, 1).unwrap(), Decision::Deny { .. }));
}
