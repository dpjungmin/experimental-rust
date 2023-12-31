#![allow(clippy::module_inception)]

mod circuit_breaker;
mod circuit_breaker_manager;

pub use circuit_breaker::{CircuitBreaker, CircuitBreakerError};
pub use circuit_breaker_manager::{CircuitBreakerManager, CircuitBreakerManagerError};
