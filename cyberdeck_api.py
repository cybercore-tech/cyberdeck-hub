#!/usr/bin/env python3
"""
cyberdeck_api.py — a zero-dependency HTTP API of local system intelligence.

A standalone, stdlib-only companion to CYBERDECK (the Rust
``cyberdeck_core`` framework): it surfaces the same class of
system-intelligence data — CPU, memory, storage, thermal, power,
network, services, hardware topology — as JSON (or CYBERDECK-style
Markdown) over HTTP, without needing the Rust binary built or running.

Every collector degrades gracefully: a missing tool or unreadable
sysfs node yields ``{"available": false, ...}`` rather than a 500, so
the API works on a bare box and gets richer as tooling is present.

Standard library only. Linux. Python 3.8+.

    python3 cyberdeck_api.py                     # http://127.0.0.1:8990
    python3 cyberdeck_api.py --port 9000
    python3 cyberdeck_api.py --host 0.0.0.0      # expose on the LAN

Endpoints (all GET):

    /                     service metadata + module list
    /health               {"status": "ok"}
    /modules              [{name, description, available}]
    /all                  every module in one document
    /<module>             one module (see /modules for names)

    ?format=json (default) | markdown      on /all and /<module>

Modules: host, cpu, memory, disks, storage_ai, network, thermal, fan,
power, hardware, services, audio
"""

from __future__ import annotations

import argparse
import json
import os
import re
import socket
import struct
import subprocess
import sys
import time
from datetime import datetime, timezone
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlparse

__version__ = "1.0.0"


# --- low-level helpers ----------------------------------------------------


def read_text(path: str) -> "str | None":
    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            return fh.read().strip()
    except OSError:
        return None


def read_int(path: str) -> "int | None":
    v = read_text(path)
    try:
        return int(v) if v is not None else None
    except ValueError:
        return None


def run(*cmd: str, timeout: float = 5.0) -> "str | None":
    try:
        out = subprocess.run(
            cmd, capture_output=True, text=True, timeout=timeout, check=False
        )
    except (OSError, subprocess.SubprocessError):
        return None
    if out.returncode != 0 and not out.stdout:
        return None
    return out.stdout


def run_json(*cmd: str, timeout: float = 5.0):
    raw = run(*cmd, timeout=timeout)
    if not raw:
        return None
    try:
        return json.loads(raw)
    except ValueError:
        return None


def have(tool: str) -> bool:
    return any(
        os.access(os.path.join(p, tool), os.X_OK)
        for p in os.environ.get("PATH", "").split(os.pathsep)
        if p
    )


def glob_dir(path: str) -> "list[str]":
    try:
        return sorted(os.listdir(path))
    except OSError:
        return []


# --- collectors ---------------------------------------------------------
#
# Each returns a JSON-serialisable dict. Raising is fine — the dispatcher
# converts an exception into {"error": ...}. Prefer {"available": false}
# for the ordinary "this box doesn't have that" case.


def collect_host() -> dict:
    u = os.uname()
    osr = {}
    raw = read_text("/etc/os-release") or ""
    for line in raw.splitlines():
        if "=" in line:
            k, _, v = line.partition("=")
            osr[k.strip()] = v.strip().strip('"')

    uptime_s = None
    up = read_text("/proc/uptime")
    if up:
        try:
            uptime_s = float(up.split()[0])
        except (ValueError, IndexError):
            pass

    loadavg = read_text("/proc/loadavg")
    load = loadavg.split()[:3] if loadavg else []

    return {
        "hostname": socket.gethostname(),
        "distro": osr.get("PRETTY_NAME") or osr.get("NAME"),
        "kernel": u.release,
        "arch": u.machine,
        "os": u.sysname,
        "uptime_seconds": round(uptime_s) if uptime_s is not None else None,
        "uptime_human": _dhms(uptime_s) if uptime_s is not None else None,
        "boot_time_utc": (
            datetime.fromtimestamp(time.time() - uptime_s, timezone.utc).isoformat()
            if uptime_s is not None
            else None
        ),
        "load_average": {"1m": _f(load, 0), "5m": _f(load, 1), "15m": _f(load, 2)},
        "cpu_count": os.cpu_count(),
    }


