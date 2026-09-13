## Snippet: Archive_Files

```rust
let fmt = format.to_lowercase();

            //-NOTE: Ensure output directory exists.
            if let Err(e) = std::fs::create_dir_all("output") {
```