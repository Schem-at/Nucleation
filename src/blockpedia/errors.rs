use std::error::Error as StdError;
use std::fmt;

/// Main error type for all Blockpedia operations
#[derive(Debug, Clone, PartialEq)]
pub enum BlockpediaError {
    /// Block-related errors
    Block(BlockError),
    /// Property-related errors
    Property(PropertyError),
    /// BlockState parsing and validation errors
    State(StateError),
    /// Query execution errors
    Query(QueryError),
    /// Fetcher-related errors
    Fetcher(FetcherError),
    /// Data validation errors
    Validation(ValidationError),
    /// I/O and data loading errors
    Data(DataError),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockError {
    /// Block ID not found in the database
    NotFound(String),
    /// Block ID format is invalid
    InvalidId(String),
    /// Block data is corrupted or missing required fields
    CorruptedData(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyError {
    /// Property doesn't exist for the specified block
    NotFound { block_id: String, property: String },
    /// Property value is not valid for this property
    InvalidValue {
        block_id: String,
        property: String,
        value: String,
        valid_values: Vec<String>,
    },
    /// Property name format is invalid
    InvalidName(String),
    /// Property has no valid values defined
    NoValues(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum StateError {
    /// BlockState string parsing failed
    ParseFailed { input: String, reason: String },
    /// BlockState validation failed
    ValidationFailed { state: String, errors: Vec<String> },
    /// Attempting to modify immutable state
    ImmutableState(String),
    /// State contains conflicting properties
    ConflictingProperties {
        prop1: String,
        prop2: String,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryError {
    /// Query syntax is invalid
    InvalidSyntax(String),
    /// Query parameters are out of range or invalid
    InvalidParameters(String),
    /// Query execution failed due to data issues
    ExecutionFailed(String),
    /// Query timed out (for future async queries)
    Timeout(String),
    /// No results found for query
    NoResults(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FetcherError {
    /// Fetcher initialization failed
    InitializationFailed(String),
    /// Fetcher data source is unavailable
    DataSourceUnavailable(String),
    /// Fetcher returned invalid data
    InvalidData(String),
    /// Multiple fetchers provide conflicting data
    ConflictingData {
        fetcher1: String,
        fetcher2: String,
        block_id: String,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Input fails format validation
    InvalidFormat {
        input: String,
        expected_format: String,
    },
    /// Input is out of acceptable range
    OutOfRange {
        value: String,
        min: String,
        max: String,
    },
    /// Required field is missing
    MissingRequired(String),
    /// Input contains invalid characters
    InvalidCharacters {
        input: String,
        invalid_chars: Vec<char>,
    },
    /// Input is too long or too short
    InvalidLength {
        input: String,
        min_length: usize,
        max_length: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataError {
    /// JSON parsing failed
    JsonParse(String),
    /// Network request failed
    NetworkFailed(String),
    /// File I/O failed
    IoFailed(String),
    /// Data format is not supported
    UnsupportedFormat(String),
    /// Data integrity check failed
    IntegrityCheckFailed(String),
}

/// Convenience type alias for Results with BlockpediaError
pub type Result<T> = std::result::Result<T, BlockpediaError>;

impl fmt::Display for BlockpediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockpediaError::Block(e) => write!(f, "Block error: {}", e),
            BlockpediaError::Property(e) => write!(f, "Property error: {}", e),
            BlockpediaError::State(e) => write!(f, "State error: {}", e),
            BlockpediaError::Query(e) => write!(f, "Query error: {}", e),
            BlockpediaError::Fetcher(e) => write!(f, "Fetcher error: {}", e),
            BlockpediaError::Validation(e) => write!(f, "Validation error: {}", e),
            BlockpediaError::Data(e) => write!(f, "Data error: {}", e),
        }
    }
}

impl fmt::Display for BlockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockError::NotFound(id) => write!(f, "Block '{}' not found", id),
            BlockError::InvalidId(id) => write!(f, "Invalid block ID format: '{}'", id),
            BlockError::CorruptedData(msg) => write!(f, "Block data corrupted: {}", msg),
        }
    }
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PropertyError::NotFound { block_id, property } => {
                write!(
                    f,
                    "Property '{}' not found on block '{}'",
                    property, block_id
                )
            }
            PropertyError::InvalidValue {
                block_id,
                property,
                value,
                valid_values,
            } => {
                write!(
                    f,
                    "Invalid value '{}' for property '{}' on block '{}'. Valid values: {:?}",
                    value, property, block_id, valid_values
                )
            }
            PropertyError::InvalidName(name) => {
                write!(f, "Invalid property name format: '{}'", name)
            }
            PropertyError::NoValues(property) => {
                write!(f, "Property '{}' has no valid values defined", property)
            }
        }
    }
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateError::ParseFailed { input, reason } => {
                write!(f, "Failed to parse BlockState '{}': {}", input, reason)
            }
            StateError::ValidationFailed { state, errors } => {
                write!(
                    f,
                    "BlockState '{}' validation failed: {}",
                    state,
                    errors.join(", ")
                )
            }
            StateError::ImmutableState(msg) => {
                write!(f, "Cannot modify immutable state: {}", msg)
            }
            StateError::ConflictingProperties {
                prop1,
                prop2,
                reason,
            } => {
                write!(
                    f,
                    "Conflicting properties '{}' and '{}': {}",
                    prop1, prop2, reason
                )
            }
        }
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::InvalidSyntax(syntax) => write!(f, "Invalid query syntax: {}", syntax),
            QueryError::InvalidParameters(params) => {
                write!(f, "Invalid query parameters: {}", params)
            }
            QueryError::ExecutionFailed(reason) => write!(f, "Query execution failed: {}", reason),
            QueryError::Timeout(query) => write!(f, "Query timed out: {}", query),
            QueryError::NoResults(query) => write!(f, "No results found for query: {}", query),
        }
    }
}

impl fmt::Display for FetcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FetcherError::InitializationFailed(msg) => {
                write!(f, "Fetcher initialization failed: {}", msg)
            }
            FetcherError::DataSourceUnavailable(source) => {
                write!(f, "Data source unavailable: {}", source)
            }
            FetcherError::InvalidData(msg) => write!(f, "Invalid fetcher data: {}", msg),
            FetcherError::ConflictingData {
                fetcher1,
                fetcher2,
                block_id,
            } => {
                write!(
                    f,
                    "Conflicting data from fetchers '{}' and '{}' for block '{}'",
                    fetcher1, fetcher2, block_id
                )
            }
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidFormat {
                input,
                expected_format,
            } => {
                write!(
                    f,
                    "Invalid format for '{}', expected: {}",
                    input, expected_format
                )
            }
            ValidationError::OutOfRange { value, min, max } => {
                write!(f, "Value '{}' out of range [{}, {}]", value, min, max)
            }
            ValidationError::MissingRequired(field) => {
                write!(f, "Required field missing: {}", field)
            }
            ValidationError::InvalidCharacters {
                input,
                invalid_chars,
            } => {
                write!(f, "Invalid characters in '{}': {:?}", input, invalid_chars)
            }
            ValidationError::InvalidLength {
                input,
                min_length,
                max_length,
            } => {
                write!(
                    f,
                    "Invalid length for '{}', must be between {} and {} characters",
                    input, min_length, max_length
                )
            }
        }
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataError::JsonParse(msg) => write!(f, "JSON parsing failed: {}", msg),
            DataError::NetworkFailed(msg) => write!(f, "Network request failed: {}", msg),
            DataError::IoFailed(msg) => write!(f, "I/O operation failed: {}", msg),
            DataError::UnsupportedFormat(format) => {
                write!(f, "Unsupported data format: {}", format)
            }
            DataError::IntegrityCheckFailed(msg) => {
                write!(f, "Data integrity check failed: {}", msg)
            }
        }
    }
}

