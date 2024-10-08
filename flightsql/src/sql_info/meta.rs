//! SQL metadata tables (originally from [queryrouterd])
//!
//! TODO: figure out how to generate these keywords automatically from DataFusion / sqlparser-rs
//!
//! [queryrouterd]: https://github.com/influxdata/idpe/blob/85aa7a52b40f173cc4d79ac02b3a4a13e82333c4/queryrouter/internal/server/flightsql_info.go#L4

use arrow::{
    compute::can_cast_types,
    datatypes::{
        DataType,
        IntervalUnit::{DayTime, YearMonth},
        TimeUnit::Nanosecond,
    },
};
use arrow_flight::sql::SqlSupportsConvert;
use std::{collections::HashMap, sync::LazyLock};

pub(crate) const SQL_INFO_SQL_KEYWORDS: &[&str] = &[
    // SQL-92 Reserved Words
    "absolute",
    "action",
    "add",
    "all",
    "allocate",
    "alter",
    "and",
    "any",
    "are",
    "as",
    "asc",
    "assertion",
    "at",
    "authorization",
    "avg",
    "begin",
    "between",
    "bit",
    "bit_length",
    "both",
    "by",
    "cascade",
    "cascaded",
    "case",
    "cast",
    "catalog",
    "char",
    "char_length",
    "character",
    "character_length",
    "check",
    "close",
    "coalesce",
    "collate",
    "collation",
    "column",
    "commit",
    "connect",
    "connection",
    "constraint",
    "constraints",
    "continue",
    "convert",
    "corresponding",
    "count",
    "create",
    "cross",
    "current",
    "current_date",
    "current_time",
    "current_timestamp",
    "current_user",
    "cursor",
    "date",
    "day",
    "deallocate",
    "dec",
    "decimal",
    "declare",
    "default",
    "deferrable",
    "deferred",
    "delete",
    "desc",
    "describe",
    "descriptor",
    "diagnostics",
    "disconnect",
    "distinct",
    "domain",
    "double",
    "drop",
    "else",
    "end",
    "end-exec",
    "escape",
    "except",
    "exception",
    "exec",
    "execute",
    "exists",
    "external",
    "extract",
    "false",
    "fetch",
    "first",
    "float",
    "for",
    "foreign",
    "found",
    "from",
    "full",
    "get",
    "global",
    "go",
    "goto",
    "grant",
    "group",
    "having",
    "hour",
    "identity",
    "immediate",
    "in",
    "indicator",
    "initially",
    "inner",
    "input",
    "insensitive",
    "insert",
    "int",
    "integer",
    "intersect",
    "interval",
    "into",
    "is",
    "isolation",
    "join",
    "key",
    "language",
    "last",
    "leading",
    "left",
    "level",
    "like",
    "local",
    "lower",
    "match",
    "max",
    "min",
    "minute",
    "module",
    "month",
    "names",
    "national",
    "natural",
    "nchar",
    "next",
    "no",
    "not",
    "null",
    "nullif",
    "numeric",
    "octet_length",
    "of",
    "on",
    "only",
    "open",
    "option",
    "or",
    "order",
    "outer",
    "output",
    "overlaps",
    "pad",
    "partial",
    "position",
    "precision",
    "prepare",
    "preserve",
    "primary",
    "prior",
    "privileges",
    "procedure",
    "public",
    "read",
    "real",
    "references",
    "relative",
    "restrict",
    "revoke",
    "right",
    "rollback",
    "rows",
    "schema",
    "scroll",
    "second",
    "section",
    "select",
    "session",
    "session_user",
    "set",
    "size",
    "smallint",
    "some",
    "space",
    "sql",
    "sqlcode",
    "sqlerror",
    "sqlstate",
    "substring",
    "sum",
    "system_user",
    "table",
    "temporary",
    "then",
    "time",
    "timestamp",
    "timezone_hour",
    "timezone_minute",
    "to",
    "trailing",
    "transaction",
    "translate",
    "translation",
    "trim",
    "true",
    "union",
    "unique",
    "unknown",
    "update",
    "upper",
    "usage",
    "user",
    "using",
    "value",
    "values",
    "varchar",
    "varying",
    "view",
    "when",
    "whenever",
    "where",
    "with",
    "work",
    "write",
    "year",
    "zone",
];

pub(crate) const SQL_INFO_NUMERIC_FUNCTIONS: &[&str] = &[
    "abs", "acos", "asin", "atan", "atan2", "ceil", "cos", "exp", "floor", "ln", "log", "log10",
    "log2", "pow", "power", "round", "signum", "sin", "sqrt", "tan", "trunc",
];

