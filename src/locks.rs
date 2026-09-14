//! Backend-neutral fenced lock identities for Fiducia services.
//!
//! These keys protect orchestration and external side effects around the Raft
//! state machines. They do **not** replace Raft quorum, log replication, term
//! leadership, or membership safety. In particular, Fiducia may back these
//! leases with Cloudflare Durable Objects so the system does not recursively
//! require Fiducia merely to coordinate Fiducia startup/recovery work.

use ores_locks_and_leases::LockKey;

const PREFIX: &str = "fiducia-cloud";

fn key(domain: &str, name: &str) -> LockKey {
    LockKey::new(format!("{PREFIX}/{domain}/{name}"))
        .expect("static Fiducia lock prefix and validated resource names must fit LockKey")
}

/// Serialize application of one brain placement/reconciliation plan outside
/// the replicated Raft decision itself.
pub fn brain_plan_apply(plan_id: &str) -> LockKey {
    key("brain", &format!("plan-apply:{plan_id}"))
}

/// Fence an external shard bootstrap/recovery action for one shard.
pub fn shard_bootstrap(shard_id: &str) -> LockKey {
    key("node", &format!("shard-bootstrap:{shard_id}"))
}

/// One migration runner across a deployment at a time.
pub fn migration(component: &str) -> LockKey {
    key("migrations", component)
}

/// Singleton maintenance jobs that must not overlap across replicas.
pub fn singleton_job(job: &str) -> LockKey {
    key("jobs", &format!("singleton:{job}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brain_and_node_keys_are_disjoint_and_namespaced() {
        let plan = brain_plan_apply("p-42");
        let shard = shard_bootstrap("17");
        assert_eq!(plan.as_str(), "fiducia-cloud/brain/plan-apply:p-42");
        assert_eq!(shard.as_str(), "fiducia-cloud/node/shard-bootstrap:17");
        assert_ne!(plan, shard);
    }

    #[test]
    fn migrations_and_jobs_have_stable_domains() {
        assert_eq!(migration("brain").as_str(), "fiducia-cloud/migrations/brain");
        assert_eq!(
            singleton_job("membership-repair").as_str(),
            "fiducia-cloud/jobs/singleton:membership-repair"
        );
    }
}
