# pagespeed

CLI tool untuk Google PageSpeed Insights — menghasilkan laporan lengkap level Lighthouse dalam format JSON, tersimpan rapi per folder per test.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/zamkara/pagespeed/main/install.sh | sh
```

Mendukung Linux (x86\_64, ARM64, ARMv7), macOS (Intel & Apple Silicon), dan Windows (x86\_64, ARM64).

> Untuk install ke direktori custom:
> ```sh
> INSTALL_DIR=~/.local/bin curl -fsSL https://raw.githubusercontent.com/zamkara/pagespeed/main/install.sh | sh
> ```

---

## Prasyarat

Butuh API key dari Google PageSpeed Insights:

1. Buka [Google Cloud Console](https://console.cloud.google.com/)
2. Aktifkan **PageSpeed Insights API**
3. Buat API key di **APIs & Services → Credentials**

API key bisa di-set via environment variable atau diinput interaktif saat pertama kali dijalankan.

---

## Penggunaan

```sh
pagespeed <domain>
```

Jika `PAGESPEED_API_KEY` belum di-set, tool akan meminta input dan menyimpannya otomatis ke shell config (`~/.bashrc`, `~/.zshrc`, atau `~/.config/fish/config.fish`).

### Opsi

| Flag | Default | Keterangan |
|------|---------|------------|
| `<domain>` | — | URL atau domain yang dianalisis |
| `-s, --strategy` | `mobile` | Strategy: `mobile` atau `desktop` |
| `-c, --categories` | `performance` | Kategori: `performance`, `accessibility`, `best-practices`, `seo` (bisa kombinasi, pisah koma) |
| `-k, --key` | env `PAGESPEED_API_KEY` | Google API key |
| `-u, --update` | — | Update pagespeed ke versi terbaru |

### Contoh

```sh
# analisis dasar
pagespeed example.com

# desktop + semua kategori
pagespeed example.com --strategy desktop --categories performance,accessibility,best-practices,seo

# gunakan API key langsung
pagespeed example.com --key AIza...

# update ke versi terbaru
pagespeed -u
```

---

## Output

Setiap test menghasilkan folder dengan format:

```
<domain>-<strategy>-<YYYYMMDD>-<HHMMSS>/
├── report.json     # data Lighthouse lengkap (semua audit, metrics, entities, dll)
└── summary.txt     # ringkasan: skor, metrik utama, daftar issue
```

Contoh:
```
cv.zamkara.uk-mobile-20260629-143200/
├── report.json
└── summary.txt
```

### Struktur `report.json`

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
    },
    ...
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

Mendeteksi versi terbaru dari GitHub Releases, mengunduh binary yang sesuai dengan arsitektur sistem, dan mengganti binary yang sedang berjalan secara otomatis.

---

## Build dari source

```sh
git clone https://github.com/zamkara/pagespeed.git
cd pagespeed
cargo build --release
```

Atau gunakan script untuk build semua target:

```sh
./build-prod.sh                          # native
./build-prod.sh aarch64-unknown-linux-gnu  # cross
```

---

## License

MIT
