## Snippet: Generate_date-stamped_filename

```rust
let now = chrono::Local::now();
            let date_str = now.format("%Y-%m-%d").to_string();

            let file_ext = if fmt == "gzip" { "tar.gz" } else { &fmt };
            let filename = format!("cyberdeck_archive_{}.{}", date_str, file_ext);
            let output_path = format!("output/{}", filename);

            let mut s = state.lock().await;
            tracing::info!("Archiving to: {}", output_path);
```