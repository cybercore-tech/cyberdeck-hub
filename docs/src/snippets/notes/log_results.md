## Snippet: Log results

```rust
match result {
                Ok(status) if status.success() => {
                    tracing::info!("Archive successful: {}", filename);
                    s.execution_log.push(format!("[{}] Archive created: {}", timestamp, filename));
                }
                _ => {
                    tracing::error!("Compression command failed.");
                    s.execution_log.push(format!("[{}] Critical: Compression failed.", timestamp));
                }
            }
```