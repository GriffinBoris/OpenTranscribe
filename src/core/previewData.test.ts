import { createPreviewSnapshot } from "@/core/previewData";

test("uses the requested library path", () => {
  const snapshot = createPreviewSnapshot("/Volumes/Meetings");

  expect(snapshot.library?.path).toBe("/Volumes/Meetings");
  expect(snapshot.recent_sessions.length).toBeGreaterThan(0);
});
