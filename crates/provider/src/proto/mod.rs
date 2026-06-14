//! Protobuf types generated from the TRON protocol `.proto` files via tonic_prost_build.
//!
//! These types are intentionally **not** re-exported: callers work with the
//! public domain types in [`crate::types`]. The codec conversions
//! (domain ↔ proto) live alongside the transport implementations.

// Bring in the prost/tonic generated code.
tonic::include_proto!("protocol");
