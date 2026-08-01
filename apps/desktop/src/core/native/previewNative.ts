import type { NativeBridge } from "@/core/native/NativeBridge";
import { previewApplicationBridge } from "@/core/native/preview/applicationBridge";
import { previewRecordingBridge } from "@/core/native/preview/recordingBridge";
import { previewSessionBridge } from "@/core/native/preview/sessionBridge";
import { previewTranscriptionBridge } from "@/core/native/preview/transcriptionBridge";

export const previewNative = {
  ...previewApplicationBridge,
  ...previewRecordingBridge,
  ...previewTranscriptionBridge,
  ...previewSessionBridge,
} satisfies NativeBridge;