impl StdError for BlockpediaError {}
impl StdError for BlockError {}
impl StdError for PropertyError {}
impl StdError for StateError {}
impl StdError for QueryError {}
impl StdError for FetcherError {}
impl StdError for ValidationError {}
impl StdError for DataError {}

// Convenience constructors for common error patterns
impl BlockpediaError {
    pub fn block_not_found(id: &str) -> Self {
        BlockpediaError::Block(BlockError::NotFound(id.to_string()))
    }

    pub fn invalid_block_id(id: &str) -> Self {
        BlockpediaError::Block(BlockError::InvalidId(id.to_string()))
    }

    pub fn property_not_found(block_id: &str, property: &str) -> Self {
        BlockpediaError::Property(PropertyError::NotFound {
            block_id: block_id.to_string(),
            property: property.to_string(),
        })
    }

    pub fn invalid_property_value(
        block_id: &str,
        property: &str,
        value: &str,
        valid_values: Vec<String>,
    ) -> Self {
        BlockpediaError::Property(PropertyError::InvalidValue {
            block_id: block_id.to_string(),
            property: property.to_string(),
            value: value.to_string(),
            valid_values,
        })
    }

    pub fn parse_failed(input: &str, reason: &str) -> Self {
        BlockpediaError::State(StateError::ParseFailed {
            input: input.to_string(),
            reason: reason.to_string(),
        })
    }

