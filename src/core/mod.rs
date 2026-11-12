//! 纯逻辑层：记录模型与过滤。不依赖任何 UI / 平台 API。

pub mod model;

pub use model::{Item, Kind, initial_items};
