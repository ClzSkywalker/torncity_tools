use super::error::OrderChangeError;
use super::Result;

pub trait Recoverable<T> {
    fn recover_with_fallback<F>(self, fallback: F) -> RecoveryResult<T>
    where
        F: FnOnce() -> T;

    fn recover_with_error<F>(self, error_handler: F) -> RecoveryResult<T>
    where
        F: FnOnce(&OrderChangeError) -> T;

    fn recover_with_callback<F, C>(self, fallback: F, callback: C) -> RecoveryResult<T>
    where
        F: FnOnce() -> T,
        C: FnOnce(&OrderChangeError);
}

pub enum RecoveryResult<T> {
    Success(T),
    Recovered(T, OrderChangeError),
    Failed(OrderChangeError),
}

impl<T> RecoveryResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, RecoveryResult::Success(_))
    }

    pub fn is_recovered(&self) -> bool {
        matches!(self, RecoveryResult::Recovered(_, _))
    }

    pub fn is_failed(&self) -> bool {
        matches!(self, RecoveryResult::Failed(_))
    }

    pub fn get_value(self) -> Option<T> {
        match self {
            RecoveryResult::Success(v) | RecoveryResult::Recovered(v, _) => Some(v),
            RecoveryResult::Failed(_) => None,
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            RecoveryResult::Success(v) | RecoveryResult::Recovered(v, _) => v,
            RecoveryResult::Failed(e) => panic!("Failed to unwrap: {}", e),
        }
    }

    pub fn unwrap_or<F>(self, fallback: F) -> T
    where
        F: FnOnce() -> T,
    {
        match self {
            RecoveryResult::Success(v) | RecoveryResult::Recovered(v, _) => v,
            RecoveryResult::Failed(_) => fallback(),
        }
    }
}

impl<T, E> Recoverable<T> for std::result::Result<T, E>
where
    E: Into<OrderChangeError>,
{
    fn recover_with_fallback<F>(self, fallback: F) -> RecoveryResult<T>
    where
        F: FnOnce() -> T,
    {
        match self {
            Ok(v) => RecoveryResult::Success(v),
            Err(e) => {
                let order_error = e.into();
                if order_error.is_recoverable() {
                    let fallback_value = fallback();
                    RecoveryResult::Recovered(fallback_value, order_error)
                } else {
                    RecoveryResult::Failed(order_error)
                }
            }
        }
    }

    fn recover_with_error<F>(self, error_handler: F) -> RecoveryResult<T>
    where
        F: FnOnce(&OrderChangeError) -> T,
    {
        match self {
            Ok(v) => RecoveryResult::Success(v),
            Err(e) => {
                let order_error = e.into();
                let fallback_value = error_handler(&order_error);
                RecoveryResult::Recovered(fallback_value, order_error)
            }
        }
    }

    fn recover_with_callback<F, C>(self, fallback: F, callback: C) -> RecoveryResult<T>
    where
        F: FnOnce() -> T,
        C: FnOnce(&OrderChangeError),
    {
        match self {
            Ok(v) => RecoveryResult::Success(v),
            Err(e) => {
                let order_error = e.into();
                callback(&order_error);
                if order_error.is_recoverable() {
                    let fallback_value = fallback();
                    RecoveryResult::Recovered(fallback_value, order_error)
                } else {
                    RecoveryResult::Failed(order_error)
                }
            }
        }
    }
}

pub struct SafeRenderer {
    max_retries: usize,
    current_retry: usize,
}

impl Default for SafeRenderer {
    fn default() -> Self {
        Self {
            max_retries: 3,
            current_retry: 0,
        }
    }
}

impl SafeRenderer {
    pub fn new(max_retries: usize) -> Self {
        Self {
            max_retries,
            current_retry: 0,
        }
    }

    pub fn render_with_retry<F, R>(&mut self, render_fn: F) -> RecoveryResult<R>
    where
        F: Fn() -> Result<R>,
    {
        loop {
            match render_fn() {
                Ok(result) => return RecoveryResult::Success(result),
                Err(e) => {
                    if e.is_recoverable() && self.current_retry < self.max_retries {
                        self.current_retry += 1;
                        continue;
                    } else {
                        return RecoveryResult::Failed(e);
                    }
                }
            }
        }
    }
}

pub struct ErrorHandler;

impl ErrorHandler {
    pub fn log_error(error: &OrderChangeError, context: &str) {
        if error.is_critical() {
            godot::global::godot_error!("{}: Critical error: {}", context, error);
        } else {
            godot::global::godot_warn!("{}: Recoverable error: {}", context, error);
        }
    }

    pub fn handle_with_fallback<T, F>(
        result: Result<T>,
        fallback: F,
        context: &str,
    ) -> T
    where
        F: FnOnce() -> T,
    {
        match result {
            Ok(v) => v,
            Err(e) => {
                Self::log_error(&e, context);
                fallback()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order_change::error::OrderChangeError;

    #[test]
    fn test_recovery_success() {
        let result: Result<i32> = Ok(42);
        let recovery = result.recover_with_fallback(|| 0);
        assert!(recovery.is_success());
        assert_eq!(recovery.unwrap(), 42);
    }

    #[test]
    fn test_recovery_recoverable() {
        let result: Result<i32> = Err(OrderChangeError::LengthMismatch {
            current: 1,
            target: 2,
        });
        let recovery = result.recover_with_fallback(|| 0);
        assert!(recovery.is_recovered());
        assert_eq!(recovery.unwrap(), 0);
    }

    #[test]
    fn test_recovery_failed() {
        let result: Result<i32> = Err(OrderChangeError::InvalidData("test".to_string()));
        let recovery = result.recover_with_fallback(|| 0);
        assert!(recovery.is_failed());
        assert!(recovery.unwrap_or(|| -1) == -1);
    }
}
