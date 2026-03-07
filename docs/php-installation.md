# PHP Installation Guide

## Overview

The pivot-pdf PHP extension consists of two parts:

1. **Composer package** (`pivot-pdf/pivot-pdf`) — provides IDE autocompletion stubs
2. **Native extension binary** (`.so` / `.dylib` / `.dll`) — the actual PDF generation engine

The Composer package is installed the usual way. The extension binary is a native library
built from Rust and must be installed separately.

---

## Step 1: Install the Composer Package

```bash
composer require pivot-pdf/pivot-pdf
```

This installs `pdf-php/pdf-php.stubs.php` into your project's vendor directory and adds it
to Composer's autoload. IDEs such as PhpStorm and VS Code (Intelephense) will automatically
pick up the type hints and autocompletion for all pivot-pdf classes.

---

## Step 2: Download the Extension Binary

Visit the [GitHub Releases page](https://github.com/pivotpdftools/pivot-pdf/releases/latest) and
download the binary that matches your platform and PHP version.

**File naming convention:**

| Platform | File                                                 |
|----------|------------------------------------------------------|
| Linux    | `pdf_php-linux-php<PHP_MAJOR_MINOR>.so` |
| macOS    | `pdf_php-macos-php<PHP_MAJOR_MINOR>.dylib` |
| Windows  | `pdf_php-windows-php<PHP_MAJOR_MINOR>.dll` |

**Examples:**
- `pdf_php-linux-php83.so` — Linux, PHP 8.3
- `pdf_php-macos-php82.dylib` — macOS, PHP 8.2

Check your PHP version:
```bash
php --version
```

---

## Step 3: Find Your Extension Directory

PHP has a designated directory for native extensions. Find yours with:

```bash
php -r "echo ini_get('extension_dir') . PHP_EOL;"
```

Example output: `/usr/lib/php/20230831`

---

## Step 4: Place the Binary

Copy the downloaded binary into the directory from Step 3:

```bash
cp pdf_php-linux-php83.so /usr/lib/php/20230831/pdf_php.so
```

---

## Step 5: Enable the Extension in php.ini

Find your active php.ini:
```bash
php --ini
```

Because the file is in your `extension_dir`, you only need the filename — no full path required:

**Linux:**
```ini
extension=pdf_php.so
```

**macOS:**
```ini
extension=pdf_php.dylib
```

**Windows:**
```ini
extension=pdf_php.dll
```

If you're using PHP-FPM or Apache, restart the service after editing php.ini:
```bash
# PHP-FPM
sudo systemctl restart php8.3-fpm

# Apache
sudo systemctl restart apache2
```

---

## Step 6: Verify the Installation

Run a quick check to confirm the extension is loaded:

```bash
php -r "new PdfDocument(); echo 'pivot-pdf extension loaded' . PHP_EOL;"
```

Or check the extension list:
```bash
php -m | grep pdf_php
```

---

## Building from Source

If no pre-built binary exists for your platform, you can compile the extension yourself.

**Requirements:**
- Rust stable toolchain — install from [rustup.rs](https://rustup.rs)
- PHP development headers:
  ```bash
  sudo apt install php-dev        # Debian/Ubuntu
  sudo dnf install php-devel      # Fedora/RHEL
  brew install php                 # macOS (Homebrew includes dev headers)
  ```
- Clang development libraries:
  ```bash
  sudo apt install libclang-dev   # Debian/Ubuntu
  sudo dnf install clang-devel    # Fedora/RHEL
  ```

**Build:**
```bash
git clone https://github.com/PivotPDF/pivot-pdf.git
cd pivot-pdf
cargo build --release -p pdf-php
```

The compiled extension is at `target/release/libpdf_php.so`.

---

## Using the Extension Without Composer

If you prefer not to use Composer, you can add the stubs file to your project manually:

1. Copy `pdf-php/pdf-php.stubs.php` from the repository into your project
2. Configure your IDE to include the file as a stubs source

The stubs file is optional — it only affects IDE autocompletion and has no runtime effect.

---

## Troubleshooting

**"Class PdfDocument not found"**
The extension binary is not loaded. Check that the `extension=` path in php.ini is correct
and that you are running PHP with the right php.ini. Run `php --ini` to see which file is
active.

**Extension loads but crashes**
The binary may have been built for a different PHP version. Verify the PHP version in the
filename matches the output of `php --version`, including major and minor version (e.g. 8.3).

**Permission denied on the .so file**
Ensure the extension file is readable by the PHP process user:
```bash
chmod 644 /usr/local/lib/php/extensions/libpdf_php.so
```

---

## Support

- GitHub Issues: [https://github.com/PivotPDF/pivot-pdf/issues](https://github.com/PivotPDF/pivot-pdf/issues)