def collect_cpu() -> dict:
    model = None
    physical_ids = set()
    logical = 0
    cpuinfo = read_text("/proc/cpuinfo") or ""
    for block in cpuinfo.split("\n\n"):
        if not block.strip():
            continue
        logical += 1
        for line in block.splitlines():
            key, _, val = line.partition(":")
            key, val = key.strip(), val.strip()
            if key == "model name" and model is None:
                model = val
            elif key == "physical id":
                physical_ids.add(val)

    governors = sorted(
        {
            read_text(
                f"/sys/devices/system/cpu/{c}/cpufreq/scaling_governor"
            )
            for c in glob_dir("/sys/devices/system/cpu")
            if re.fullmatch(r"cpu\d+", c)
        }
        - {None}
    )

    freqs_khz = [
        read_int(f"/sys/devices/system/cpu/{c}/cpufreq/scaling_cur_freq")
        for c in sorted(glob_dir("/sys/devices/system/cpu"))
        if re.fullmatch(r"cpu\d+", c)
    ]
    freqs_mhz = [round(f / 1000, 1) for f in freqs_khz if f]

    load = (read_text("/proc/loadavg") or "").split()[:3]

    return {
        "model": model,
        "sockets": len(physical_ids) or 1,
        "logical_cpus": logical or os.cpu_count(),
        "governors": governors,
        "cur_freq_mhz": {
            "min": min(freqs_mhz) if freqs_mhz else None,
            "max": max(freqs_mhz) if freqs_mhz else None,
            "per_cpu": freqs_mhz or None,
        },
        "load_average": {"1m": _f(load, 0), "5m": _f(load, 1), "15m": _f(load, 2)},
        "utilisation_percent": _cpu_util_percent(),
    }


def collect_memory() -> dict:
    info = {}
    raw = read_text("/proc/meminfo") or ""
    for line in raw.splitlines():
        key, _, val = line.partition(":")
        parts = val.split()
        if parts:
            try:
                info[key.strip()] = int(parts[0])  # kB
            except ValueError:
                pass

    def kb(name):
        return info.get(name)

    total = kb("MemTotal")
    avail = kb("MemAvailable")
    used = (total - avail) if (total and avail is not None) else None
    swap_total = kb("SwapTotal")
    swap_free = kb("SwapFree")

    return {
        "unit": "kB",
        "total": total,
        "available": avail,
        "used": used,
        "free": kb("MemFree"),
        "buffers": kb("Buffers"),
        "cached": kb("Cached"),
        "used_percent": round(100 * used / total, 1) if (used and total) else None,
        "swap": {
            "total": swap_total,
            "free": swap_free,
            "used": (swap_total - swap_free)
            if (swap_total and swap_free is not None)
            else None,
            "used_percent": (
                round(100 * (swap_total - swap_free) / swap_total, 1)
                if swap_total
                else None
            ),
        },
    }


_FS_SKIP = {
    "proc", "sysfs", "devtmpfs", "devpts", "tmpfs", "cgroup", "cgroup2",
    "pstore", "bpf", "securityfs", "debugfs", "tracefs", "mqueue",
    "hugetlbfs", "configfs", "fusectl", "efivarfs", "autofs", "binfmt_misc",
    "ramfs", "nsfs", "squashfs", "overlay",
}


