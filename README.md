# M3u8 proxy written in rust

A fast, no-caching proxy server for `.m3u8` HLS playlists and segments, built using Rust and Actix-Web.

It rewrites `.m3u8` files so that all segment requests (like `.ts`, `.vtt`, etc.) go through the same proxy — enabling CORS and header manipulation.

- Streams `.m3u8`, `.ts`, `.vtt`, etc.
- Supports custom headers via `&headers=...`
- Supports custom `Origin` via `&origin=...`
- Handles CORS automatically
- Fast: uses keep-alive connection pooling

---

## Getting Started

### 1. Clone the repository

```bash
git clone https://github.com/zuhaz/rust-proxy.git
cd rust-proxy
```

### 2. (If needed) Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

## Running the Server

### Option 1: Native (via Cargo)

```bash
cargo run
```

The server will start at:

```
http://127.0.0.1:8080
```

To change the port or allowed origins, edit `main.rs`.

---

### Option 2: Docker

Pull the latest image:

```bash
docker pull ghcr.io/zuhaz/rustproxy:latest
```

Run it:

```bash
docker run -p 8080:8080 ghcr.io/zuhaz/rustproxy:latest
```

---

## API Usage

### Proxy a direct file or media segment

```
GET /?url=https://example.com/file.ts
```

### Proxy a .m3u8 playlist and rewrite internal URLs

```
GET /?url=https://example.com/playlist.m3u8
```

### Proxy with headers (JSON string, URL encoded if needed)

```
GET /?url=https://example.com/playlist.m3u8&headers={"Referer":"https://example.com"}
```

### Proxy with origin (string)

```
GET /?url=https://example.com/playlist.m3u8&origin=https://example.com
```

---

## Configuration

The server will automatically load environment settings from the included `.env` file. No additional setup is required.

### To change CORS behavior:

Edit `ENABLE_CORS` in the `.env` file:

```env
ENABLE_CORS=true
```

Or disable it:

```env
ENABLE_CORS=false
```

### To add a new allowed origin

You need to update the following section in `main.rs`:

```rust
static ALLOWED_ORIGINS: Lazy<[&str; N]> = Lazy::new(|| [
    "http://localhost:5173",
    "http://localhost:3000",
    "http://your-new-origin.com", // <-- add here
]);
```

Both changes are required for proper CORS behavior when `ENABLE_CORS=true`.

## LICENSE

Using: [Apache License 2.0](LICENSE)


## Credits

Inspired by: https://github.com/Gratenes/m3u8CloudflareWorkerProxy