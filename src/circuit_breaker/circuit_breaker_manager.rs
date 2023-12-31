use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    time::Duration,
};

use fnv::FnvHasher;

use super::{CircuitBreaker, CircuitBreakerError};

pub enum CircuitBreakerManagerError<E> {
    UnknownKey,
    CircuitBreakerError(CircuitBreakerError<E>),
}

#[derive(Default)]
pub struct CircuitBreakerManager {
    circuit_breakers: HashMap<u64, CircuitBreaker>,
}

impl CircuitBreakerManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new circuit breaker.
    ///
    /// If the key was present, the value is updated.
    pub fn add_circuit_breaker<K>(
        &mut self,
        key: &K,
        failure_threshold: u32,
        reset_timeout: Duration,
    ) where
        K: Hash + ?Sized,
    {
        let hashed_key = self.compute_key(key);
        let circuit_breaker = CircuitBreaker::new(failure_threshold, reset_timeout);

        self.circuit_breakers.insert(hashed_key, circuit_breaker);
    }

    pub async fn try_call<K, F, T, E>(
        &self,
        key: &K,
        f: F,
    ) -> Result<T, CircuitBreakerManagerError<E>>
    where
        K: Hash + ?Sized,
        F: FnOnce() -> Result<T, E> + Send,
        T: Send,
    {
        let hashed_key = self.compute_key(key);

        let Some(circuit_breakers) = self.circuit_breakers.get(&hashed_key) else {
            return Err(CircuitBreakerManagerError::UnknownKey);
        };

        circuit_breakers
            .try_call(f)
            .await
            .map_err(|e| CircuitBreakerManagerError::CircuitBreakerError(e))
    }

    fn compute_key<T>(&self, key: &T) -> u64
    where
        T: Hash + ?Sized,
    {
        let mut hasher = FnvHasher::default();
        key.hash(&mut hasher);
        hasher.finish()
    }
}
