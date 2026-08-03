const METER_FLOOR_DECIBELS = -60;
const METER_CEILING_DECIBELS = -6;
const DEFAULT_NOISE_GATE = 0.3;
const AMBIENT_CEILING = 0.55;
const AMBIENT_TRACKING_MARGIN = 0.24;
const AMBIENT_SMOOTHING = 0.04;

export function normalizeAudioLevel(peak: number) {
  const level = decibelLevel(peak);
  return gateLevel(level, DEFAULT_NOISE_GATE);
}

export class AdaptiveAudioLevelNormalizer {
  private ambientLevel = DEFAULT_NOISE_GATE;

  normalize(peak: number) {
    const level = decibelLevel(peak);

    if (
      level <= AMBIENT_CEILING &&
      level <= this.ambientLevel + AMBIENT_TRACKING_MARGIN
    ) {
      this.ambientLevel += (level - this.ambientLevel) * AMBIENT_SMOOTHING;
    }

    return gateLevel(level, this.ambientLevel);
  }

  reset() {
    this.ambientLevel = DEFAULT_NOISE_GATE;
  }
}

export function normalizeWaveformLevels(peaks: number[]) {
  const normalizer = new AdaptiveAudioLevelNormalizer();
  return peaks.map((peak) => normalizer.normalize(peak));
}

function decibelLevel(peak: number) {
  if (peak <= 0) {
    return 0;
  }

  const decibels = 20 * Math.log10(Math.min(1, peak));
  return Math.min(
    1,
    Math.max(
      0,
      (decibels - METER_FLOOR_DECIBELS) /
        (METER_CEILING_DECIBELS - METER_FLOOR_DECIBELS),
    ),
  );
}

function gateLevel(level: number, gate: number) {
  return Math.max(0, (level - gate) / (1 - gate));
}
