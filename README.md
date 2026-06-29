# pagespeed

CLI tool for Google PageSpeed Insights — generates full Lighthouse-level reports as JSON, organized per run into individual folders.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/zamkara/pagespeed/main/install.sh | sh
```

Supports Linux (x86\_64, ARM64, ARMv7), macOS (Intel & Apple Silicon), and Windows (x86\_64).

> To install to a custom directory:
> ```sh
> INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/zamkara/pagespeed/main/install.sh | sh
> ```

---

## Prerequisites

Requires a Google PageSpeed Insights API key:

1. Open [Google Cloud Console](https://console.cloud.google.com/)
2. Enable the **PageSpeed Insights API**
3. Create an API key under **APIs & Services → Credentials**

The API key can be set via the `PAGESPEED_API_KEY` environment variable. If not set, the tool will prompt for it on first run and save it automatically to your shell config.

---

## Usage

```sh
pagespeed <domain>
```

If `PAGESPEED_API_KEY` is not set, the tool will prompt for it and save it to `~/.bashrc`, `~/.zshrc`, or `~/.config/fish/config.fish` depending on your shell.

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `<domain>` | — | URL or domain to analyze |
| `-s, --strategy` | `mobile` | Strategy: `mobile` or `desktop` |
| `-c, --categories` | `performance` | Categories: `performance`, `accessibility`, `best-practices`, `seo` (comma-separated) |
| `-k, --key` | env `PAGESPEED_API_KEY` | Google API key |
| `-u, --update` | — | Update pagespeed to the latest release |

### Examples

```sh
# Basic analysis
pagespeed example.com

# Desktop strategy with all categories
pagespeed example.com --strategy desktop --categories performance,accessibility,best-practices,seo

# Pass API key directly
pagespeed example.com --key AIza...

# Update to latest version
pagespeed -u
```

---

## Output

Each run produces a folder named:

```
<domain>-<strategy>-<YYYYMMDD>-<HHMMSS>/
├── report.json     # Full Lighthouse data — all audits, metrics, entities, etc.
└── summary.txt     # Human-readable summary: scores, key metrics, issues
```

Example:
```
cv.zamkara.uk-mobile-20260629-143200/
├── report.json
└── summary.txt
```

### `report.json` structure

```json
{
  "url": "https://example.com",
  "strategy": "mobile",
  "fetch_time": "2026-06-29T14:32:00Z",
  "scores": {
    "performance": 97
  },
  "categories": { ... },
  "audits": {
    "first-contentful-paint": {
      "title": "First Contentful Paint",
      "displayValue": "1.7 s",
      "score": 0.91,
      "numericValue": 1726,
      "details": { ... }
    }
  },
  "environment": { ... },
  "entities": [ ... ],
  "stack_packs": [ ... ]
}
```

---

## Update

```sh
pagespeed -u
```

Detects the latest release from GitHub, downloads the matching binary for your OS and architecture, and replaces the running binary in place.

---

## Build from source

```sh
git clone https://github.com/zamkara/pagespeed.git
cd pagespeed
cargo build --release
```

Or use the build script for a specific target:

```sh
./build-prod.sh                              # native target
./build-prod.sh aarch64-unknown-linux-musl   # cross-compile
```

---

## License

MIT