pub(crate) const SQL_INFO_STRING_FUNCTIONS: &[&str] = &[
    "arrow_typeof",
    "ascii",
    "bit_length",
    "btrim",
    "char_length",
    "character_length",
    "chr",
    "concat",
    "concat_ws",
    "digest",
    "from_unixtime",
    "initcap",
    "left",
    "length",
    "lower",
    "lpad",
    "ltrim",
    "md5",
    "octet_length",
    "random",
    "regexp_match",
    "regexp_replace",
    "repeat",
    "replace",
    "reverse",
    "right",
    "rpad",
    "rtrim",
    "sha224",
    "sha256",
    "sha384",
    "sha512",
    "split_part",
    "starts_with",
    "strpos",
    "substr",
    "to_hex",
    "translate",
    "trim",
    "upper",
    "uuid",
];

pub(crate) const SQL_INFO_DATE_TIME_FUNCTIONS: &[&str] = &[
    "current_date",
    "current_time",
    "date_bin",
    "date_part",
    "date_trunc",
    "datepart",
    "datetrunc",
    "from_unixtime",
    "now",
    "to_timestamp",
    "to_timestamp_micros",
    "to_timestamp_millis",
    "to_timestamp_seconds",
];

pub(crate) const SQL_INFO_SYSTEM_FUNCTIONS: &[&str] = &["array", "arrow_typeof", "struct"];

static SQL_DATA_TYPE_TO_ARROW_DATA_TYPE: LazyLock<HashMap<SqlSupportsConvert, DataType>> =
    LazyLock::new(|| {
        [
            // Referenced from DataFusion data types
            // https://arrow.apache.org/datafusion/user-guide/sql/data_types.html
            // Some SQL types are not supported by DataFusion
            // https://arrow.apache.org/datafusion/user-guide/sql/data_types.html#unsupported-sql-types
            (SqlSupportsConvert::SqlConvertBigint, DataType::Int64),
            // SqlSupportsConvert::SqlConvertBinary is not supported
            (SqlSupportsConvert::SqlConvertBit, DataType::Boolean),
            (SqlSupportsConvert::SqlConvertChar, DataType::Utf8),
            (SqlSupportsConvert::SqlConvertDate, DataType::Date32),
            (
                SqlSupportsConvert::SqlConvertDecimal,
                // Use the max precision 38
                // https://docs.rs/arrow-schema/47.0.0/arrow_schema/constant.DECIMAL128_MAX_PRECISION.html
                DataType::Decimal128(38, 2),
            ),
            (SqlSupportsConvert::SqlConvertFloat, DataType::Float32),
            (SqlSupportsConvert::SqlConvertInteger, DataType::Int32),
            (
                SqlSupportsConvert::SqlConvertIntervalDayTime,
                DataType::Interval(DayTime),
            ),
            (
                SqlSupportsConvert::SqlConvertIntervalYearMonth,
                DataType::Interval(YearMonth),
            ),
            // SqlSupportsConvert::SqlConvertLongvarbinary is not supported
            // LONG VARCHAR is identical to VARCHAR
            // https://docs.oracle.com/javadb/10.6.2.1/ref/rrefsqlj15147.html
            (SqlSupportsConvert::SqlConvertLongvarchar, DataType::Utf8),
            // NUMERIC is a synonym for DECIMAL and behaves the same way
            // https://docs.oracle.com/javadb/10.6.2.1/ref/rrefsqlj12362.html
            (
                SqlSupportsConvert::SqlConvertNumeric,
                // Use the max precision 38
                // https://docs.rs/arrow-schema/47.0.0/arrow_schema/constant.DECIMAL128_MAX_PRECISION.html
                DataType::Decimal128(38, 2),
            ),
            (SqlSupportsConvert::SqlConvertReal, DataType::Float32),
            (SqlSupportsConvert::SqlConvertSmallint, DataType::Int16),
            (
                SqlSupportsConvert::SqlConvertTime,
                DataType::Time64(Nanosecond),
            ),
            (
                SqlSupportsConvert::SqlConvertTimestamp,
                DataType::Timestamp(Nanosecond, None),
            ),
            (SqlSupportsConvert::SqlConvertTinyint, DataType::Int8),
            // SqlSupportsConvert::SqlConvertVarbinary is not supported
            (SqlSupportsConvert::SqlConvertVarchar, DataType::Utf8),
        ]
        .iter()
        .cloned()
        .collect()
    });

pub(crate) static SQL_INFO_SUPPORTS_CONVERT: LazyLock<HashMap<i32, Vec<i32>>> =
    LazyLock::new(|| {
        let mut convert: HashMap<i32, Vec<i32>> = HashMap::new();
        for (from_type_sql, from_type_arrow) in SQL_DATA_TYPE_TO_ARROW_DATA_TYPE.clone().into_iter()
        {
            let mut can_convert_to: Vec<i32> = vec![];
            for (to_type_sql, to_type_arrow) in SQL_DATA_TYPE_TO_ARROW_DATA_TYPE.clone().into_iter()
            {
                if can_cast_types(&from_type_arrow, &to_type_arrow) {
                    can_convert_to.push(to_type_sql as i32)
                }
            }
            if !can_convert_to.is_empty() {
                convert.insert(from_type_sql as i32, can_convert_to);
            }
        }
        convert
    });