def collect_disks() -> dict:
    mounts = []
    raw = read_text("/proc/mounts") or ""
    seen = set()
    for line in raw.splitlines():
        parts = line.split()
        if len(parts) < 3:
            continue
        dev, mnt, fstype = parts[0], parts[1].replace("\\040", " "), parts[2]
        if fstype in _FS_SKIP or not dev.startswith("/dev/"):
            continue
        if mnt in seen:
            continue
        seen.add(mnt)
        entry = {"device": dev, "mountpoint": mnt, "fstype": fstype}
        try:
            st = os.statvfs(mnt)
            size = st.f_blocks * st.f_frsize
            free = st.f_bfree * st.f_frsize
            avail = st.f_bavail * st.f_frsize
            used = size - free
            entry.update(
                size_bytes=size,
                used_bytes=used,
                avail_bytes=avail,
                used_percent=round(100 * used / size, 1) if size else None,
            )
        except OSError as exc:
            entry["error"] = str(exc)
        mounts.append(entry)

    block = []
    for name in glob_dir("/sys/block"):
        if name.startswith(("loop", "ram", "zram")):
            continue
        base = f"/sys/block/{name}"
        sectors = read_int(f"{base}/size")
        block.append(
            {
                "name": name,
                "size_bytes": sectors * 512 if sectors else None,
                "rotational": bool(read_int(f"{base}/queue/rotational")),
                "model": read_text(f"{base}/device/model"),
                "removable": bool(read_int(f"{base}/removable")),
            }
        )

    return {"filesystems": mounts, "block_devices": block}


def collect_storage_ai() -> dict:
    if not have("smartctl"):
        return {"available": False, "reason": "smartctl not installed"}
    disks = []
    for name in glob_dir("/sys/block"):
        if name.startswith(("loop", "ram", "zram", "dm-", "md")):
            continue
        dev = f"/dev/{name}"
        data = run_json("smartctl", "-H", "-A", "-j", dev, timeout=8)
        if not data:
            disks.append({"device": dev, "available": False})
            continue
        health = (data.get("smart_status") or {}).get("passed")
        temp = (data.get("temperature") or {}).get("current")
        poh = (data.get("power_on_time") or {}).get("hours")
        disks.append(
            {
                "device": dev,
                "model": data.get("model_name"),
                "serial": data.get("serial_number"),
                "smart_passed": health,
                "temperature_c": temp,
                "power_on_hours": poh,
                "rotation_rate": data.get("rotation_rate"),
            }
        )
    return {"available": True, "disks": disks}


def collect_network() -> dict:
    ifaces = []
    for name in glob_dir("/sys/class/net"):
        base = f"/sys/class/net/{name}"
        stats = f"{base}/statistics"
        ifaces.append(
            {
                "name": name,
                "state": read_text(f"{base}/operstate"),
                "mac": read_text(f"{base}/address"),
                "mtu": read_int(f"{base}/mtu"),
                "speed_mbps": read_int(f"{base}/speed"),
                "rx_bytes": read_int(f"{stats}/rx_bytes"),
                "tx_bytes": read_int(f"{stats}/tx_bytes"),
                "is_loopback": name == "lo",
            }
        )

    addrs = {}
    ipj = run_json("ip", "-j", "addr") if have("ip") else None
    if ipj:
        for item in ipj:
            addrs[item.get("ifname")] = [
                {
                    "family": a.get("family"),
                    "address": a.get("local"),
                    "prefix": a.get("prefixlen"),
                }
                for a in item.get("addr_info", [])
            ]
    for i in ifaces:
        i["addresses"] = addrs.get(i["name"], [])

    routes = run_json("ip", "-j", "route") if have("ip") else None
    default_gw = None
    if routes:
        for r in routes:
            if r.get("dst") == "default":
                default_gw = r.get("gateway")
                break

    return {
        "interfaces": ifaces,
        "default_gateway": default_gw,
        "listening_tcp_ports": _listening_tcp_ports(),
    }


