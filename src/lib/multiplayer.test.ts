import { describe, expect, it } from "vitest";
import { multiplayerWinner } from "./multiplayer";
import type { FinishPayload } from "../state/game";

const run = (durationMs: number, clicks: number, success = true): FinishPayload => ({
  startTitle: "Minecraft", targetTitle: "Sweden", durationMs, clicks,
  clickLimit: 6, withinClickLimit: success, route: [], isPersonalBest: false,
  success, outcome: success ? "success" : "connectionLost", mode: "twoPlayer",
  difficulty: .75, streak: 0, bestStreak: 0, stageCount: 1,
});

describe("multiplayer winner", () => {
  it("ranks a completed run ahead of a failed run", () => expect(multiplayerWinner(run(2000, 3), run(1000, 2, false))).toBe(1));
  it("ranks time first and clicks as tie-breaker", () => {
    expect(multiplayerWinner(run(1000, 5), run(2000, 1))).toBe(1);
    expect(multiplayerWinner(run(1000, 5), run(1000, 4))).toBe(2);
  });
  it("allows exact ties", () => expect(multiplayerWinner(run(1000, 3), run(1000, 3))).toBe(0));
});
