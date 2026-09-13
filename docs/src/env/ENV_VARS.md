### Updated System Environment Variable Registry

| **Variable Name**           | **Type**        | **Example Value**                  | **Target System Block**           |
| --------------------------- | --------------- | ---------------------------------- | --------------------------------- |
| `CYBERDECK_ENV`             | String (Script) | `init_display; run_module network` | Initialization Language Engine    |
| `CYBERDECK_PORT`            | Integer         | `3000`                             | Local Network API Router          |
| `CYBERDECK_API_BIND`        | String (IP)     | `127.0.0.1`                        | Network Interface Exposure        |
| `CYBERDECK_SECRET_KEY`      | String (Crypto) | `df83j28f0s...`                    | API Access Authentication         |
| `CYBERDECK_LOG_LEVEL`       | String          | `info`                             | Rust Internal Tracing Driver      |
| `CYBERDECK_CPU_GOVERNOR`    | String          | `performance`                      | Low-level Hardware Scaling        |
| `CYBERDECK_DISPLAY_BACKEND` | String          | `wayland`                          | Core UI Window Manager            |
| `CYBERDECK_NET_INTERFACE`   | String          | `wlan0`                            | Diagnostics Hardware Layer        |
| `CYBERDECK_DNS_OVERRIDE`    | String (IP)     | `1.1.1.1`                          | Custom Resolver Core              |
| `CYBERDECK_STORAGE_ROOT`    | String (Path)   | `/var/lib/cyberdeck/`              | Persistent Database Engine        |
| `CYBERDECK_TMP_CHROOT`      | String (Path)   | `/tmp/cyberdeck_sb/`               | Module Sandbox Subshell Isolation |

## Operations & API Interaction

### Testing Native Module Startup

To pass script loops that explicitly invoke specific system routing modules through the environment string:

Bash

```
Cyberdeck_ENV="init_display; stealth_on; run_module hardware; run_module storage; generate_report" cargo run
```

### Dynamic Module Routing over API

While the core program is listening, you can bypass manual shell triggers and spin up additional system checking modules directly via runtime injections:

Bash

```
curl -X POST http://127.0.0.1:3000/api/cyberdeck/command \
  -H "Content-Type: application/json" \
  -d '{"type": "RunModule", "args": "NetworkDiagnostics"}'
```
