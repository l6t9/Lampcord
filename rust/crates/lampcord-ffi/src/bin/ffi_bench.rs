use std::hint::black_box;

use lampcord_ffi::runtime;
use lampcord_ffi::snapshot::SnapshotVersions;

fn bench<F>(name: &str, iterations: usize, mut operation: F)
where
    F: FnMut(),
{
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        operation();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() / iterations as u128;
    println!("{name}: {ns_per_op} ns/op ({iterations} iterations)");
}

fn main() {
    let client = runtime::benchmark_handle();
    let current = client.snapshot_versions();
    let iterations = 10_000;

    bench("full_snapshot_clone", iterations, || {
        black_box(client.snapshot());
    });
    bench("snapshot_patch_unchanged", iterations, || {
        black_box(client.snapshot_patch(current.clone()));
    });

    let meta_only = SnapshotVersions {
        meta: 0,
        ..current.clone()
    };
    bench("snapshot_patch_meta_only", iterations, || {
        black_box(client.snapshot_patch(meta_only.clone()));
    });

    let messages_only = SnapshotVersions {
        messages: 0,
        ..current.clone()
    };
    bench("snapshot_patch_messages_only", iterations, || {
        black_box(client.snapshot_patch(messages_only.clone()));
    });
}
