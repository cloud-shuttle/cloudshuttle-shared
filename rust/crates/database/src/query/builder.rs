//! Query builder for dynamic SQL construction

use crate::types::models::{QueryCriteria, Filter, SortOrder, Pagination};
use serde_json;

/// Query builder for dynamic SQL construction
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    table: String,
    select_fields: Vec<String>,
    where_conditions: Vec<String>,
    params: Vec<serde_json::Value>,
    order_by: Vec<String>,
    limit: Option<usize>,
    offset: Option<usize>,
    joins: Vec<String>,
}

impl QueryBuilder {
    /// Create a new query builder for a table
    pub fn new<S: Into<String>>(table: S) -> Self {
        Self {
            table: table.into(),
            select_fields: vec!["*".to_string()],
            where_conditions: Vec::new(),
            params: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            joins: Vec::new(),
        }
    }

    /// Specify which fields to select
    pub fn select<S: Into<String>>(mut self, fields: Vec<S>) -> Self {
        self.select_fields = fields.into_iter().map(|s| s.into()).collect();
        self
    }

    /// Add a WHERE condition
    pub fn where_eq<S: Into<String>, V: Into<serde_json::Value>>(mut self, field: S, value: V) -> Self {
        let condition = format!("{} = ${}", field.into(), self.params.len() + 1);
        self.where_conditions.push(condition);
        self.params.push(value.into());
        self
    }

    /// Add a WHERE condition with custom operator
    pub fn where_condition<S: Into<String>>(mut self, condition: S) -> Self {
        self.where_conditions.push(condition.into());
        self
    }

    /// Add an ORDER BY clause
    pub fn order_by<S: Into<String>>(mut self, field: S, direction: Option<&str>) -> Self {
        let order = match direction {
            Some("desc") | Some("DESC") => format!("{} DESC", field.into()),
            _ => field.into(),
        };
        self.order_by.push(order);
        self
    }

    /// Add a LIMIT clause
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add an OFFSET clause
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Add a JOIN clause
    pub fn join<S: Into<String>>(mut self, join: S) -> Self {
        self.joins.push(join.into());
        self
    }

    /// Build the final SQL query and parameters
    pub fn build(self) -> (String, Vec<serde_json::Value>) {
        let mut sql = String::new();

        // SELECT clause
        sql.push_str("SELECT ");
        sql.push_str(&self.select_fields.join(", "));
        sql.push_str(" FROM ");
        sql.push_str(&self.table);

        // JOIN clauses
        for join in &self.joins {
            sql.push_str(" ");
            sql.push_str(join);
        }

        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_conditions.join(" AND "));
        }

        // ORDER BY clause
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            sql.push_str(&self.order_by.join(", "));
        }

        // LIMIT clause
        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // OFFSET clause
        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        (sql, self.params)
    }

    /// Build a count query
    pub fn build_count(self) -> (String, Vec<serde_json::Value>) {
        let mut sql = String::new();

        sql.push_str("SELECT COUNT(*) FROM ");
        sql.push_str(&self.table);

        // JOIN clauses
        for join in &self.joins {
            sql.push_str(" ");
            sql.push_str(join);
        }

        // WHERE clause
        if !self.where_conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.where_conditions.join(" AND "));
        }

        (sql, self.params)
    }

    /// Apply QueryCriteria to the builder
    pub fn apply_criteria(mut self, criteria: QueryCriteria) -> Self {
        // Apply filters
        for filter in criteria.filters {
            match filter.operator {
                crate::types::models::FilterOperator::Equal => {
                    self = self.where_eq(filter.field, filter.value);
                }
                // Add other operators as needed
                _ => {
                    // For now, add as custom condition
                    let condition = format!("{} {} ?", filter.field, filter.operator.to_string());
                    self = self.where_condition(condition);
                    self.params.push(filter.value);
                }
            }
        }

        // Apply sorting
        for sort in criteria.sorting {
            let direction = match sort.direction {
                crate::types::models::SortDirection::Descending => Some("DESC"),
                _ => None,
            };
            self = self.order_by(sort.field, direction);
        }

        // Apply pagination
        if let Some(pagination) = criteria.pagination {
            if let Some(limit) = pagination.page_size.checked_mul(pagination.page) {
                self = self.limit(limit as usize);
            }
            if pagination.page > 1 {
                let offset = (pagination.page - 1) * pagination.page_size;
                self = self.offset(offset as usize);
            }
        }

        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_basic() {
        let (sql, params) = QueryBuilder::new("users")
            .select(vec!["id", "name", "email"])
            .where_eq("active", true)
            .order_by("name", None)
            .limit(10)
            .build();

        assert!(sql.contains("SELECT id, name, email FROM users"));
        assert!(sql.contains("WHERE active = $1"));
        assert!(sql.contains("ORDER BY name"));
        assert!(sql.contains("LIMIT 10"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_query_builder_count() {
        let (sql, params) = QueryBuilder::new("users")
            .where_eq("active", true)
            .build_count();

        assert!(sql.contains("SELECT COUNT(*) FROM users"));
        assert!(sql.contains("WHERE active = $1"));
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_query_builder_complex() {
        let (sql, params) = QueryBuilder::new("users")
            .select(vec!["id", "name"])
            .where_eq("active", true)
            .where_condition("age > 18")
            .join("LEFT JOIN profiles p ON users.id = p.user_id")
            .order_by("name", Some("DESC"))
            .limit(50)
            .offset(100)
            .build();

        assert!(sql.contains("SELECT id, name FROM users"));
        assert!(sql.contains("LEFT JOIN profiles p ON users.id = p.user_id"));
        assert!(sql.contains("WHERE active = $1 AND age > 18"));
        assert!(sql.contains("ORDER BY name DESC"));
        assert!(sql.contains("LIMIT 50"));
        assert!(sql.contains("OFFSET 100"));
        assert_eq!(params.len(), 1);
    }
}
