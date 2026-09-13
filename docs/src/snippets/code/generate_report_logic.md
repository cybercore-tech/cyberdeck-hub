## Snippet: Generate_Report_Logic

```rust
let mut s = state.lock().await;
            s.reports_generated += 1;
            // Extract the value into a local variable first (this is an immutable copy)
            let current_report_num = s.reports_generated;
            // Now pass that local variable to the formatter.
            // s is now free to be borrowed mutably by log_event.

            s.execution_log.push(format!("[{}] System Report #{} generated.", timestamp, current_report_num));
```