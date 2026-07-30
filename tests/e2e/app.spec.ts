import { expect, test } from "./fixtures";

test("opens the utility shell and starts a preview recording", async ({
  page,
}) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Home" })).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Main navigation" }),
  ).toBeVisible();
  const brandIcon = page.getByTestId("app-brand-icon");
  await expect(brandIcon).toBeVisible();
  await expect
    .poll(() =>
      brandIcon.evaluate((image: HTMLImageElement) => image.naturalWidth),
    )
    .toBeGreaterThan(0);
  await expect(page.locator(".window-titlebar")).toHaveAttribute(
    "data-tauri-drag-region",
    "",
  );
  await expect(page.locator(".window-titlebar__handle")).toBeVisible();
  await expect(page.locator(".window-titlebar__handle")).toHaveAttribute(
    "data-tauri-drag-region",
    "",
  );
  await expect(page.getByText("Recent sessions")).toBeVisible();

  await page.keyboard.press("Control+K");
  await expect(
    page.getByRole("dialog", { name: "Search library" }),
  ).toBeVisible();
  await page
    .getByPlaceholder("Search sessions, transcripts, and notes")
    .fill("weekly");
  await page
    .getByRole("dialog")
    .getByRole("button", { name: "Search" })
    .click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("button", { name: /Weekly product sync/ }),
  ).toBeVisible();
  await page.keyboard.press("Escape");

  await page.getByRole("button", { name: "Start recording" }).click();

  await expect(page.getByText("Recording safely to disk")).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();
});