    pub fn invalid_format(input: &str, expected: &str) -> Self {
        BlockpediaError::Validation(ValidationError::InvalidFormat {
            input: input.to_string(),
            expected_format: expected.to_string(),
        })
    }

    pub fn custom(message: String) -> Self {
        BlockpediaError::Data(DataError::JsonParse(message))
    }
}

/// Error recovery utilities
pub mod recovery {

    /// Attempt to recover from a block not found error by suggesting similar blocks
    pub fn suggest_similar_blocks(block_id: &str) -> Vec<String> {
        // In a real implementation, this would use fuzzy matching
        // For now, return some basic suggestions
        let mut suggestions = Vec::new();

        if block_id.starts_with("minecraft:") {
            // Already namespaced, suggest removing prefix for common blocks
            if let Some(name) = block_id.strip_prefix("minecraft:") {
                if !name.is_empty() {
                    suggestions.push(format!("Did you mean '{}'?", name));
                }
            }
        } else {
            // Not namespaced, suggest adding minecraft prefix
            suggestions.push(format!("minecraft:{}", block_id));
        }

        suggestions
    }

    /// Attempt to recover from property value errors by suggesting valid values
    pub fn suggest_property_values(
        _property: &str,
        invalid_value: &str,
        valid_values: &[String],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();

        // Find values that are similar to the invalid one
        for valid in valid_values {
            if valid.to_lowercase().contains(&invalid_value.to_lowercase())
                || invalid_value.to_lowercase().contains(&valid.to_lowercase())
            {
                suggestions.push(valid.clone());
            }
        }

        // If no similar values found, suggest a few common ones
        if suggestions.is_empty() && !valid_values.is_empty() {
            suggestions.extend(valid_values.iter().take(3).cloned());
        }

        suggestions
    }

    /// Attempt to fix common parsing errors
    pub fn fix_common_parse_errors(input: &str) -> String {
        let mut fixed = input.to_string();

        // Fix missing brackets
        if input.contains('=') && !input.contains('[') && !input.contains(']') {
            if let Some(colon_pos) = input.find(':') {
                if let Some(equals_pos) = input.find('=') {
                    if equals_pos > colon_pos {
                        let (block_part, _props_part) = input.split_at(equals_pos);
                        // Find the last valid block ID character
                        if let Some(space_pos) = block_part.rfind(' ') {
                            let (prefix, block_id) = block_part.split_at(space_pos + 1);
                            let properties = &input[equals_pos..];
                            fixed = format!(
                                "{}{}[{}{}]",
                                prefix,
                                block_id,
                                properties.chars().next().unwrap_or('='),
                                &properties[1..]
                            );
                        }
                    }
                }
            }
        }

        // Fix double colons
        fixed = fixed.replace("::", ":");

        // Fix spaces around equals
        fixed = fixed.replace(" = ", "=");

        fixed
    }
}

