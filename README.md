<div align="center">

# Unescaper

Unescape strings with escape sequences written out as literal characters.

[![License MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![License GPLv3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)
[![Checks](https://github.com/hack-ink/unescaper/actions/workflows/checks.yml/badge.svg?branch=main)](https://github.com/hack-ink/unescaper/actions/workflows/checks.yml)
[![Release](https://github.com/hack-ink/unescaper/actions/workflows/release.yml/badge.svg)](https://github.com/hack-ink/unescaper/actions/workflows/release.yml)
[![GitHub tag (latest by date)](https://img.shields.io/github/v/tag/hack-ink/unescaper)](https://github.com/hack-ink/unescaper/tags)
[![GitHub last commit](https://img.shields.io/github/last-commit/hack-ink/unescaper?color=red&style=plastic)](https://github.com/hack-ink/unescaper)

</div>

## Usage

```rust
fn main() {
	assert_eq!(unescaper::unescape(r"\u000a").unwrap(), "\n");
	assert_eq!(unescaper::unescape(r"\u{a}").unwrap(), "\n");
	assert_eq!(unescaper::unescape(r"\x0a").unwrap(), "\n");
	assert_eq!(unescaper::unescape(r"\12").unwrap(), "\n");
	assert_eq!(unescaper::unescape_lossy(r"\q\n"), "\\q\n");
}
```

Supported escapes:

| Escape | Output |
| --- | --- |
| `\b`, `\f`, `\n`, `\r`, `\t`, `\v` | ASCII control characters |
| `\'`, `\"`, `\\`, `\/` | Literal quote, backslash, or slash |
| `\u000a`, `\u{a}` | Unicode scalar values |
| `\x0a` | Byte values |
| `\12` | Octal byte values |

`unescape` returns an error for malformed escapes. `unescape_lossy` decodes valid escapes and
preserves malformed escapes as written.

## Support Me

If you find this project helpful and would like to support its development, you can buy me a coffee!

Your support is greatly appreciated and motivates me to keep improving this project.

- **Fiat**
    - [Ko-fi](https://ko-fi.com/hack_ink)
    - [Afdian](https://afdian.com/a/hack_ink)
- **Crypto**
    - **Bitcoin**
        - `bc1pedlrf67ss52md29qqkzr2avma6ghyrt4jx9ecp9457qsl75x247sqcp43c`
    - **Ethereum**
        - `0x3e25247CfF03F99a7D83b28F207112234feE73a6`
    - **Polkadot**
        - `156HGo9setPcU2qhFMVWLkcmtCEGySLwNqa3DaEiYSWtte4Y`

Thank you for your support!

## Appreciation

We would like to extend our heartfelt gratitude to the following projects and contributors:

- [unescape-rs](https://github.com/saghm/unescape-rs) for the original idea this crate builds on.

---

<div align="right">

### License

<sup>Licensed under [MIT](LICENSE-MIT) or [GPL-3.0-only](LICENSE-GPL3).</sup>

</div>
