//! Exhaustive finite-state safety check; not an unbounded or implementation proof.
use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

fn explore<S: Clone + Eq + Hash + Debug>(
    initial: S,
    next: impl Fn(&S) -> Vec<(&'static str, S)>,
    invariant: impl Fn(&S) -> bool,
) -> Result<(usize, usize), Vec<&'static str>> {
    assert!(invariant(&initial), "invalid initial state");
    let mut seen = HashSet::from([initial.clone()]);
    let mut queue = VecDeque::from([(initial, Vec::new())]);
    let mut edges = 0;
    while let Some((state, trace)) = queue.pop_front() {
        for (action, successor) in next(&state) {
            edges += 1;
            let mut path = trace.clone();
            path.push(action);
            if !invariant(&successor) {
                eprintln!("counterexample {path:?}: {successor:?}");
                return Err(path);
            }
            if seen.insert(successor.clone()) {
                assert!(
                    seen.len() <= 200_000,
                    "state budget exceeded: incomplete exploration"
                );
                queue.push_back((successor, path));
            }
        }
    }
    println!("bounded fixed point: {} states, {edges} edges", seen.len());
    Ok((seen.len(), edges))
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
struct State {
    pc: [u8; 2],
    snapshot: [u8; 2],
    used: u8,
    allowed: u8,
    owner: Option<usize>,
}
// A capacity-one fixed window; each caller reads then commits admission.
fn transitions(s: &State, unlocked: bool) -> Vec<(&'static str, State)> {
    let mut out = vec![];
    for i in 0..2 {
        let mut n = s.clone();
        if s.pc[i] == 0 && (unlocked || s.owner.is_none()) {
            n.pc[i] = 1;
            n.snapshot[i] = s.used;
            if !unlocked {
                n.owner = Some(i);
            }
            out.push(("read-admission-state", n));
        } else if s.pc[i] == 1 {
            n.pc[i] = 2;
            if s.snapshot[i] < 1 {
                n.used = s.snapshot[i] + 1;
                n.allowed += 1;
            }
            n.owner = None;
            out.push(("commit-admission", n));
        }
    }
    out
}
fn valid(s: &State) -> bool {
    s.allowed <= 1 && s.allowed == s.used
}
#[test]
fn serialized_read_decide_commit_respects_capacity() {
    assert_eq!(
        explore(
            State {
                pc: [0; 2],
                snapshot: [0; 2],
                used: 0,
                allowed: 0,
                owner: None
            },
            |s| transitions(s, false),
            valid
        )
        .unwrap(),
        (9, 8)
    );
}
#[test]
fn moving_transition_outside_mutex_overadmits() {
    let trace = explore(
        State {
            pc: [0; 2],
            snapshot: [0; 2],
            used: 0,
            allowed: 0,
            owner: None,
        },
        |s| transitions(s, true),
        valid,
    )
    .unwrap_err();
    assert_eq!(trace.len(), 4);
    assert_eq!(trace.last(), Some(&"commit-admission"));
}
