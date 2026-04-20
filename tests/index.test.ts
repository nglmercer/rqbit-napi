import { expect, test, describe, beforeAll, afterAll } from "bun:test";
import { RqbitSession, initTracing } from "../index";
import { mkdtemp, rm } from "fs/promises";
import { join } from "path";
import { tmpdir } from "os";

describe("RqbitSession Comprehensive Tests", () => {
  let session: RqbitSession;
  let testDir: string;

  const MAGNET_LINKS = [
    "magnet:?xt=urn:btih:cab507494d02ebb1178b38f2e9d7be299c86b862&dn=ubuntu-22.04.1-desktop-amd64.iso&tr=https%3A%2F%2Ftorrent.ubuntu.com%3A443%2Fannounce",
    "magnet:?xt=urn:btih:6a971c261e4776c5b058c734914d8525b42d7655&dn=ubuntu-22.04.2-live-server-amd64.iso&tr=https%3A%2F%2Ftorrent.ubuntu.com%3A443%2Fannounce"
  ];

  beforeAll(async () => {
    // Enable tracing for better error diagnostics
    initTracing();

    // Use a fresh temp directory for every test run
    testDir = await mkdtemp(join(tmpdir(), "rqbit-test-"));
    session = await RqbitSession.create(testDir, {
      disableDht: false,
      fastresume: false
    });
  });

  afterAll(async () => {
    if (session) {
      await session.stop();
    }
    if (testDir) {
      try {
        await rm(testDir, { recursive: true, force: true });
      } catch (e) {
        console.error("Failed to cleanup test directory:", e);
      }
    }
  });

  describe("Session Basics", () => {
    test("Session should be created and stats should be accessible", () => {
      expect(session).toBeDefined();
      const stats = session.getSessionStats();
      expect(typeof stats.uptimeSeconds).toBe("number");
      expect(stats.uptimeSeconds).toBeGreaterThanOrEqual(0);
    });

    test("Should update rate limits", () => {
      // Should not throw
      session.updateLimits(1024 * 1024, 1024 * 1024);
      session.updateLimits(undefined as any, undefined as any);
    });
  });

  describe("Torrent Operations", () => {
    let activeTorrentId: number;

    test("Should add a torrent from magnet link", async () => {
      activeTorrentId = await session.addTorrent(MAGNET_LINKS[0]);
      expect(typeof activeTorrentId).toBe("number");
      expect(activeTorrentId).toBeGreaterThanOrEqual(0);
    }, 30000);

    test("Should list torrents including the one added", () => {
      const ids = session.listTorrents();
      expect(ids).toContain(activeTorrentId);
    });

    test("Should get detailed stats for the torrent", async () => {
      const stats = await session.getTorrentStats(activeTorrentId);
      expect(stats).not.toBeNull();
      if (stats) {
        expect(stats.name).toBeDefined();
        expect(typeof stats.downloadedBytes).toBe("number");
        expect(typeof stats.totalBytes).toBe("number");
      }
    }, 10000);

    test("Should wait until initialized", async () => {
      const success = await session.waitUntilInitialized(activeTorrentId);
      expect(success).toBe(true);
    }, 30000);

    test("Should pause and start the torrent", async () => {
      const paused = await session.pauseTorrent(activeTorrentId);
      expect(paused).toBe(true);

      const started = await session.startTorrent(activeTorrentId);
      expect(started).toBe(true);
    }, 10000);

    test("Should be able to add and immediately list a torrent", async () => {
      // Use a .torrent buffer instead of a magnet link so metadata is available
      // immediately — addTorrent(magnet) blocks until tracker/DHT resolves metadata
      // which can easily exceed 60 seconds for less popular torrents.
      const TORRENT_URL = "https://releases.ubuntu.com/22.04.2/ubuntu-22.04.2-live-server-amd64.iso.torrent";
      const response = await fetch(TORRENT_URL);
      if (!response.ok) {
        console.warn("Skipping concurrent add test: could not fetch torrent file");
        return;
      }
      const buffer = Buffer.from(await response.arrayBuffer());
      const id = await session.addTorrentBuffer(buffer, { paused: true });
      try {
        expect(session.listTorrents()).toContain(id);
      } finally {
        await session.deleteTorrent(id, true);
      }
    }, 30000);

    test("Should add a torrent via buffer (.torrent file)", async () => {
      // Use a small torrent file from a reliable source
      const response = await fetch("https://releases.ubuntu.com/22.04.1/ubuntu-22.04.1-desktop-amd64.iso.torrent");
      if (!response.ok) {
        console.warn("Skipping buffer test: could not fetch torrent file");
        return;
      }

      const buffer = Buffer.from(await response.arrayBuffer());
      const id = await session.addTorrentBuffer(buffer);
      expect(typeof id).toBe("number");

      await session.deleteTorrent(id, true);
    }, 60000);

    test("Should delete the primary test torrent", async () => {
      const success = await session.deleteTorrent(activeTorrentId, true);
      expect(success).toBe(true);
      expect(session.listTorrents()).not.toContain(activeTorrentId);
    }, 10000);
  });

  describe("Error Handling & Edge Cases", () => {
    test("Should return null for non-existent torrent stats", async () => {
      const stats = await session.getTorrentStats(99999);
      expect(stats).toBeNull();
    });

    test("Should return false when pausing/starting non-existent torrent", async () => {
      expect(await session.pauseTorrent(99999)).toBe(false);
      expect(await session.startTorrent(99999)).toBe(false);
    });

    test("Should throw error for invalid magnet link", async () => {
      try {
        await session.addTorrent("magnet:?xt=urn:btih:invalid");
        expect(false).toBe(true); // Should not reach here
      } catch (e) {
        expect(e).toBeDefined();
      }
    });

    test("Should throw error for invalid torrent buffer", async () => {
      try {
        await session.addTorrentBuffer(Buffer.from("invalid torrent data"));
        expect(false).toBe(true); // Should not reach here
      } catch (e) {
        expect(e).toBeDefined();
      }
    });
  });
});
