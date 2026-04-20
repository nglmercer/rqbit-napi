import { expect, test, describe } from "bun:test";
import {
  RqbitSession
} from "../index";

describe("NAPI Module Tests", () => {
  // 1. Test Synchronous Functions
  test("Session", async () => {
    const session = await RqbitSession.create("/tmp");
    console.log(session)
    expect(session).toBeDefined();
  });

});
