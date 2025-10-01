# 🪝 Catchhook

[![CI](https://github.com/abimaelmartell/catchhook/actions/workflows/ci.yml/badge.svg)](https://github.com/abimaelmartell/catchhook/actions/workflows/ci.yml)

A lightweight, self-hosted webhook testing tool that captures and displays HTTP requests in real-time.

## Features

- **Capture All HTTP Methods** - GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- **Real-time Inspection** - View headers, body (JSON/text/binary), and raw request data
- **Auto-refresh** - Updates every 5 seconds + refetch on window focus
- **Persistent Storage** - Requests are saved to disk with configurable limits
- **Modern UI** - Clean, dark-themed interface with method badges and time tracking
- **Zero Dependencies** - Self-contained binary with embedded static assets

## Quick Start

```bash
# Build and run
cargo run --release

# Server starts at http://localhost:43999
# Send requests to http://localhost:43999/webhook
```

## Configuration

Environment variables:

- `CATCHHOOK_PORT` - Server port (default: `43999`)
- `CATCHHOOK_DATA` - Data directory path (default: `./catchhook-data`)
- `CATCHHOOK_MAX_REQS` - Maximum stored requests (default: `10000`)

Example:

```bash
CATCHHOOK_PORT=8080 CATCHHOOK_DATA=/tmp/webhooks cargo run --release
```

## API Endpoints

- `GET /` - Web UI
- `GET /health` - Health check
- `ANY /webhook` - Webhook endpoint (accepts all HTTP methods)
- `GET /latest` - Get latest requests (JSON)
- `GET /req/{id}` - Get specific request by ID (JSON)

## Usage

1. Start the server
2. Open http://localhost:43999 in your browser
3. Copy the webhook URL
4. Send test requests:

```bash
curl -X POST http://localhost:43999/webhook \
  -H "Content-Type: application/json" \
  -d '{"test": "data"}'
```

5. View captured requests in the web UI

## Tech Stack

- **Backend**: Rust with Axum web framework
- **Storage**: File-based with redb (embedded database)
- **Frontend**: Vanilla JavaScript with modern CSS

## License

MIT
