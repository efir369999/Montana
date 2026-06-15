// mt-conformance — публичный набор binding test vectors из Montana spec
// для cross-implementation verification. M9 milestone deliverable.
//
// Использование из второй реализации:
//
//   let v = vectors::envelope_a1();
//   let actual = your_implementation::encode(&v.input);
//   assert_eq!(actual, v.expected_bytes, "Vector A1 byte mismatch");
//
// Все векторы извлечены из spec разделов A (envelope), B (IBT), C (per-msg),
// D (MeshFrame), E (SF envelope), F (Bootstrap PoW target).

pub mod harness;
pub mod vectors;

pub use vectors::*;