def collect_thermal() -> dict:
    zones = []
    for z in glob_dir("/sys/class/thermal"):
        if not z.startswith("thermal_zone"):
            continue
        base = f"/sys/class/thermal/{z}"
        milli = read_int(f"{base}/temp")
        zones.append(
            {
                "zone": z,
                "type": read_text(f"{base}/type"),
                "temp_c": round(milli / 1000, 1) if milli is not None else None,
            }
        )

    hwmon = []
    for h in glob_dir("/sys/class/hwmon"):
        base = f"/sys/class/hwmon/{h}"
        chip = read_text(f"{base}/name")
        for f in glob_dir(base):
            m = re.fullmatch(r"temp(\d+)_input", f)
            if not m:
                continue
            milli = read_int(f"{base}/{f}")
            hwmon.append(
                {
                    "chip": chip,
                    "label": read_text(f"{base}/temp{m.group(1)}_label")
                    or f"temp{m.group(1)}",
                    "temp_c": round(milli / 1000, 1) if milli is not None else None,
                }
            )

    temps = [z["temp_c"] for z in zones if z["temp_c"]] + [
        h["temp_c"] for h in hwmon if h["temp_c"]
    ]
    return {
        "thermal_zones": zones,
        "hwmon_sensors": hwmon,
        "max_temp_c": max(temps) if temps else None,
    }


def collect_fan() -> dict:
    fans = []
    for h in glob_dir("/sys/class/hwmon"):
        base = f"/sys/class/hwmon/{h}"
        chip = read_text(f"{base}/name")
        for f in glob_dir(base):
            m = re.fullmatch(r"fan(\d+)_input", f)
            if not m:
                continue
            fans.append(
                {
                    "chip": chip,
                    "label": read_text(f"{base}/fan{m.group(1)}_label")
                    or f"fan{m.group(1)}",
                    "rpm": read_int(f"{base}/{f}"),
                }
            )
    return {"available": bool(fans), "fans": fans}


def collect_power() -> dict:
    supplies = []
    for name in glob_dir("/sys/class/power_supply"):
        base = f"/sys/class/power_supply/{name}"
        kind = read_text(f"{base}/type")
        entry = {"name": name, "type": kind}
        if kind == "Mains":
            entry["online"] = bool(read_int(f"{base}/online"))
        else:
            cap = read_int(f"{base}/capacity")
            energy_now = read_int(f"{base}/energy_now") or read_int(f"{base}/charge_now")
            energy_full = read_int(f"{base}/energy_full") or read_int(
                f"{base}/charge_full"
            )
            entry.update(
                status=read_text(f"{base}/status"),
                capacity_percent=cap,
                health=read_text(f"{base}/health"),
                power_now_uw=read_int(f"{base}/power_now"),
                voltage_now_uv=read_int(f"{base}/voltage_now"),
                energy_now=energy_now,
                energy_full=energy_full,
                wear_percent=(
                    round(100 * (1 - energy_full / read_int(f"{base}/energy_full_design")), 1)
                    if read_int(f"{base}/energy_full_design")
                    else None
                ),
            )
        supplies.append(entry)
    return {"available": bool(supplies), "supplies": supplies}


_CHASSIS = {
    "3": "Desktop", "4": "Low Profile Desktop", "5": "Pizza Box",
    "6": "Mini Tower", "7": "Tower", "8": "Portable", "9": "Laptop",
    "10": "Notebook", "11": "Hand Held", "13": "All In One",
    "14": "Sub Notebook", "30": "Tablet", "31": "Convertible",
    "32": "Detachable",
}


def collect_hardware() -> dict:
    d = "/sys/class/dmi/id"
    ct = read_text(f"{d}/chassis_type")
    return {
        "system": {
            "vendor": read_text(f"{d}/sys_vendor"),
            "product": read_text(f"{d}/product_name"),
            "family": read_text(f"{d}/product_family"),
            "version": read_text(f"{d}/product_version"),
        },
        "baseboard": {
            "vendor": read_text(f"{d}/board_vendor"),
            "name": read_text(f"{d}/board_name"),
            "version": read_text(f"{d}/board_version"),
        },
        "bios": {
            "vendor": read_text(f"{d}/bios_vendor"),
            "version": read_text(f"{d}/bios_version"),
            "date": read_text(f"{d}/bios_date"),
            "ec_firmware": read_text(f"{d}/ec_firmware_release"),
        },
        "chassis": {"type_code": ct, "type": _CHASSIS.get(ct or "", "Unknown")},
    }


