# Maths Preview for the web

## Building

Install the prerequisites:

```bash
cargo install wasm-bindgen-cli
rustup target add wasm32-unknown-unknown
```

Build it:

```bash
source build.sh
```

## Running locally

Start a server in folder `www/`, e.g.

```bash
python -m http.server
```

Open your browser at [http://0.0.0.0:8000/](http://0.0.0.0:8000/)


