//! Health checking module for upstream proxies

pub mod checker;
pub mod validator;

pub use checker::{HealthCheckConfig, HealthChecker, HealthCheckResult};
pub use validator::{LocationInfo, ProxyValidationResult, ProxyValidator};
