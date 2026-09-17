//! Static contract tests for the private PostgreSQL benchmark discovery projection.

const POSTGRES_ADAPTER: &str = include_str!("../src/postgres.rs");

#[test]
fn benchmark_discovery_counts_each_dataset_case_once_across_run_joins() {
    assert!(
        POSTGRES_ADAPTER.contains("COUNT(DISTINCT dataset_case.case_id) AS case_count"),
        "benchmark discovery must not multiply dataset case counts by joined decision runs"
    );
}
