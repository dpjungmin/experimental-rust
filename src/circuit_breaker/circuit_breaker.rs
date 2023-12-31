use std::time::{Duration, Instant};

use tokio::sync::Mutex;

enum State {
    Closed,
    Open(Instant),
    HalfOpen,
}

pub enum CircuitBreakerError<E> {
    OpenCircuit,
    OperationError(E),
}

pub struct CircuitBreaker {
    state: Mutex<State>,
    failure_threshold: u32,
    failure_count: Mutex<u32>,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        Self {
            state: Mutex::new(State::Closed),
            failure_threshold,
            failure_count: Mutex::new(0),
            reset_timeout,
        }
    }

    pub async fn try_call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Fn() -> Result<T, E> + Send,
        T: Send,
    {
        let mut state = self.state.lock().await;

        match *state {
            State::Closed => match f() {
                Ok(x) => Ok(x),
                Err(e) => {
                    let mut failure_count = self.failure_count.lock().await;
                    *failure_count += 1;

                    if *failure_count > self.failure_threshold {
                        let reset_time = Instant::now() + self.reset_timeout;
                        *state = State::Open(reset_time);
                    }

                    Err(CircuitBreakerError::OperationError(e))
                }
            },
            State::HalfOpen => {
                drop(state);
                self.try_again(f).await
            }
            State::Open(ref reset_time) => {
                if Instant::now() < *reset_time {
                    return Err(CircuitBreakerError::OpenCircuit);
                }

                *state = State::HalfOpen;
                drop(state);
                self.try_again(f).await
            }
        }
    }

    async fn try_again<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: Fn() -> Result<T, E> + Send,
        T: Send,
    {
        let mut state = self.state.lock().await;

        match f() {
            Ok(x) => {
                *state = State::Closed;
                *self.failure_count.lock().await = 0;
                Ok(x)
            }
            Err(e) => {
                let reset_time = Instant::now() + self.reset_timeout;
                *state = State::Open(reset_time);
                Err(CircuitBreakerError::OperationError(e))
            }
        }
    }
}
