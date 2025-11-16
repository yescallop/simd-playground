use pct_enc::*;

#[test]
fn test_validate() {
    let fns = [
        sse41::validate_triple_loadu,
        sse41::validate_alignr,
        sse41::validate_bsrli_transposed,
        sse41::validate_bslli,
        sse41::validate_bslli_transposed,
        avx2::validate_triple_loadu,
        avx2::validate_alignr,
    ];

    let good = b"%3C%88,Kh%9C%3E%90%3F@%BB%B4%E8%96%18%9F%3C%5C%93@%1D%CD%25%13%3F%99%1CP%FA%88%EA";
    let not_hexdig =
        b"%3C%8,Kh%9C%3E%90%3F@%BB%B4%E8%96%18%9F%3C%5C%93@%1D%CD%25%13%3F%99%1CP%FA%88%EA";
    let null =
        b"%3C%88\0,Kh%9C%3E%90%3F@%BB%B4%E8%96%18%9F%3C%5C%93@%1D%CD%25%13%3F%99%1CP%FA%88%EA";
    let non_ascii =
        b"%3C%88\xf0,Kh%9C%3E%90%3F@%BB%B4%E8%96%18%9F%3C%5C%93@%1D%CD%25%13%3F%99%1CP%FA%88%EA";

    for (i, f) in fns.into_iter().enumerate() {
        unsafe {
            assert!(f(good), "good failed on {i}");
            assert!(!f(not_hexdig), "disallowed failed on {i}");
            assert!(!f(null), "null failed on {i}");
            assert!(!f(non_ascii), "non-ASCII failed on {i}");
        }
    }
}
