use experimental_rust::circuit_breaker::{
    CircuitBreakerError, CircuitBreakerManager, CircuitBreakerManagerError,
};
use rand::prelude::*;
use std::time::Duration;

fn foo(s: &str) -> Result<(), &str> {
    let mut rng = rand::thread_rng();
    let probability = rng.gen::<f64>();

    if probability < 0.00001 {
        return Err(s);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let mut manager = CircuitBreakerManager::new();

    manager.add_circuit_breaker(&1, 10, Duration::from_secs(1));
    manager.add_circuit_breaker(&'a', 20, Duration::from_secs(2));
    manager.add_circuit_breaker("server", 30, Duration::from_secs(3));
    manager.add_circuit_breaker(&true, 40, Duration::from_secs(4));

    loop {
        for x in [
            ("1", manager.try_call(&1, || foo("1")).await),
            ("a", manager.try_call(&'a', || foo("a")).await),
            ("server", manager.try_call("server", || foo("server")).await),
            ("true", manager.try_call(&true, || foo("true")).await),
        ] {
            let (key, result) = x;

            let Err(e) = result else {
                continue;
            };

            let CircuitBreakerManagerError::CircuitBreakerError(e) = e else {
                continue;
            };

            match e {
                CircuitBreakerError::OpenCircuit => {
                    // eprintln!("{key} (open circuit)")
                }
                CircuitBreakerError::OperationError(_) => {
                    eprintln!("{key} (operation error)");
                }
            }
        }
    }
}