def collect_services() -> dict:
    if not have("systemctl"):
        return {"available": False, "reason": "systemctl not installed"}
    running = run(
        "systemctl", "list-units", "--type=service", "--state=running",
        "--no-legend", "--plain", "--no-pager",
    )
    names = []
    if running:
        for line in running.splitlines():
            parts = line.split()
            if parts:
                names.append(parts[0])
    failed = run(
        "systemctl", "list-units", "--type=service", "--state=failed",
        "--no-legend", "--plain", "--no-pager",
    )
    failed_names = [l.split()[0] for l in (failed or "").splitlines() if l.split()]
    state = (run("systemctl", "is-system-running") or "").strip() or None
    return {
        "available": True,
        "system_state": state,
        "running_count": len(names),
        "running": names,
        "failed_count": len(failed_names),
        "failed": failed_names,
    }


def collect_audio() -> dict:
    if not have("pactl"):
        return {"available": False, "reason": "pactl not installed"}
    default_sink = (run("pactl", "get-default-sink") or "").strip() or None
    default_source = (run("pactl", "get-default-source") or "").strip() or None

    def short(kind):
        out = run("pactl", "list", "short", kind) or ""
        rows = []
        for line in out.splitlines():
            cols = line.split("\t")
            if len(cols) >= 2:
                rows.append({"id": cols[0], "name": cols[1]})
        return rows

    return {
        "available": True,
        "server": (run("pactl", "info") or "").splitlines()[0].split(": ", 1)[-1]
        if run("pactl", "info")
        else None,
        "default_sink": default_sink,
        "default_source": default_source,
        "sinks": short("sinks"),
        "sources": short("sources"),
    }


MODULES = {
    "host": (collect_host, "Hostname, distro, kernel, uptime, load"),
    "cpu": (collect_cpu, "Model, core counts, governor, per-core MHz, utilisation"),
    "memory": (collect_memory, "RAM and swap totals / usage from /proc/meminfo"),
    "disks": (collect_disks, "Mounted filesystems and block devices"),
    "storage_ai": (collect_storage_ai, "SMART health / temperature (smartctl)"),
    "network": (collect_network, "Interfaces, addresses, gateway, listening ports"),
    "thermal": (collect_thermal, "Thermal zones and hwmon temperature sensors"),
    "fan": (collect_fan, "Fan RPM from hwmon"),
    "power": (collect_power, "Battery and AC power supplies"),
    "hardware": (collect_hardware, "DMI: system, baseboard, BIOS, chassis"),
    "services": (collect_services, "systemd running / failed units"),
    "audio": (collect_audio, "PulseAudio/PipeWire sinks and sources (pactl)"),
}

# cheap availability probes for /modules
_AVAIL = {
    "storage_ai": lambda: have("smartctl"),
    "services": lambda: have("systemctl"),
    "audio": lambda: have("pactl"),
    "fan": lambda: bool(glob_dir("/sys/class/hwmon")),
    "power": lambda: bool(glob_dir("/sys/class/power_supply")),
}


# --- small format helpers ----------------------------------------------


def _f(seq, idx):
    try:
        return float(seq[idx])
    except (ValueError, IndexError, TypeError):
        return None


def _dhms(seconds: float) -> str:
    s = int(seconds)
    d, s = divmod(s, 86400)
    h, s = divmod(s, 3600)
    m, s = divmod(s, 60)
    out = []
    if d:
        out.append(f"{d}d")
    if h or d:
        out.append(f"{h}h")
    out.append(f"{m}m")
    return " ".join(out)


