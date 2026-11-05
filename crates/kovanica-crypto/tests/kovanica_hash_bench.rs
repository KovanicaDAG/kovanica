use kovanica_crypto::KovanicaHasher;
use std::time::Instant;

#[test]
fn kovanica_hash_throughput_bench() {
    let mut hasher = KovanicaHasher::new();
    let preimage = [0x42u8; 132]; // canonical 1-parent header size

    let iterations = 20;
    let start = Instant::now();

    for i in 0..iterations {
        let mut buf = preimage;
        buf[124..132].copy_from_slice(&(i as u64).to_le_bytes());
        let _h = hasher.hash(&buf);
    }

    let elapsed = start.elapsed();
    let per_hash_ms = elapsed.as_secs_f64() * 1000.0 / (iterations as f64);
    let hashes_per_sec = (iterations as f64) / elapsed.as_secs_f64();

    println!(
        "KovanicaHash throughput: {} iterations in {:.2?} ({:.2} ms/hash, {:.1} H/s)",
        iterations, elapsed, per_hash_ms, hashes_per_sec
    );

    // Sanity check: must complete in reasonable time (< 200 ms per hash even in unoptimized debug mode)
    assert!(
        per_hash_ms < 500.0,
        "Hasher is too slow: {:.2} ms/hash",
        per_hash_ms
    );
}
