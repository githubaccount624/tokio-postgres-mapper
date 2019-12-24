//! # tokio-postgres-mapper
//!
//! `tokio-postgres-mapper` is a proc-macro designed to make mapping from postgresql
//! tables to structs simple.
//!
//! ### Why?
//!
//! It can be frustrating to write a lot of boilerplate and, ultimately, duplicated
//! code for mapping from postgres Rows into structs.
//!
//! For example, this might be what someone would normally write:
//!
//! ```rust
//! use postgres::row::Row;
//!
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//!     pub email: Option<String>,
//! }
//!
//! impl From<Row> for User {
//!     fn from(row: Row) -> Self {
//!         Self {
//!             id: row.get("id"),
//!             name: row.get("name"),
//!             email: row.get("email"),
//!         }
//!     }
//! }
//!
//! // code to execute a query here and get back a row
//! let user = User::from(row); // this can panic
//! ```
//!
//! This becomes worse when manually implementating using the non-panicking
//! `get_opt` method variant.
//!
//! Using this crate, the boilerplate is removed, and panicking and non-panicking
//! implementations are derived:
//!
//! ```rust
//! #[macro_use] extern crate tokio_postgres_mapper_derive;
//! use tokio_postgres_mapper;
//!
//! use tokio_postgres_mapper::FromPostgresRow;
//!
//! #[derive(PostgresMapper)]
//! pub struct User {
//!     pub id: i64,
//!     pub name: String,
//!     pub email: Option<String>,
//! }
//!
//! // code to execute a query here and get back a row
//!
//! // `tokio_postgres_mapper::FromPostgresRow`'s methods do not panic and return a Result
//! let user = User::from(row)?;
//! ```
//!
//! ### The two crates
//!
//! This repository contains two crates: `postgres-mapper` which contains an `Error`
//! enum and traits for converting from a `postgres` or `tokio-postgres` `Row`
//! without panicking, and `postgres-mapper-derive` which contains the proc-macro.
//!
//! `postgres-mapper-derive` has 3 features that can be enabled (where T is the
//! struct being derived with the provided `PostgresMapper` proc-macro):
//!
//! `impl From<::tokio_postgres::row::Row> for T` and
//! `impl From<&::tokio_postgres::row::Row> for T` implementations
//! - `postgres-mapper` which, for each of the above features, implements
//! `postgres-mapper`'s `FromPostgresRow` and/or `FromTokioPostgresRow` traits
//!
//!
//! This will derive implementations for converting from owned and referenced
//! `tokio-postgres::row::Row`s, as well as implementing `postgres-mapper`'s
//! `FromTokioPostgresRow` trait for non-panicking conversions.
use tokio_postgres;

use tokio_postgres::row::Row as TokioRow;

use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Trait containing various methods for converting from a `tokio-postgres` Row
/// to a mapped type.
///
/// When using the `postgres_mapper_derive` crate's `PostgresMapper` proc-macro,
/// this will automatically be implemented on types.
///
/// The [`from_tokio_postgres_row`] method exists for consuming a `Row` - useful
/// for iterator mapping - while [`from_postgres_row_ref`] exists for borrowing
/// a `Row`.
pub trait FromTokioPostgresRow: Sized {
    /// Converts from a `tokio-postgres` `Row` into a mapped type, consuming the
    /// given `Row`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ColumnNotFound`] if the column in a mapping was not
    /// found.
    ///
    /// Returns [`Error::Conversion`] if there was an error converting the row
    /// column to the requested type.
    ///
    /// [`Error::ColumnNotFound`]: enum.Error.html#variant.ColumnNotFound
    /// [`Error::Conversion`]: enum.Error.html#variant.Conversion
    fn from_tokio_postgres_row(row: TokioRow) -> Result<Self, Error>;

    /// Converts from a `tokio-postgres` `Row` into a mapped type, borrowing the
    /// given `Row`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::ColumnNotFound`] if the column in a mapping was not
    /// found.
    ///
    /// Returns [`Error::Conversion`] if there was an error converting the row
    /// column into the requested type.
    ///
    /// [`Error::ColumnNotFound`]: enum.Error.html#variant.ColumnNotFound
    /// [`Error::Conversion`]: enum.Error.html#variant.Conversion
    fn from_tokio_postgres_row_ref(row: &TokioRow) -> Result<Self, Error>;
}

/// General error type returned throughout the library.
#[derive(Debug)]
pub enum Error {
    /// A column in a row was not found.
    ColumnNotFound,
    TokioPostgresError,
    /// An error from the `tokio-postgres` crate while converting a type.
    Conversion(Box<dyn StdError + Send + Sync>),
}

impl From<Box<dyn StdError + Send + Sync>> for Error {
    fn from(err: Box<dyn StdError + Send + Sync>) -> Self {
        Error::Conversion(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ColumnNotFound => "Column in row not found",
            Error::TokioPostgresError => "Tokio Postgres Error",
            Error::Conversion(ref inner) => inner.description(),
        }
    }
}

impl From<tokio_postgres::error::Error> for Error {
    fn from(_err: tokio_postgres::error::Error) -> Self {
        Error::TokioPostgresError
    }
}