## Snippet: Ensure output directory exists.

```rust
if let Err(e) = std::fs::create_dir_all("output") {
```