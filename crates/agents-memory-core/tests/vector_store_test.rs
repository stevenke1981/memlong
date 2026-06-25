use agents_memory_core::storage::VectorStore;
use tempfile::tempdir;

#[test]
fn dimension_mismatch_returns_actionable_error() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("vectors.usearch");
    let path = path.to_string_lossy().into_owned();

    // Create store with dim=4 and add one vector
    {
        let store = VectorStore::new(&path, 4).unwrap();
        store.add(1, &[0.1, 0.2, 0.3, 0.4]).unwrap();
    }

    // Reopen with dim=8 → should error with actionable message
    let err_msg = match VectorStore::new(&path, 8) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("expected dimension mismatch error"),
    };
    assert!(
        err_msg.contains("Dimension mismatch"),
        "error should mention dimension mismatch, got: {err_msg}"
    );
    assert!(
        err_msg.contains("8"),
        "error should mention configured dim 8, got: {err_msg}"
    );
    assert!(
        err_msg.contains("4"),
        "error should mention actual index dim 4, got: {err_msg}"
    );
}

#[test]
fn remove_is_immediate_and_persistent() {
    let tmp = tempdir().unwrap();
    let path = tmp.path().join("vectors.usearch");
    let path = path.to_string_lossy().into_owned();

    {
        let store = VectorStore::new(&path, 3).unwrap();
        store.add(7, &[1.0, 0.0, 0.0]).unwrap();
        assert_eq!(store.search(&[1.0, 0.0, 0.0], 5).unwrap().len(), 1);

        store.remove(7).unwrap();
        assert!(store.search(&[1.0, 0.0, 0.0], 5).unwrap().is_empty());
        assert_eq!(store.size(), 0);
    }

    let reopened = VectorStore::new(&path, 3).unwrap();
    assert!(reopened.search(&[1.0, 0.0, 0.0], 5).unwrap().is_empty());
    assert_eq!(reopened.size(), 0);
}
