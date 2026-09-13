## 1. Trigger via Environment Scripts

Launch your automated sequential module scans directly using your true crate binary name:

Bash

```
export Cyberdeck_ENV="init_display; run_module hardware; run_module power; run_module memory"
export CYBERDECK_PORT="8080"

# Execute the actual compiled binary
./target/debug/cyberdeck_core
```

## 2. Hot-Inject Dynamic Commands Live

While `cyberdeck_core` is up and running its Axum server, fire your runtime execution payloads straight into the listener:

Bash

```
# Force a power metrics sweep on the fly
curl -X POST http://127.0.0.1:8080/api/cyberdeck/command \
     -H "Content-Type: application/json" \
     -d '{"RunPowerModule": "./diagnostics"}'
```

## 3. Persistent Systemd Service Configuration

If you want this running as a background utility daemon on your environment, use this exact configuration file.

Drop this into `/etc/systemd/system/cyberdeck.service`:

Ini, TOML

```
[Unit]
Description=Cyberdeck Core Intelligence Engine
After=network.target

[Service]
Type=simple
User=raven
WorkingDirectory=/home/raven/devspace/oms-rust/cyberdeck
Environment=Cyberdeck_ENV="init_display; run_module hardware"
Environment=CYBERDECK_PORT=8888
ExecStart=/home/raven/devspace/oms-rust/cyberdeck/target/release/cyberdeck_core
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

To load and kick it live:

Bash

```
sudo systemctl daemon-reload
sudo systemctl enable --now cyberdeck
```
