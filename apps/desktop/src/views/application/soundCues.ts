export type SoundCue =
  | "recording-start"
  | "recording-stop"
  | "dictation-start"
  | "dictation-cancel"
  | "dictation-complete";

const CUE_NOTES: Record<SoundCue, readonly number[]> = {
  "recording-start": [659.25, 880],
  "recording-stop": [880, 587.33],
  "dictation-start": [783.99, 1046.5],
  "dictation-cancel": [659.25, 493.88],
  "dictation-complete": [1046.5, 1318.51],
};
const NOTE_DURATION_SECONDS = 0.055;
const NOTE_GAP_SECONDS = 0.035;
const CUE_VOLUME = 0.06;

let audioContext: AudioContext | null = null;

export function playSoundCue(cue: SoundCue) {
  let context: AudioContext | null;

  try {
    context = getAudioContext();
  } catch {
    return;
  }

  if (!context) {
    return;
  }

  if (context.state === "suspended") {
    void context
      .resume()
      .then(() => scheduleCue(context, cue))
      .catch(() => undefined);
    return;
  }

  scheduleCue(context, cue);
}

function getAudioContext() {
  if (audioContext || typeof window === "undefined" || !window.AudioContext) {
    return audioContext;
  }

  audioContext = new window.AudioContext();
  return audioContext;
}

function scheduleCue(context: AudioContext, cue: SoundCue) {
  const start = context.currentTime + 0.01;

  for (const [index, frequency] of CUE_NOTES[cue].entries()) {
    const noteStart =
      start + index * (NOTE_DURATION_SECONDS + NOTE_GAP_SECONDS);
    const oscillator = context.createOscillator();
    const gain = context.createGain();

    oscillator.type = "sine";
    oscillator.frequency.setValueAtTime(frequency, noteStart);
    gain.gain.setValueAtTime(0, noteStart);
    gain.gain.linearRampToValueAtTime(CUE_VOLUME, noteStart + 0.008);
    gain.gain.exponentialRampToValueAtTime(
      0.001,
      noteStart + NOTE_DURATION_SECONDS,
    );
    oscillator.connect(gain).connect(context.destination);
    oscillator.start(noteStart);
    oscillator.stop(noteStart + NOTE_DURATION_SECONDS);
  }
}
