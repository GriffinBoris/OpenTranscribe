import { createPreviewSnapshot } from "@/core/native/preview/previewData";

test("uses Documents/OpenTranscribe as the preview default", () => {
  expect(createPreviewSnapshot().library?.path).toBe(
    "~/Documents/OpenTranscribe",
  );
});

test("uses the requested library path", () => {
  const snapshot = createPreviewSnapshot("/Volumes/Meetings");

  expect(snapshot.library?.path).toBe("/Volumes/Meetings");
  expect(snapshot.recent_sessions.length).toBeGreaterThan(0);
});
