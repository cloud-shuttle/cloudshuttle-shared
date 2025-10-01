//! Query filtering and condition building

use serde::{Deserialize, Serialize};

/// Sorting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOption {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,
    #[serde(rename = "desc")]
    Descending,
}

impl SortOption {
    pub fn asc<S: Into<String>>(field: S) -> Self {
        Self {
            field: field.into(),
            direction: SortDirection::Ascending,
        }
    }

    pub fn desc<S: Into<String>>(field: S) -> Self {
        Self {
            field: field.into(),
            direction: SortDirection::Descending,
        }
    }

    pub fn to_sql(&self) -> String {
        match self.direction {
            SortDirection::Ascending => self.field.clone(),
            SortDirection::Descending => format!("{} DESC", self.field),
        }
    }
}

/// Filtering options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOption {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    #[serde(rename = "eq")]
    Equal,
    #[serde(rename = "ne")]
    NotEqual,
    #[serde(rename = "gt")]
    GreaterThan,
    #[serde(rename = "lt")]
    LessThan,
    #[serde(rename = "gte")]
    GreaterThanOrEqual,
    #[serde(rename = "lte")]
    LessThanOrEqual,
    #[serde(rename = "like")]
    Like,
    #[serde(rename = "in")]
    In,
    #[serde(rename = "nin")]
    NotIn,
    #[serde(rename = "null")]
    IsNull,
    #[serde(rename = "nnull")]
    IsNotNull,
    #[serde(rename = "between")]
    Between,
    #[serde(rename = "contains")]
    Contains,
    #[serde(rename = "startswith")]
    StartsWith,
    #[serde(rename = "endswith")]
    EndsWith,
}

impl FilterOperator {
    /// Convert operator to SQL condition fragment
    pub fn to_sql_condition(&self, field: &str, param_index: usize) -> String {
        match self {
            FilterOperator::Equal => format!("{} = ${}", field, param_index),
            FilterOperator::NotEqual => format!("{} != ${}", field, param_index),
            FilterOperator::GreaterThan => format!("{} > ${}", field, param_index),
            FilterOperator::LessThan => format!("{} < ${}", field, param_index),
            FilterOperator::GreaterThanOrEqual => format!("{} >= ${}", field, param_index),
            FilterOperator::LessThanOrEqual => format!("{} <= ${}", field, param_index),
            FilterOperator::Like => format!("{} LIKE ${}", field, param_index),
            FilterOperator::In => format!("{} = ANY(${})", field, param_index),
            FilterOperator::NotIn => format!("{} != ALL(${})", field, param_index),
            FilterOperator::IsNull => format!("{} IS NULL", field),
            FilterOperator::IsNotNull => format!("{} IS NOT NULL", field),
            FilterOperator::Between => format!("{} BETWEEN ${} AND ${}", field, param_index, param_index + 1),
            FilterOperator::Contains => format!("{} @> ${}", field, param_index),
            FilterOperator::StartsWith => format!("{} LIKE ${}", field, param_index),
            FilterOperator::EndsWith => format!("{} LIKE ${}", field, param_index),
        }
    }

    /// Check if operator requires a parameter
    pub fn requires_parameter(&self) -> bool {
        !matches!(self, FilterOperator::IsNull | FilterOperator::IsNotNull)
    }

    /// Check if operator requires two parameters (e.g., BETWEEN)
    pub fn requires_two_parameters(&self) -> bool {
        matches!(self, FilterOperator::Between)
    }

    /// Convert operator to human-readable string
    pub fn as_str(&self) -> &'static str {
        match self {
            FilterOperator::Equal => "equals",
            FilterOperator::NotEqual => "not equals",
            FilterOperator::GreaterThan => "greater than",
            FilterOperator::LessThan => "less than",
            FilterOperator::GreaterThanOrEqual => "greater than or equal",
            FilterOperator::LessThanOrEqual => "less than or equal",
            FilterOperator::Like => "like",
            FilterOperator::In => "in",
            FilterOperator::NotIn => "not in",
            FilterOperator::IsNull => "is null",
            FilterOperator::IsNotNull => "is not null",
            FilterOperator::Between => "between",
            FilterOperator::Contains => "contains",
            FilterOperator::StartsWith => "starts with",
            FilterOperator::EndsWith => "ends with",
        }
    }
}

/// Filter group for complex AND/OR logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterGroup {
    pub operator: FilterGroupOperator,
    pub filters: Vec<FilterCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterGroupOperator {
    #[serde(rename = "and")]
    And,
    #[serde(rename = "or")]
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterCondition {
    Filter(FilterOption),
    Group(FilterGroup),
}

/// Query filter builder for complex filtering logic
pub struct FilterBuilder {
    conditions: Vec<FilterCondition>,
}

impl FilterBuilder {
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
        }
    }

    /// Add a simple filter condition
    pub fn filter(mut self, field: impl Into<String>, operator: FilterOperator, value: serde_json::Value) -> Self {
        self.conditions.push(FilterCondition::Filter(FilterOption {
            field: field.into(),
            operator,
            value,
        }));
        self
    }

    /// Add a filter group
    pub fn group(mut self, group: FilterGroup) -> Self {
        self.conditions.push(FilterCondition::Group(group));
        self
    }

    /// Build the filter conditions
    pub fn build(self) -> Vec<FilterCondition> {
        self.conditions
    }

    /// Create an AND group
    pub fn and_group(filters: Vec<FilterCondition>) -> FilterGroup {
        FilterGroup {
            operator: FilterGroupOperator::And,
            filters,
        }
    }

    /// Create an OR group
    pub fn or_group(filters: Vec<FilterCondition>) -> FilterGroup {
        FilterGroup {
            operator: FilterGroupOperator::Or,
            filters,
        }
    }
}

