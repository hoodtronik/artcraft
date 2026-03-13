//! A client for the Wan2GP ArtCraft Bridge API.
//!
//! Communicates with the bridge plugin running inside Wan2GP to provide
//! local open-source model inference for ArtCraft.
//!
//! The bridge API runs on `http://localhost:7861` by default.

pub mod client;
pub mod error;
pub mod types;
