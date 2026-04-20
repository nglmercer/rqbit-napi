import { RqbitSessionEmitter } from "../emitter";
import { join } from "path";
import { tmpdir } from "os";

async function main() {
  const downloadPath = join(tmpdir(), "rqbit-emitter-downloads");
  console.log(`Initialising emitter session in ${downloadPath}...`);

  const session = await RqbitSessionEmitter.create(downloadPath, { disableDht: false }, 1000);

  session.on("start", (id, stats) => {
    console.log(`[Event: start] Torrent ${id} started. Initial name: ${stats?.name || 'unknown'}`);
  });

  session.on("progress", (id, stats, percentage) => {
    console.log(`[Event: progress] Torrent ${id} (${stats.name}) - ${percentage.toFixed(2)}% (Down: ${stats.downloadSpeed.toFixed(2)} MiB/s)`);
  });

  session.on("done", (id, stats) => {
    console.log(`[Event: done] Torrent ${id} finished downloading!`, stats);
  });

  session.on("error", (id, error) => {
    console.error(`[Event: error] Torrent ${id} encountered an error:`, error);
  });

  // Ubuntu 22.04.1 Desktop ISO magnet with trackers
  const magnet = "magnet:?xt=urn:btih:cab507494d02ebb1178b38f2e9d7be299c86b862&dn=ubuntu-22.04.1-desktop-amd64.iso&tr=https%3A%2F%2Ftorrent.ubuntu.com%3A443%2Fannounce";

  console.log("Adding torrent...");
  const id = await session.addTorrent(magnet, { overwrite: true });
  console.log(`Torrent added with id: ${id}`);
  // Let it run for 10 seconds to see progress events
  setTimeout(async () => {
    console.log("Stopping session...");
    await session.stop();
    console.log("Done.");
    process.exit(0);
  }, 10000);
}

main().catch(console.error);
