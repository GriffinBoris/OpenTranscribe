import { describe, expect, it } from "vitest";

import {
  AdaptiveAudioLevelNormalizer,
  normalizeAudioLevel,
  normalizeWaveformLevels,
} from "@/views/application/audioLevels";

describe("normalizeAudioLevel", () => {
  it("filters the ambient-noise range at the bottom of the visible scale", () => {
    expect(normalizeAudioLevel(0)).toBe(0);
    expect(normalizeAudioLevel(0.001)).toBe(0);
    expect(normalizeAudioLevel(0.003)).toBe(0);
  });

  it("maps spoken-level input across the available meter height", () => {
    expect(normalizeAudioLevel(0.01)).toBeCloseTo(0.101, 3);
    expect(normalizeAudioLevel(0.1)).toBeCloseTo(0.63, 3);
    expect(normalizeAudioLevel(1)).toBe(1);
  });

  it("learns sustained ambient sound without suppressing a louder voice", () => {
    const meter = new AdaptiveAudioLevelNormalizer();

    for (let index = 0; index < 120; index += 1) {
      meter.normalize(0.02);
    }

    expect(meter.normalize(0.02)).toBeLessThan(0.05);
    expect(meter.normalize(0.1)).toBeGreaterThan(0.45);
  });

  it("applies the same adaptive gate to persisted waveform samples", () => {
    const levels = normalizeWaveformLevels([0.003, 0.1]);

    expect(levels[0]).toBe(0);
    expect(levels[1]).toBeCloseTo(0.632, 3);
  });
});
