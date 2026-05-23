

pub mod mailer;
pub mod redis;

// Re-export for easier access
pub use mailer::{create_mailer, MailConfig};
pub use redis::create_redis_pool;