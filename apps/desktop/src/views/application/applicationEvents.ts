import { useDictationStore } from "@/views/application/dictationStore";
import { native } from "@/core/native";
import type { AppEvent } from "@/types/domain";
import { useApplicationStore } from "@/views/application/applicationStore";
import { useRecordingStore } from "@/views/application/recordingStore";

let subscribed = false;

export async function subscribeToApplicationEvents() {
  if (subscribed) {
    return;
  }

  const application = useApplicationStore();
  const recording = useRecordingStore();

  await native.subscribe((event) => handleEvent(event, application, recording));
  subscribed = true;
}

function handleEvent(
  event: AppEvent,
  application: ReturnType<typeof useApplicationStore>,
  recording: ReturnType<typeof useRecordingStore>,
) {
  if (event.type === "dictation_state_changed") {
    useDictationStore().apply(event.payload);
    return;
  }
  if (
    event.type === "recording_levels" ||
    event.type === "recording_state_changed" ||
    event.type === "live_transcript_changed"
  ) {
    recording.handleEvent(event);
    return;
  }

  if (event.type === "job_state_changed") {
    application.upsertJob(event.payload);

    if (["completed", "failed", "canceled"].includes(event.payload.state)) {
      application.clearTranscriptionPreview(event.payload.session_id);
    }
    return;
  }

  if (event.type === "transcription_preview_changed") {
    application.updateTranscriptionPreview(event.payload);
    return;
  }

  if (event.type === "job_progress") {
    application.updateJobProgress(event.payload.job_id, event.payload.progress);
    return;
  }

  if (event.type === "library_changed") {
    void application.refreshLibrary();
    return;
  }

  if (event.type === "import_requested") {
    application.requestImport();
    return;
  }

  if (event.type === "attention_required") {
    application.operationError = event.payload;
  }
}
