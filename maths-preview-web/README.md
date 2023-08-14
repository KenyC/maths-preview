# Maths Preview for the web

Type formulas, see them rendered in real-time in your browser, save the result as an image.

See a hosted version [here](https://maths-preview.netlify.app/).

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

After building, start a server in folder `www/`, e.g.

```bash
python -m http.server
```

Open your browser at [http://0.0.0.0:8000/](http://0.0.0.0:8000/)


