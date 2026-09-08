//! Run against the packaged distribution with MAGIKA_RUNTIME_DIR set.
use magika_runtime::{Backend, BackendRequest, FEATURE_SIZE, NUM_LABELS, Runtime};

#[test]
#[ignore = "requires the packaged native backend libraries"]
fn packaged_backend_scores_shapes_and_lifetimes() {
    let gpu = std::env::var("MAGIKA_TEST_BACKEND").as_deref() == Ok("gpu");
    let request = if gpu { BackendRequest::Gpu } else { BackendRequest::Cpu };
    let runtime = Runtime::with_max_batch(request, 8).unwrap();
    assert_eq!(runtime.backend_info().backend(), if gpu { Backend::Gpu } else { Backend::Cpu });
    let mut session = runtime.session().unwrap();
    // The session must own everything needed after its public Runtime is gone.
    drop(runtime);
    let row: Vec<i32> = (0..FEATURE_SIZE).map(|i| (i % 257) as i32).collect();
    let expected: Vec<f32> = include_bytes!("../../tract-runtime/models/model.probe.f32le")
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
        .collect();
    assert_eq!(expected.len(), NUM_LABELS);
    for batch in [1, 9, 1] {
        // Exercise padding, splitting and state reuse.
        let output = session.run(&row.repeat(batch), batch).unwrap();
        assert_eq!(output.len(), batch * NUM_LABELS);
        for scores in output.chunks_exact(NUM_LABELS) {
            for (actual, expected) in scores.iter().zip(&expected) {
                assert!(
                    actual.is_finite() && (actual - expected).abs() <= 1e-3,
                    "{actual} vs {expected}"
                );
            }
        }
    }
    assert!(session.run(&[], 0).is_err());
    assert!(session.run(&row[..1], 1).is_err());
    assert!(session.run(&[], usize::MAX).is_err());
    assert!(Runtime::with_max_batch(request, 0).is_err());
}
