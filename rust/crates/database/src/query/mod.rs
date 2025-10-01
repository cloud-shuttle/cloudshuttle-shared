//! Query building and execution utilities
//!
//! This module provides comprehensive query building, filtering, sorting,
//! batch operations, and execution options for database operations.

pub mod builder;
pub mod results;
pub mod filters;
pub mod batch;
pub mod options;

// Re-export commonly used types for convenience
pub use builder::QueryBuilder;
pub use results::{QueryResult, PaginatedResult, QueryStats, QueryExecutionContext as QueryResultContext};
pub use filters::{SortOption, SortDirection, FilterOption, FilterOperator, FilterBuilder};
pub use batch::{BatchOperation, BatchResult, BatchBuilder, BatchOptions};
pub use options::{QueryOptions, QueryExecutionContext, QueryMetrics, QueryHints};

// Re-export all types for backward compatibility
pub use builder::*;
pub use results::*;
pub use filters::*;
pub use batch::*;
pub use options::*;