/// Filter validation and sanitization
pub struct FilterValidator;

impl FilterValidator {
    /// Validate that a filter value is appropriate for the operator
    pub fn validate_filter(filter: &FilterOption) -> Result<(), String> {
        match filter.operator {
            FilterOperator::In | FilterOperator::NotIn => {
                // Must be an array
                if !filter.value.is_array() {
                    return Err(format!("Operator '{}' requires an array value", filter.operator.as_str()));
                }
            }
            FilterOperator::Between => {
                // Must be an array with exactly 2 elements
                if !filter.value.is_array() {
                    return Err(format!("Operator '{}' requires an array value", filter.operator.as_str()));
                }
                if filter.value.as_array().unwrap().len() != 2 {
                    return Err(format!("Operator '{}' requires exactly 2 values", filter.operator.as_str()));
                }
            }
            FilterOperator::IsNull | FilterOperator::IsNotNull => {
                // Should not have a value
                if !filter.value.is_null() {
                    return Err(format!("Operator '{}' should not have a value", filter.operator.as_str()));
                }
            }
            _ => {
                // Other operators should have a non-null value
                if filter.value.is_null() {
                    return Err(format!("Operator '{}' requires a non-null value", filter.operator.as_str()));
                }
            }
        }
        Ok(())
    }

    /// Sanitize filter input to prevent injection
    pub fn sanitize_filter_input(input: &str) -> String {
        // Basic sanitization - remove potentially dangerous characters
        input.chars()
            .filter(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '-'))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_option() {
        let asc = SortOption::asc("name");
        assert_eq!(asc.field, "name");
        assert_eq!(asc.to_sql(), "name");

        let desc = SortOption::desc("created_at");
        assert_eq!(desc.field, "created_at");
        assert_eq!(desc.to_sql(), "created_at DESC");
    }

    #[test]
    fn test_filter_operator_sql() {
        assert_eq!(FilterOperator::Equal.to_sql_condition("field", 1), "field = $1");
        assert_eq!(FilterOperator::GreaterThan.to_sql_condition("age", 2), "age > $2");
        assert_eq!(FilterOperator::Like.to_sql_condition("name", 3), "name LIKE $3");
        assert_eq!(FilterOperator::IsNull.to_sql_condition("deleted_at", 1), "deleted_at IS NULL");
    }

    #[test]
    fn test_filter_operator_requirements() {
        assert!(FilterOperator::Equal.requires_parameter());
        assert!(FilterOperator::Between.requires_two_parameters());
        assert!(!FilterOperator::IsNull.requires_parameter());
        assert!(!FilterOperator::IsNull.requires_two_parameters());
    }

    #[test]
    fn test_filter_builder() {
        let filters = FilterBuilder::new()
            .filter("active", FilterOperator::Equal, serde_json::json!(true))
            .filter("age", FilterOperator::GreaterThan, serde_json::json!(18))
            .build();

        assert_eq!(filters.len(), 2);
    }

    #[test]
    fn test_filter_validator() {
        // Valid filter
        let valid_filter = FilterOption {
            field: "name".to_string(),
            operator: FilterOperator::Equal,
            value: serde_json::json!("test"),
        };
        assert!(FilterValidator::validate_filter(&valid_filter).is_ok());

        // Invalid IN filter (not an array)
        let invalid_in_filter = FilterOption {
            field: "tags".to_string(),
            operator: FilterOperator::In,
            value: serde_json::json!("not_an_array"),
        };
        assert!(FilterValidator::validate_filter(&invalid_in_filter).is_err());

        // Invalid BETWEEN filter (wrong number of values)
        let invalid_between_filter = FilterOption {
            field: "range".to_string(),
            operator: FilterOperator::Between,
            value: serde_json::json!([1, 2, 3]), // Should be exactly 2
        };
        assert!(FilterValidator::validate_filter(&invalid_between_filter).is_err());
    }

    #[test]
    fn test_filter_sanitization() {
        assert_eq!(FilterValidator::sanitize_filter_input("user_name"), "user_name");
        assert_eq!(FilterValidator::sanitize_filter_input("user'name"), "username");
        assert_eq!(FilterValidator::sanitize_filter_input("user<script>name"), "userscriptname");
    }

    #[test]
    fn test_filter_group() {
        let group = FilterBuilder::and_group(vec![
            FilterCondition::Filter(FilterOption {
                field: "active".to_string(),
                operator: FilterOperator::Equal,
                value: serde_json::json!(true),
            }),
            FilterCondition::Filter(FilterOption {
                field: "age".to_string(),
                operator: FilterOperator::GreaterThan,
                value: serde_json::json!(18),
            }),
        ]);

        assert_eq!(group.filters.len(), 2);
        assert!(matches!(group.operator, FilterGroupOperator::And));
    }
}
