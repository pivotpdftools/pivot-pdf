
## Run the examples
```bash
cargo run --example generate_sample
```

```bash
cargo run --example generate_textflow
```

## PHP Extension

### Prerequisites
```bash
sudo apt install php-dev libclang-dev
```

### Build
```bash
cargo build --release -p pdf-php
```

### Run tests
```bash
php -d extension=target/release/libpdf_php.so pdf-php/tests/test.php
```