def _cpu_util_percent(interval: float = 0.12):
    def snap():
        line = read_text("/proc/stat") or ""
        for row in line.splitlines():
            if row.startswith("cpu "):
                vals = [int(x) for x in row.split()[1:]]
                idle = vals[3] + (vals[4] if len(vals) > 4 else 0)
                return sum(vals), idle
        return None

    a = snap()
    if not a:
        return None
    time.sleep(interval)
    b = snap()
    if not b:
        return None
    dt, di = b[0] - a[0], b[1] - a[1]
    if dt <= 0:
        return None
    return round(100 * (1 - di / dt), 1)


def _listening_tcp_ports():
    ports = set()
    for proto in ("/proc/net/tcp", "/proc/net/tcp6"):
        raw = read_text(proto)
        if not raw:
            continue
        for line in raw.splitlines()[1:]:
            parts = line.split()
            if len(parts) < 4 or parts[3] != "0A":  # 0A = LISTEN
                continue
            local = parts[1]
            try:
                port = int(local.split(":")[1], 16)
                ports.add(port)
            except (ValueError, IndexError):
                pass
    return sorted(ports)


# --- Markdown rendering (CYBERDECK-style hierarchical output) ----------


def to_markdown(title: str, data, level: int = 1) -> str:
    lines: "list[str]" = [f"{'#' * level} {title}", ""]
    _md_value(data, lines, level + 1)
    return "\n".join(lines).rstrip() + "\n"


def _md_scalar(v):
    if v is None:
        return "—"
    if isinstance(v, bool):
        return "yes" if v else "no"
    return str(v)


def _md_value(data, lines, level):
    if isinstance(data, dict):
        for key, val in data.items():
            if isinstance(val, dict):
                lines.append(f"{'#' * min(level, 6)} {key}")
                lines.append("")
                _md_value(val, lines, level + 1)
                lines.append("")
            elif isinstance(val, list) and val and isinstance(val[0], dict):
                lines.append(f"{'#' * min(level, 6)} {key}")
                lines.append("")
                _md_table(val, lines)
                lines.append("")
            elif isinstance(val, list):
                lines.append(f"- **{key}**: " + ", ".join(_md_scalar(x) for x in val))
            else:
                lines.append(f"- **{key}**: {_md_scalar(val)}")
    elif isinstance(data, list) and data and isinstance(data[0], dict):
        _md_table(data, lines)
    elif isinstance(data, list):
        for x in data:
            lines.append(f"- {_md_scalar(x)}")
    else:
        lines.append(_md_scalar(data))


def _md_table(rows, lines):
    cols = []
    for r in rows:
        for k in r:
            if k not in cols:
                cols.append(k)
    lines.append("| " + " | ".join(cols) + " |")
    lines.append("| " + " | ".join("---" for _ in cols) + " |")
    for r in rows:
        lines.append(
            "| " + " | ".join(_md_scalar(r.get(c)) for c in cols) + " |"
        )


# --- HTTP -------------------------------------------------------------


def collect(name: str) -> dict:
    fn = MODULES[name][0]
    try:
        return fn()
    except Exception as exc:  # noqa: BLE001 - collectors must never 500
        return {"error": f"{type(exc).__name__}: {exc}"}


