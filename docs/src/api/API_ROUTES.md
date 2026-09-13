# Local API Specifications

<div class="note">
    **Note:** This is the core API.
</div>

The Cyberdeck exposes a high-performance local network interface on `127.0.0.1:4400` to manipulate states or append commands mid-lifecycle.

## 1. Fetch System Diagnostics

Returns a complete serialization snapshot of the hardware configurations, running modules, and execution sequence histories.

* **Endpoint:** `/api/cyberdeck/state`
* **Method:** `GET`
* **Headers:** `Accept: application/json`

### Example Response Payload (`200 OK`)

```json
{
  "display_active": true,
  "active_modules": ["HardwareAudit"],
  "stealth_mode": true,
  "reports_generated": 1,
  "execution_log": [
    "[1719715740] Display system activated.",
    "[1719715740] Dispatched Module Execution: HardwareAudit"
  ]
}
