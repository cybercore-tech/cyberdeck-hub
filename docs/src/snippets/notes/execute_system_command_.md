## Snippet: Execute system command.

```rust
let result = match fmt.as_str() {
                "zip" => Command::new("zip").args(["-r", &output_path, "./diagnostics"]).status(),
                "tar" => Command::new("tar").args(["-cvf", &output_path, "./diagnostics"]).status(),
                "gzip" => Command::new("tar").args(["-czvf", &output_path, "./diagnostics"]).status(),
                "7z" => Command::new("7z").args(["a", &output_path, "./diagnostics"]).status(),
                _ => {
                    tracing::error!("Unsupported format requested: {}", fmt);
                    s.execution_log.push(format!("[{}] Error: Unsupported format '{}'", timestamp, fmt));
                    return;
                }
            };
```