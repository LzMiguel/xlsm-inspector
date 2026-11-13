import { describe, expect, it } from "vitest";

import { withRecent, type RecentFile } from "./recentFiles";

const f = (path: string, openedAt = 0): RecentFile => ({ path, name: path, openedAt });

describe("withRecent", () => {
  it("põe no topo e não duplica", () => {
    const out = withRecent([f("a"), f("b")], f("b", 5));
    expect(out.map((r) => r.path)).toEqual(["b", "a"]);
    expect(out[0].openedAt).toBe(5);
  });

  it("limita a 10", () => {
    const list = Array.from({ length: 10 }, (_, i) => f(String(i)));
    const out = withRecent(list, f("novo"));
    expect(out).toHaveLength(10);
    expect(out[0].path).toBe("novo");
  });
});
