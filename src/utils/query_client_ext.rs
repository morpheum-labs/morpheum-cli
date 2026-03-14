//! Extension trait for all query clients from `morpheum-sdk-native`.
//!
//! This is the **canonical** way every `query/*.rs` module executes queries.
//! It provides clean, consistent, and DRY methods that:
//! - Execute the async query
//! - Automatically map `SdkError` → `CliError` and `io::Error` → `CliError`
//! - Render results using the shared `Output` handler (Table or JSON)
//! - Handle `Option<T>` (not-found) and `(Vec<T>, u32)` (paginated) patterns
//!
//! Zero boilerplate in query modules. Pure extension trait pattern.

use crate::error::CliError;
use crate::output::Output;
use morpheum_sdk_native::{MorpheumClient, SdkError};
use serde::Serialize;
use std::future::Future;
use tabled::Tabled;

/// Extension trait for any Morpheum SDK query client.
///
/// Provides four ergonomic methods covering every query return-type pattern:
/// - `query_and_print_item`      → `Result<T, SdkError>`
/// - `query_and_print_list`      → `Result<Vec<T>, SdkError>`
/// - `query_and_print_optional`  → `Result<Option<T>, SdkError>`
/// - `query_and_print_paginated` → `Result<(Vec<T>, u32), SdkError>`
pub trait QueryClientExt: MorpheumClient + Send + Sync {
    /// Single-item query → render.
    async fn query_and_print_item<T, F, Fut>(
        &self,
        output: &Output,
        query_fn: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: Future<Output = Result<T, SdkError>> + Send,
        T: Tabled + Serialize;

    /// List query → render.
    async fn query_and_print_list<T, F, Fut>(
        &self,
        output: &Output,
        query_fn: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: Future<Output = Result<Vec<T>, SdkError>> + Send,
        T: Tabled + Serialize;

    /// Optional-item query → render item or warn if `None`.
    async fn query_and_print_optional<T, F, Fut>(
        &self,
        output: &Output,
        not_found_msg: &str,
        query_fn: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: Future<Output = Result<Option<T>, SdkError>> + Send,
        T: Tabled + Serialize;

    /// Paginated list query → print total count + render items.
    async fn query_and_print_paginated<T, F, Fut>(
        &self,
        output: &Output,
        query_fn: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: Future<Output = Result<(Vec<T>, u32), SdkError>> + Send,
        T: Tabled + Serialize;
}

impl<C> QueryClientExt for C
where
    C: MorpheumClient + Send + Sync,
{
    async fn query_and_print_item<T, F, Fut>(
        &self,
        output: &Output,
        query_fn: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: Future<Output = Result<T, SdkError>> + Send,
        T: Tabled + Serialize,
    {
        let item = query_fn(self).await?;
        output.print_item(&item)?;
        Ok(())
    }

    async fn query_and_print_list<T, F, Fut>(
        &self,
        output: &Output,
        query_fn: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: Future<Output = Result<Vec<T>, SdkError>> + Send,
        T: Tabled + Serialize,
    {
        let items = query_fn(self).await?;
        output.print_list(&items)?;
        Ok(())
    }

    async fn query_and_print_optional<T, F, Fut>(
        &self,
        output: &Output,
        not_found_msg: &str,
        query_fn: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: Future<Output = Result<Option<T>, SdkError>> + Send,
        T: Tabled + Serialize,
    {
        match query_fn(self).await? {
            Some(item) => output.print_item(&item)?,
            None => output.warn(not_found_msg),
        }
        Ok(())
    }

    async fn query_and_print_paginated<T, F, Fut>(
        &self,
        output: &Output,
        query_fn: F,
    ) -> Result<(), CliError>
    where
        F: FnOnce(&Self) -> Fut,
        Fut: Future<Output = Result<(Vec<T>, u32), SdkError>> + Send,
        T: Tabled + Serialize,
    {
        let (items, total) = query_fn(self).await?;
        output.info(format!("Total: {total}"));
        output.print_list(&items)?;
        Ok(())
    }
}
