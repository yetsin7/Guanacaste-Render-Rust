//! A tiny data-parallel helper for the post-processing passes.

/// Runs `f(row_index, row)` for every `row_len`-sized row of `data`, spread over every available core.
pub fn for_each_row<F>(data: &mut [f32], row_len: usize, f: F)
where
    F: Fn(usize, &mut [f32]) + Sync,
{
    if row_len == 0 || data.is_empty() {
        return;
    }
    let rows = data.len() / row_len;
    let workers = std::thread::available_parallelism().map_or(1, |n| n.get());
    let per_worker = rows.div_ceil(workers).max(1);
    std::thread::scope(|scope| {
        for (chunk_index, chunk) in data.chunks_mut(per_worker * row_len).enumerate() {
            let f = &f;
            scope.spawn(move || {
                for (i, row) in chunk.chunks_mut(row_len).enumerate() {
                    f(chunk_index * per_worker + i, row);
                }
            });
        }
    });
}
