use std::future;

use futures_util::{TryStreamExt, StreamExt};
use tiberius::QueryStream;

/// converts each row in the stream according to the given mapper, then returns
/// the transformed rows
pub async fn map<T, F>(stream: QueryStream<'_>, mapper: F) -> Vec<T>
where 
    F: FnMut(tiberius::Row) -> T 
{
    stream
        .into_row_stream() // skip metadata
        .into_stream()
        .filter_map(|row| future::ready(row.ok())) // skip non-OK rows
        .map(mapper)
        .collect()
        .await
}