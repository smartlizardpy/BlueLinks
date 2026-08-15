import { describe, expect, it } from "vitest";
import { formatDuration, formatDurationParts } from "./timerFormat";

describe("formatDuration", () => {
  it.each([[0,"00:00.00"],[999,"00:00.99"],[60_000,"01:00.00"],[3_599_990,"59:59.99"],[3_600_000,"1:00:00.00"]])("formats %i", (ms, expected) => {
    expect(formatDuration(ms)).toBe(expected);
  });
  it("separates LiveSplit-style fractional digits", () => {
    expect(formatDurationParts(75_430)).toEqual({ whole: "01:15", fraction: "43", label: "01:15.43" });
  });
});
