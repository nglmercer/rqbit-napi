import { RqbitSession } from "../index";
import { join } from "path";
import { tmpdir } from "os";

async function main() {
  const downloadPath = join(tmpdir(), "rqbit-downloads");
  console.log(`Initialising session in ${downloadPath}...`);
  
  const session = await RqbitSession.create(downloadPath);
  
  // Ubuntu 22.04.1 Desktop ISO magnet
  const magnet = "magnet:?xt=urn:btih:cab507494d02ebb1178b38f2e9d7be299c86b862&dn=ubuntu-22.04.1-desktop-amd64.iso";
  console.log("Adding torrent...");
  
  // By default overwrite is true in our wrapper
  const id = await session.addTorrent(magnet);
  console.log(`Torrent added with ID: ${id}`);
  
  // Poll for stats
  console.log("Polling for stats (5 seconds)...");
  for (let i = 0; i < 5; i++) {
    const stats = await session.getTorrentStats(id);
    if (stats) {
      const progress = stats.totalBytes > 0 ? ((stats.downloadedBytes / stats.totalBytes) * 100).toFixed(2) : "0.00";
      console.log(`[${stats.name}] Progress: ${progress}% | Down: ${stats.downloadSpeed.toFixed(2)} MiB/s | Up: ${stats.uploadSpeed.toFixed(2)} MiB/s`);
    }
    await new Promise(r => setTimeout(r, 1000));
  }
  
  console.log("Pausing torrent...");
  await session.pauseTorrent(id);
  
  const torrents = session.listTorrents();
  console.log(`Active torrents IDs: ${torrents.join(", ")}`);
  
  console.log("Done!");
  process.exit(0);
}

main().catch(error => {
  console.error("Error:", error);
  process.exit(1);
});