/// Validation utilities
pub mod validation {
    use super::*;

    /// Validate block ID format
    pub fn validate_block_id(id: &str) -> Result<()> {
        if id.is_empty() {
            return Err(BlockpediaError::invalid_format(id, "non-empty string"));
        }

        if id.len() > 256 {
            return Err(BlockpediaError::Validation(
                ValidationError::InvalidLength {
                    input: id.to_string(),
                    min_length: 1,
                    max_length: 256,
                },
            ));
        }

        // Check for valid namespace format
        if let Some(colon_pos) = id.find(':') {
            let namespace = &id[..colon_pos];
            let name = &id[colon_pos + 1..];

            if namespace.is_empty() || name.is_empty() {
                return Err(BlockpediaError::invalid_format(id, "namespace:name"));
            }

            // Validate namespace characters
            if !namespace
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(BlockpediaError::Validation(
                    ValidationError::InvalidCharacters {
                        input: namespace.to_string(),
                        invalid_chars: namespace
                            .chars()
                            .filter(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
                            .collect(),
                    },
                ));
            }

            // Validate name characters
            if !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(BlockpediaError::Validation(
                    ValidationError::InvalidCharacters {
                        input: name.to_string(),
                        invalid_chars: name
                            .chars()
                            .filter(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
                            .collect(),
                    },
                ));
            }
        } else {
            // No namespace, just validate the name
            if !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(BlockpediaError::Validation(
                    ValidationError::InvalidCharacters {
                        input: id.to_string(),
                        invalid_chars: id
                            .chars()
                            .filter(|c| !c.is_ascii_alphanumeric() && *c != '_' && *c != '-')
                            .collect(),
                    },
                ));
            }
        }

        Ok(())
    }

    /// Validate property name format
    pub fn validate_property_name(name: &str) -> Result<()> {
        if name.is_empty() {
            return Err(BlockpediaError::Validation(
                ValidationError::MissingRequired("property name".to_string()),
            ));
        }

        if name.len() > 64 {
            return Err(BlockpediaError::Validation(
                ValidationError::InvalidLength {
                    input: name.to_string(),
                    min_length: 1,
                    max_length: 64,
                },
            ));
        }

        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(BlockpediaError::Validation(
                ValidationError::InvalidCharacters {
                    input: name.to_string(),
                    invalid_chars: name
                        .chars()
                        .filter(|c| !c.is_ascii_alphanumeric() && *c != '_')
                        .collect(),
                },
            ));
        }

        Ok(())
    }

    /// Validate property value format
    pub fn validate_property_value(value: &str) -> Result<()> {
        if value.is_empty() {
            return Err(BlockpediaError::Validation(
                ValidationError::MissingRequired("property value".to_string()),
            ));
        }

        if value.len() > 32 {
            return Err(BlockpediaError::Validation(
                ValidationError::InvalidLength {
                    input: value.to_string(),
                    min_length: 1,
                    max_length: 32,
                },
            ));
        }

        // Property values can contain more characters than names
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
        {
            return Err(BlockpediaError::Validation(
                ValidationError::InvalidCharacters {
                    input: value.to_string(),
                    invalid_chars: value
                        .chars()
                        .filter(|c| {
                            !c.is_ascii_alphanumeric() && *c != '_' && *c != '-' && *c != '.'
                        })
                        .collect(),
                },
            ));
        }

        Ok(())
    }
}
