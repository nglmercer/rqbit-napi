import { RqbitSession } from "../index";
import { join } from "path";
import { tmpdir } from "os";

const DOWNLOAD_PATH = join(tmpdir(), "rqbit-api-downloads");
const session = await RqbitSession.create(DOWNLOAD_PATH);

console.log(`Rqbit API Server started on http://localhost:3000`);
console.log(`Downloads path: ${DOWNLOAD_PATH}`);

const server = Bun.serve({
  port: 3000,
  async fetch(req) {
    const url = new URL(req.url);
    const path = url.pathname;
    const method = req.method;

    // Simple Logger
    console.log(`${method} ${path}`);

    // GET /stats - Global session stats
    if (path === "/stats" && method === "GET") {
      return Response.json(session.getSessionStats());
    }

    // POST /stop - Graceful shutdown
    if (path === "/stop" && method === "POST") {
      await session.stop();
      return Response.json({ success: true });
    }

    // GET /torrents - List all torrents
    if (path === "/torrents" && method === "GET") {
      const ids = session.listTorrents();
      const torrents = await Promise.all(
        ids.map(async (id) => {
          const stats = await session.getTorrentStats(id);
          return { id, ...stats };
        })
      );
      return Response.json(torrents.filter((t) => t.name !== undefined));
    }

    // POST /torrents - Add a torrent
    if (path === "/torrents" && method === "POST") {
      try {
        const body = await req.json() as { url: string, overwrite?: boolean, outputFolder?: string };
        if (!body.url) return new Response("Missing url", { status: 400 });
        const id = await session.addTorrent(body.url, {
          overwrite: body.overwrite ?? true,
          outputFolder: body.outputFolder
        });
        return Response.json({ id }, { status: 201 });
      } catch (e) {
        return new Response(String(e), { status: 500 });
      }
    }

    // Torrent specific routes: /torrents/:id, /torrents/:id/pause, etc.
    const torrentMatch = path.match(/^\/torrents\/(\d+)(?:\/(pause|resume|delete))?$/);
    if (torrentMatch) {
      const id = parseInt(torrentMatch[1]);
      const action = torrentMatch[2];

      // GET /torrents/:id
      if (!action && method === "GET") {
        const stats = await session.getTorrentStats(id);
        if (!stats) return new Response("Not found", { status: 404 });
        return Response.json(stats);
      }

      // POST /torrents/:id/pause
      if (action === "pause" && method === "POST") {
        try {
          const ok = await session.pauseTorrent(id);
          return Response.json({ success: ok });
        } catch (e) {
          return new Response(String(e), { status: 500 });
        }
      }

      // POST /torrents/:id/resume
      if (action === "resume" && method === "POST") {
        try {
          const ok = await session.startTorrent(id);
          return Response.json({ success: ok });
        } catch (e) {
          return new Response(String(e), { status: 500 });
        }
      }

      // DELETE /torrents/:id or POST /torrents/:id/delete
      if (method === "DELETE" || (action === "delete" && method === "POST")) {
        try {
          const deleteFiles = url.searchParams.get("deleteFiles") === "true";
          const ok = await session.deleteTorrent(id, deleteFiles);
          return Response.json({ success: ok });
        } catch (e) {
          return new Response(String(e), { status: 500 });
        }
      }
    }

    return new Response("Not Found", { status: 404 });
  },
});
export default server;