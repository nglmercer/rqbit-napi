# rqbit-napi

A high-performance Bun/TypeScript wrapper around the [`librqbit`](https://github.com/ikatson/rqbit) BitTorrent client library using NAPI-RS.

This library allows you to embed a full-featured, extremely fast BitTorrent client directly into your Node.js or Bun applications without needing to run a separate process.

## Features

- **High Performance**: Powered by Rust and `tokio` under the hood.
- **Asynchronous**: Non-blocking API designed for modern JavaScript environments.
- **Comprehensive Control**: Add, pause, resume, and delete torrents programmatically.
- **Real-time Metrics**: Stream download speeds, upload speeds, and progress for individual torrents and the entire session.
- **Graceful Shutdown**: Built-in support for cleaning up resources cleanly.

## Installation

```bash
bun install
bun run build:debug # or build for production: bun run build
```

*(Note: Pre-built binaries can be configured if you publish this package to npm).*

## Quick Start

Here is a simple example showing how to initialize a session and download a torrent:

```typescript
import { RqbitSession } from "rqbit-napi";
import { join } from "path";
import { tmpdir } from "os";

async function main() {
  const downloadPath = join(tmpdir(), "rqbit-downloads");
  
  // 1. Initialize the session
  const session = await RqbitSession.create(downloadPath);
  
  // 2. Add a torrent (Ubuntu ISO example)
  const magnet = "magnet:?xt=urn:btih:cab507494d02ebb1178b38f2e9d7be299c86b862&dn=ubuntu-22.04.1-desktop-amd64.iso";
  const id = await session.addTorrent(magnet);
  
  // 3. Monitor progress
  for (let i = 0; i < 5; i++) {
    const stats = await session.getTorrentStats(id);
    if (stats) {
      const progress = stats.totalBytes > 0 
        ? ((stats.downloadedBytes / stats.totalBytes) * 100).toFixed(2) 
        : "0.00";
      console.log(`[${stats.name}] Progress: ${progress}% | Down: ${stats.downloadSpeed.toFixed(2)} MiB/s`);
    }
    await new Promise(r => setTimeout(r, 1000));
  }
  
  // 4. Clean up
  await session.stop();
}

main();
```

## REST API Example

You can easily wrap `rqbit-napi` in a web server (like Bun's native `Bun.serve`) to control downloads remotely. We provide a full example in `examples/server.ts`.

To run the REST API example:
```bash
bun run server
```

**Available Endpoints:**
- `GET /stats` - Get global session bandwidth and uptime.
- `GET /torrents` - List all active torrents and their stats.
- `POST /torrents` - Add a new torrent (`{"url": "magnet:..."}`).
- `GET /torrents/:id` - Get stats for a specific torrent.
- `POST /torrents/:id/pause` - Pause a torrent.
- `POST /torrents/:id/resume` - Resume a torrent.
- `DELETE /torrents/:id?deleteFiles=true` - Delete a torrent (and optionally its files).
- `POST /stop` - Gracefully shutdown the server.

## Event Emitter API

For event-driven architectures, we provide an `EventEmitter` wrapper (`emitter.ts`) that polls the session automatically and emits real-time events.

```typescript
import { RqbitSessionEmitter } from "rqbit-napi/emitter";

const session = await RqbitSessionEmitter.create("/downloads");

session.on("start", (id, stats) => console.log(`Torrent ${id} started`));
session.on("progress", (id, stats, percentage) => console.log(`${percentage}% done`));
session.on("done", (id, stats) => console.log(`Torrent ${id} completed`));
session.on("error", (id, err) => console.error(err));

await session.addTorrent("magnet:...");
```

## API Reference

### `RqbitSession`

The main class representing a BitTorrent engine session.

#### `static create(downloadPath: string, options?: RqbitSessionOptions): Promise<RqbitSession>`
Creates a new `librqbit` session.
- `downloadPath`: The default directory where files will be downloaded.
- `options`: Optional configuration (e.g., `{ disableDht: true }`).

#### `addTorrent(url: string, options?: RqbitAddTorrentOptions): Promise<number>`
Adds a torrent via a Magnet URI or HTTP URL to a `.torrent` file.
- Returns a unique numeric `id` representing the torrent in this session.

#### `getTorrentStats(index: number): Promise<TorrentStats | null>`
Retrieves real-time statistics for a specific torrent.

#### `listTorrents(): Array<number>`
Returns an array of all active torrent IDs managed by this session.

#### `pauseTorrent(index: number): Promise<boolean>`
Pauses the torrent. Returns `true` if successful.

#### `startTorrent(index: number): Promise<boolean>`
Resumes a paused torrent. Returns `true` if successful.

#### `waitUntilInitialized(index: number): Promise<boolean>`
Waits until the torrent has finished its initial metadata resolution and file checking phase. Useful before attempting to pause a newly added magnet link.

#### `deleteTorrent(index: number, deleteFiles: boolean): Promise<boolean>`
Removes the torrent from the session. If `deleteFiles` is `true`, it will also delete the downloaded data from the disk.

#### `getSessionStats(): RqbitSessionStats`
Returns aggregate statistics for the entire session (total bandwidth, uptime).

#### `stop(): Promise<void>`
Gracefully stops the session, saving state and disconnecting from peers.

---

### Types

#### `RqbitSessionOptions`
```typescript
interface RqbitSessionOptions {
  disableDht?: boolean;
  disableDhtPersistence?: boolean;
}
```

#### `RqbitAddTorrentOptions`
```typescript
interface RqbitAddTorrentOptions {
  outputFolder?: string; // Override the session's default download path
  overwrite?: boolean;   // Allow overwriting existing files (default: true)
}
```

#### `TorrentStats`
```typescript
interface TorrentStats {
  name: string;
  finished: boolean;
  totalBytes: number;
  downloadedBytes: number;
  uploadedBytes: number;
  downloadSpeed: number; // In MiB/s
  uploadSpeed: number;   // In MiB/s
}
```

#### `RqbitSessionStats`
```typescript
interface RqbitSessionStats {
  fetchedBytes: number;
  uploadedBytes: number;
  downloadSpeed: number; // In MiB/s
  uploadSpeed: number;   // In MiB/s
  uptimeSeconds: number;
}
```
