import type { UtcTimestamp } from "@/types";

export class TestClock {
  private currentMs: number;

  constructor(initial = "2026-01-01T00:00:00.000Z") {
    const initialMs = Date.parse(initial);
    if (!Number.isFinite(initialMs)) {
      throw new Error(`Invalid test clock timestamp: ${initial}`);
    }

    this.currentMs = initialMs;
  }

  now(): UtcTimestamp {
    return new Date(this.currentMs).toISOString() as UtcTimestamp;
  }

  advance(milliseconds: number): UtcTimestamp {
    if (!Number.isInteger(milliseconds) || milliseconds < 0) {
      throw new Error("TestClock.advance expects a non-negative integer");
    }

    this.currentMs += milliseconds;
    return this.now();
  }

  set(value: string | number): UtcTimestamp {
    const nextMs = typeof value === "number" ? value : Date.parse(value);
    if (!Number.isFinite(nextMs)) {
      throw new Error("TestClock.set expects a valid timestamp");
    }

    this.currentMs = nextMs;
    return this.now();
  }
}