class Handler(BaseHTTPRequestHandler):
    server_version = f"cyberdeck_api/{__version__}"

    def _send(self, status, obj=None, raw=None, content_type="application/json; charset=utf-8"):
        if raw is None:
            raw = json.dumps(obj, indent=2, ensure_ascii=False) + "\n"
        body = raw.encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, OPTIONS")
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)

    def _error(self, status, message, **extra):
        payload = {"error": message}
        payload.update(extra)
        self._send(status, payload)

    def log_message(self, fmt, *args):
        sys.stderr.write("%s  %s\n" % (self.log_date_time_string(), fmt % args))

    def do_OPTIONS(self):  # noqa: N802
        self.send_response(HTTPStatus.NO_CONTENT)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "*")
        self.end_headers()

    def do_HEAD(self):  # noqa: N802
        self.do_GET()

    def do_GET(self):  # noqa: N802
        parsed = urlparse(self.path)
        parts = [p for p in parsed.path.split("/") if p]
        query = parse_qs(parsed.query)
        fmt = query.get("format", ["json"])[-1].lower()
        if fmt not in ("json", "markdown", "md"):
            return self._error(
                HTTPStatus.BAD_REQUEST, f"unknown format {fmt!r}; use json or markdown"
            )
        want_md = fmt in ("markdown", "md")

        if not parts:
            return self._send(HTTPStatus.OK, self._index())
        if parts == ["health"]:
            return self._send(HTTPStatus.OK, {"status": "ok"})
        if parts == ["modules"]:
            return self._send(HTTPStatus.OK, self._modules())
        if parts == ["all"]:
            report = {name: collect(name) for name in MODULES}
            if want_md:
                doc = [f"# CYBERDECK SYSTEM REPORT", ""]
                doc.append(f"_generated {datetime.now(timezone.utc).isoformat()}_")
                doc.append("")
                for name, data in report.items():
                    doc.append(to_markdown(name, data, level=2))
                return self._send(
                    HTTPStatus.OK, raw="\n".join(doc), content_type="text/markdown; charset=utf-8"
                )
            return self._send(
                HTTPStatus.OK,
                {
                    "generated_utc": datetime.now(timezone.utc).isoformat(),
                    "modules": report,
                },
            )
        if len(parts) == 1 and parts[0] in MODULES:
            data = collect(parts[0])
            if want_md:
                return self._send(
                    HTTPStatus.OK,
                    raw=to_markdown(parts[0], data, level=1),
                    content_type="text/markdown; charset=utf-8",
                )
            return self._send(HTTPStatus.OK, data)

        return self._error(
            HTTPStatus.NOT_FOUND,
            f"no such endpoint: /{'/'.join(parts)}",
            modules=sorted(MODULES),
            endpoints=["/", "/health", "/modules", "/all", "/<module>"],
        )

    def do_POST(self):  # noqa: N802
        self._error(
            HTTPStatus.METHOD_NOT_ALLOWED,
            f"{self.command} not allowed; this API is read-only (GET)",
        )

    do_PUT = do_DELETE = do_PATCH = do_POST

    def _index(self):
        return {
            "service": "cyberdeck-api",
            "description": "Zero-dependency HTTP view of local system intelligence",
            "version": __version__,
            "host": socket.gethostname(),
            "endpoints": ["/", "/health", "/modules", "/all", "/<module>"],
            "query": {"format": ["json", "markdown"]},
            "modules": sorted(MODULES),
        }

    def _modules(self):
        out = []
        for name, (_, desc) in MODULES.items():
            probe = _AVAIL.get(name)
            out.append(
                {
                    "name": name,
                    "description": desc,
                    "available": bool(probe()) if probe else True,
                }
            )
        return out


def main(argv=None):
    ap = argparse.ArgumentParser(
        prog="cyberdeck_api.py",
        description="Zero-dependency HTTP API of local system intelligence.",
    )
    ap.add_argument("--host", default="127.0.0.1", help="bind address (default: 127.0.0.1)")
    ap.add_argument("--port", type=int, default=8990, help="bind port (default: 8990)")
    ap.add_argument("--version", action="store_true", help="print version and exit")
    args = ap.parse_args(argv)

    if args.version:
        print(f"cyberdeck_api {__version__}")
        return 0

    httpd = ThreadingHTTPServer((args.host, args.port), Handler)
    print(
        f"cyberdeck-api {__version__} listening on http://{args.host}:{args.port}\n"
        f"  modules : {', '.join(sorted(MODULES))}\n"
        f"  try     : curl -s http://{args.host}:{args.port}/all?format=markdown",
        file=sys.stderr,
    )
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\ncyberdeck-api: shutting down", file=sys.stderr)
    finally:
        httpd.server_close()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