test("completes first-run setup with a timed source test", async ({ page }) => {
  await page.clock.install();
  await page.goto("/?firstRun=1");

  await expect(
    page.getByRole("heading", { name: "Set up recording" }),
  ).toBeVisible();
  await expect(page.getByText(/Recordings are saved in/)).toBeVisible();

  await page.getByRole("button", { name: "Start 10-second test" }).click();
  await expect(page.getByText(/remaining/)).toBeVisible();

  await page.clock.fastForward(10_500);

  await expect(
    page.getByRole("heading", { name: "Audio test ready" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Finish setup" }).click();

  await expect(page.getByText("Recent sessions")).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Set up recording" }),
  ).not.toBeVisible();
});

test("recovers an interrupted session from its crash-safe chunks", async ({
  page,
}) => {
  await page.goto("/sessions/01KDEMOSESSION1?recovery=1");

  await expect(page.getByText("Recovery needed")).toBeVisible();
  await expect(
    page.getByText(
      "Crash-safe audio chunks were found. Recover them to make this session playable.",
    ),
  ).toBeVisible();

  await page.getByRole("button", { name: "Recover audio" }).click();

  await expect(
    page.getByRole("button", { name: "Recover audio" }),
  ).not.toBeVisible();
  await expect(page.getByText("Recovery needed")).not.toBeVisible();

  const notes = page.getByRole("textbox", { name: "Meeting notes" });
  await notes.fill("Agenda");
  await page.keyboard.press("Control+Shift+M");
  await expect(notes).toHaveValue(/Agenda\n- \[\d+:\d{2}\] /);

  await page.keyboard.press("Control+F");
  const search = page.getByRole("dialog", { name: "Search this session" });
  await search.getByPlaceholder("Search transcript and notes").fill("0:00");
  const timestampedNote = search.getByRole("button", {
    name: /Notes · 0:00.*\[0:00\]/,
  });
  await expect(timestampedNote).toBeVisible();
  await timestampedNote.click();
  await expect(search).not.toBeVisible();
});

test("starts a session with customized recording options", async ({ page }) => {
  await page.goto("/");

  await page
    .getByRole("button", { name: "Recording options", exact: true })
    .click();

  const dialog = page.getByRole("dialog", { name: "Recording options" });
  await expect(dialog).toBeVisible();
  await dialog.getByLabel("Meeting title").fill("Quarterly planning review");
  await expect(dialog.getByLabel("Project")).toBeVisible();
  await expect(dialog.getByLabel("Microphone")).toBeVisible();
  await expect(dialog.getByLabel("After recording")).toBeVisible();
  await expect(dialog.getByLabel("Spoken language")).toBeVisible();

  await dialog.getByRole("button", { name: "Start recording" }).click();

  await expect(
    page.getByRole("heading", { name: "Quarterly planning review" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();
});

test("opens the session workspace from the sidebar recording action", async ({
  page,
}) => {
  await page.goto("/inbox");

  await page.getByRole("button", { name: "New recording" }).click();

  await expect(page).toHaveURL(/\/sessions\/[^/]+$/);
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();
  await expect(
    page.getByRole("textbox", { name: "Meeting notes" }),
  ).toBeVisible();
  await expect(page.getByText("Recovery needed")).not.toBeVisible();
});

test("shows storage and shortcut utilities in settings", async ({ page }) => {
  await page.goto("/settings#storage");

  await expect(page.getByRole("heading", { name: "Storage" })).toBeVisible();
  await expect(
    page.getByText("~/Documents/OpenTranscribe Library"),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Change folder" }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Shortcuts" }).click();

  await expect(page.getByRole("heading", { name: "Shortcuts" })).toBeVisible();
  await expect(
    page
      .locator("#shortcuts .section-heading")
      .getByText(
        "Control recording from anywhere and review shortcuts available in the app.",
      ),
  ).toBeVisible();
  await expect(page.getByText("Import media")).toBeVisible();
  await expect(page.getByText("⌘/Ctrl O")).toBeVisible();

  const globalRecordingToggle = page.getByRole("switch", {
    name: "Record from anywhere",
  });
  await globalRecordingToggle.focus();
  await page.keyboard.press("Space");
  await expect(page.getByLabel("Global recording shortcut")).toBeVisible();
  await expect(page.getByText("Active", { exact: true })).toBeVisible();

  await page.evaluate(() => {
    window.dispatchEvent(new Event("opentranscribe:preview-global-shortcut"));
  });
  await expect(page).toHaveURL(/\/sessions\/[^/]+$/);
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();

  await page.evaluate(() => {
    window.dispatchEvent(new Event("opentranscribe:preview-global-shortcut"));
  });
  await expect(page.getByRole("button", { name: "Stop" })).not.toBeVisible();
});

test("surfaces a global shortcut registration conflict", async ({ page }) => {
  await page.goto("/settings?globalShortcutError=1#shortcuts");

  const globalRecordingToggle = page.getByRole("switch", {
    name: "Record from anywhere",
  });
  await globalRecordingToggle.focus();
  await page.keyboard.press("Space");

  await expect(page.getByText("Unavailable", { exact: true })).toBeVisible();
  await expect(
    page.getByText(
      "This shortcut could not be registered: This shortcut is already in use.",
    ),
  ).toBeVisible();
});

test("shows macOS system-audio permission status and recovery accurately", async ({
  page,
}) => {
  await page.goto("/settings?systemAudioPermission=granted");

  await expect(page.getByText("System audio ready")).toBeVisible();
  await expect(
    page.getByText(
      "Screen & System Audio Recording permission is granted for this build.",
    ),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Open System Settings" }),
  ).not.toBeVisible();

  await page.goto("/settings?systemAudioPermission=required");

  await expect(page.getByText("Permission required")).toBeVisible();
  await expect(
    page.getByText(
      "Allow OpenTranscribe under Screen & System Audio Recording, then relaunch it.",
    ),
  ).toBeVisible();
  await page.getByRole("button", { name: "Open System Settings" }).click();
  await expect(page.locator(".shell-state--error")).not.toBeVisible();
});

test("filters sessions within a project", async ({ page }) => {
  await page.goto("/projects/01KDEMOPROJECT");

  const search = page.getByPlaceholder("Search this project");
  await search.fill("weekly");
  await expect(page.getByText("Weekly product sync")).toBeVisible();

  await search.fill("missing session");
  await expect(page.getByText("No sessions match this search.")).toBeVisible();
});

test("renders substantial smoothly composited processing progress", async ({
  page,
}) => {
  await page.goto("/processing");

  const progress = page.locator(".app-progress").first();
  const value = progress.locator(".app-progress__value");

  await expect(progress).toHaveCSS("height", "10px");
  await expect(value).toHaveCSS("transition-property", "transform");
  await expect
    .poll(() =>
      value.evaluate((element) => getComputedStyle(element).transform),
    )
    .not.toBe("none");
});

test("moves a session between a project and the inbox", async ({ page }) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  const project = page.getByLabel("Move to project");
  await expect(project).toContainText("Product");
  await project.click();
  await page.getByRole("option", { name: "Inbox", exact: true }).click();
  await expect(project).toContainText("Inbox");

  const notes = page.getByRole("textbox", { name: "Meeting notes" });
  await notes.click();
  await expect(notes).toHaveCSS("box-shadow", "none");

  await page.getByRole("link", { name: "Product", exact: true }).click();
  await expect(page.getByText("Weekly product sync")).not.toBeVisible();

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Weekly product sync")).toBeVisible();
});

test("supports keyboard activation for recording and stop", async ({
  page,
}) => {
  await page.goto("/");

  const start = page.getByRole("button", { name: "New recording" });
  await start.focus();
  await page.keyboard.press("Enter");

  const stop = page.getByRole("button", { name: "Stop" });
  await expect(stop).toBeVisible();
  await stop.focus();
  await page.keyboard.press("Enter");

  await expect(stop).not.toBeVisible();
});

test("queues the selected transcription when recording stops", async ({
  page,
}) => {
  await page.goto("/settings#models");

  const balancedModel = page.locator(".model-entry").filter({
    has: page.getByText("Balanced", { exact: true }),
  });
  await balancedModel.getByRole("button", { name: "Download" }).click();
  await expect(
    balancedModel.getByText("Installed", { exact: true }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Home", exact: true }).click();
  await page.getByRole("button", { name: "More recording options" }).click();
  await page
    .getByRole("menuitem", { name: "Record and transcribe locally" })
    .click();

  const stop = page.getByRole("button", { name: "Stop" });
  await expect(stop).toBeVisible();
  await stop.click();

  await expect(stop).not.toBeVisible();
  await expect(page.getByText("Waiting to start")).toBeVisible();
});

test("keeps the recording when automatic transcription cannot start", async ({
  page,
}) => {
  await page.goto("/settings");
  await page.getByLabel("Default recording action").click();
  await page
    .getByRole("option", { name: "Record and transcribe locally" })
    .click();
  await page.getByRole("button", { name: "New recording" }).click();

  const stop = page.getByRole("button", { name: "Stop" });
  await expect(stop).toBeVisible();
  await stop.click();

  await expect(stop).not.toBeVisible();
  await expect(
    page.getByText("Install a local transcription model in Settings."),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: /Untitled recording/ }),
  ).toBeVisible();
});

test("keeps the recording timeline fixed while paused", async ({ page }) => {
  await page.clock.install();
  await page.goto("/");
  await page.getByRole("button", { name: "Start recording" }).click();

  await page.clock.fastForward(5_000);
  await expect(page.getByText("00:00:05")).toBeVisible();

  await page.getByRole("button", { name: "Pause" }).click();
  await page.clock.fastForward(5_000);
  await expect(page.getByText("00:00:05")).toBeVisible();

  await page.getByRole("button", { name: "Resume" }).click();
  await page.clock.fastForward(2_000);
  await expect(page.getByText("00:00:07")).toBeVisible();
});
