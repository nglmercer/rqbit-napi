import { expect, test, describe, beforeAll, afterAll } from "bun:test";
import { RqbitSession } from "../index";
import { mkdtemp, rm } from "fs/promises";
import { join } from "path";
import { tmpdir } from "os";

describe("RqbitSession NAPI Tests", () => {
  let session: RqbitSession;
  let testDir: string;

  beforeAll(async () => {
    // Use a fresh temp directory for every test run to avoid DHT persistence issues
    testDir = await mkdtemp(join(tmpdir(), "rqbit-test-"));
    session = await RqbitSession.create(testDir, { disableDht: false, disableDhtPersistence: true });
  });

  afterAll(async () => {
    if (session) {
      await session.stop();
    }
    if (testDir) {
      await rm(testDir, { recursive: true, force: true });
    }
  });

  test("Session should be created", () => {
    expect(session).toBeDefined();
  });

  test("Should add a torrent", async () => {
    // Using a tiny torrent for faster testing if possible, or just a magnet
    const magnet = "magnet:?xt=urn:btih:cab507494d02ebb1178b38f2e9d7be299c86b862&dn=ubuntu-22.04.1-desktop-amd64.iso&tr=https%3A%2F%2Ftorrent.ubuntu.com%3A443%2Fannounce";
    const id = await session.addTorrent(magnet);
    expect(typeof id).toBe("number");
    expect(id).toBeGreaterThanOrEqual(0);
  }, 30000);

  test("Should list torrents", () => {
    const ids = session.listTorrents();
    expect(Array.isArray(ids)).toBe(true);
    expect(ids.length).toBeGreaterThan(0);
  });

  test("Should get torrent stats", async () => {
    const ids = session.listTorrents();
    const id = ids[0];
    const stats = await session.getTorrentStats(id);
    expect(stats).not.toBeNull();
    if (stats) {
      expect(typeof stats.name).toBe("string");
      expect(typeof stats.finished).toBe("boolean");
      expect(typeof stats.totalBytes).toBe("number");
      expect(typeof stats.downloadedBytes).toBe("number");
      expect(typeof stats.uploadedBytes).toBe("number");
      expect(typeof stats.downloadSpeed).toBe("number");
      expect(typeof stats.uploadSpeed).toBe("number");
    }
  }, 30000);

  test("Should pause and resume torrent", async () => {
    const ids = session.listTorrents();
    const id = ids[0];
    
    await session.waitUntilInitialized(id);
    const paused = await session.pauseTorrent(id);
    expect(paused).toBe(true);
    
    const resumed = await session.startTorrent(id);
    expect(resumed).toBe(true);
  }, 30000);

  test("Should get session stats", () => {
    const stats = session.getSessionStats();
    expect(stats).toBeDefined();
    expect(typeof stats.uptimeSeconds).toBe("number");
    expect(typeof stats.downloadSpeed).toBe("number");
    expect(typeof stats.uploadSpeed).toBe("number");
    expect(typeof stats.fetchedBytes).toBe("number");
    expect(typeof stats.uploadedBytes).toBe("number");
  });

  test("Should delete torrent", async () => {
    const ids = session.listTorrents();
    const id = ids[0];
    
    const deleted = await session.deleteTorrent(id, true);
    expect(deleted).toBe(true);
    
    const remainingIds = session.listTorrents();
    expect(remainingIds).not.toContain(id);
  }, 30000);

  test("Should handle non-existent torrent stats", async () => {
    const stats = await session.getTorrentStats(9999);
    expect(stats).toBeNull();
  }, 30000);
});
