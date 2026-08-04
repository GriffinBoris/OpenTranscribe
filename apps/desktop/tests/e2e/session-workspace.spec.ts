import { expect, test } from "./fixtures";

test("cancels transcription from the session workspace", async ({ page }) => {
  await page.goto("/sessions/01KDEMOSESSION3");

  const cancel = page.getByRole("button", { name: "Cancel" });
  await expect(cancel).toBeVisible();
  await cancel.click();

  await expect(cancel).not.toBeVisible();
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

test("renders Markdown notes in preview mode", async ({ page }) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  const notesPane = page.getByRole("region", { name: "Notes" });
  const notes = notesPane.getByRole("textbox", { name: "Meeting notes" });
  await notes.fill("# Design review\n\n**Owner:** Rowan\n\n- Follow up");
  await notesPane.getByRole("button", { name: "Preview", exact: true }).click();

  await expect(
    notesPane.getByRole("heading", { name: "Design review" }),
  ).toBeVisible();
  await expect(notesPane.locator(".notes-preview__content strong")).toHaveText(
    "Owner:",
  );
  await expect(notesPane.getByRole("listitem")).toHaveText("Follow up");
  await expect(notes).not.toBeVisible();

  await notesPane.getByRole("button", { name: "Edit", exact: true }).click();
  await expect(notes).toHaveValue(
    "# Design review\n\n**Owner:** Rowan\n\n- Follow up",
  );
});

test("keeps notes toolbar controls inside a constrained pane", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1000, height: 800 });
  await page.goto("/sessions/01KDEMOSESSION1");

  const toolbar = page.locator(".notes-toolbar");
  await expect(
    toolbar.getByRole("button", { name: "Add timestamp" }),
  ).toBeVisible();
  await expect(
    toolbar.getByRole("button", { name: "Edit", exact: true }),
  ).toBeVisible();
  await expect(
    toolbar.getByRole("button", { name: "Preview", exact: true }),
  ).toBeVisible();

  const paneToolbars = page.locator(".session-pane-toolbar");
  await expect(paneToolbars).toHaveCount(2);
  const toolbarBounds = await paneToolbars.evaluateAll((elements) =>
    elements.map((element) => {
      const bounds = element.getBoundingClientRect();
      return { bottom: bounds.bottom, height: bounds.height };
    }),
  );

  expect(toolbarBounds.every(({ height }) => height === 52)).toBe(true);
  expect(
    Math.abs(toolbarBounds[0].bottom - toolbarBounds[1].bottom),
  ).toBeLessThan(0.5);

  await expect
    .poll(() =>
      toolbar.evaluate((element) => {
        const toolbarBounds = element.getBoundingClientRect();
        return Array.from(
          element.querySelectorAll<HTMLButtonElement>(
            ".notes-toolbar__actions button",
          ),
        ).every((button) => {
          const buttonBounds = button.getBoundingClientRect();
          return (
            buttonBounds.left >= toolbarBounds.left &&
            buttonBounds.right <= toolbarBounds.right
          );
        });
      }),
    )
    .toBe(true);
});

test("resizes and hides the transcript pane for focused note-taking", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/sessions/01KDEMOSESSION1");

  const workspace = page.locator(".session-workspace__panes");
  const notes = page.getByRole("region", { name: "Notes" });
  const separator = page.getByRole("separator", {
    name: "Resize transcript and notes",
  });
  const [notesBefore, separatorBefore] = await Promise.all([
    notes.boundingBox(),
    separator.boundingBox(),
  ]);

  expect(notesBefore).not.toBeNull();
  expect(separatorBefore).not.toBeNull();

  await page.mouse.move(
    separatorBefore!.x + separatorBefore!.width / 2,
    separatorBefore!.y + separatorBefore!.height / 2,
  );
  await page.mouse.down();
  await page.mouse.move(
    separatorBefore!.x - 120,
    separatorBefore!.y + separatorBefore!.height / 2,
  );
  await page.mouse.up();

  await expect
    .poll(async () => (await notes.boundingBox())?.width)
    .toBeGreaterThan(notesBefore!.width + 100);

  await notes.getByRole("button", { name: "Hide transcript" }).click();
  await expect(
    page.getByRole("separator", { name: "Resize transcript and notes" }),
  ).not.toBeVisible();
  await expect(
    notes.getByRole("button", { name: "Show transcript" }),
  ).toBeVisible();

  const [workspaceBox, notesBox] = await Promise.all([
    workspace.boundingBox(),
    notes.boundingBox(),
  ]);
  expect(workspaceBox).not.toBeNull();
  expect(notesBox).not.toBeNull();
  expect(Math.abs(notesBox!.width - workspaceBox!.width)).toBeLessThanOrEqual(
    1,
  );
});

test("keeps the session toolbar aligned across viewport sizes", async ({
  page,
}) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  const header = page.locator(".session-header");
  const metadata = page.locator(".session-header__metadata");
  const controls = page.locator(".session-header__controls");
  await expect(header).toBeVisible();
  await expect(metadata).toBeVisible();
  await expect(controls).toBeVisible();

  for (const width of [2200, 1900, 1200, 800]) {
    await page.setViewportSize({ width, height: 800 });

    const layout = await header.evaluate((headerElement) => {
      const headerBounds = headerElement.getBoundingClientRect();
      const metadataBounds = headerElement
        .querySelector(".session-header__metadata")
        ?.getBoundingClientRect();
      const controlsBounds = headerElement
        .querySelector(".session-header__controls")
        ?.getBoundingClientRect();
      const controls = Array.from(
        headerElement.querySelectorAll<HTMLElement>(
          ".session-header__controls button, .session-header__controls .app-select",
        ),
      );

      return {
        metadataAboveControls:
          metadataBounds !== undefined &&
          controlsBounds !== undefined &&
          metadataBounds.bottom <= controlsBounds.top,
        metadataSharesControlsRow:
          metadataBounds !== undefined &&
          controlsBounds !== undefined &&
          metadataBounds.top < controlsBounds.bottom &&
          controlsBounds.top < metadataBounds.bottom,
        controlsFit: controls.every((control) => {
          const bounds = control.getBoundingClientRect();

          return (
            bounds.left >= headerBounds.left &&
            bounds.right <= headerBounds.right &&
            bounds.top >= headerBounds.top &&
            bounds.bottom <= headerBounds.bottom
          );
        }),
        controlHeights: controls.map(
          (control) => control.getBoundingClientRect().height,
        ),
      };
    });

    if (width > 900) {
      expect(layout.metadataSharesControlsRow).toBe(true);
    } else {
      expect(layout.metadataAboveControls).toBe(true);
    }
    expect(layout.controlsFit).toBe(true);
    expect(
      layout.controlHeights.every((height) => Math.abs(height - 38) < 1.5),
    ).toBe(true);
  }
});

test("uses one transcription menu for local and OpenAI targets", async ({
  page,
}) => {
  await page.goto("/settings#models");

  await page
    .getByRole("group", { name: "Fast" })
    .getByRole("button", { name: "Download" })
    .click();
  await page.getByPlaceholder("sk-…").fill("sk-preview-transcription");
  await page.getByRole("button", { name: "Save to keychain" }).click();
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await page.getByText("Weekly product sync", { exact: true }).click();

  await expect(
    page.getByRole("button", { name: "Transcribe", exact: true }),
  ).toHaveCount(1);
  await page.getByRole("button", { name: "Transcription target" }).click();
  await expect(
    page.getByRole("menuitem", { name: "Local · Fast" }),
  ).toBeVisible();
  await expect(
    page.getByRole("menuitem", { name: "OpenAI · GPT Transcribe" }),
  ).toBeVisible();
});

test("opens export choices from the header utility icon", async ({ page }) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  const exportButton = page.getByRole("button", { name: "Export" });
  await expect(exportButton).toBeEnabled();
  await exportButton.click();

  const exportDialog = page.getByRole("dialog", { name: "Export" });
  await expect(exportDialog).toBeVisible();
  await expect(exportDialog.getByLabel("Export format")).toContainText(
    "Markdown",
  );
  await expect(
    exportDialog.getByText("Complete a transcription before exporting."),
  ).toBeVisible();
  await expect(
    exportDialog.getByRole("button", { name: "Export" }),
  ).toBeVisible();
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
  await expect(
    page.getByRole("button", { name: "Move to project" }),
  ).toBeDisabled();
  await expect(page.getByText("Recovery needed")).not.toBeVisible();
});

test("keeps playback inside the session workspace", async ({ page }) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  const workspace = page.locator(".session-workspace");
  await expect(workspace).toHaveCSS("overflow-y", "hidden");

  await workspace.evaluate((element) => {
    const playback = document.createElement("footer");
    playback.className = "playback-bar relative flex h-[66px]";
    playback.dataset.testid = "playback-layout-probe";
    element.append(playback);
  });

  const playback = page.getByTestId("playback-layout-probe");
  await expect(playback).toBeVisible();

  const [workspaceBox, playbackBox] = await Promise.all([
    workspace.boundingBox(),
    playback.boundingBox(),
  ]);

  expect(workspaceBox).not.toBeNull();
  expect(playbackBox).not.toBeNull();
  expect(playbackBox!.y + playbackBox!.height).toBeLessThanOrEqual(
    workspaceBox!.y + workspaceBox!.height + 1,
  );
});

test("renames a meeting from its workspace", async ({ page }) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  await page.getByRole("button", { name: "Rename meeting" }).click();
  const renameDialog = page.getByRole("dialog", { name: "Rename meeting" });
  await renameDialog.getByRole("textbox").fill("Weekly design review");
  await renameDialog.getByRole("button", { name: "Save name" }).click();

  await expect(renameDialog).not.toBeVisible();
  await expect(
    page.getByRole("heading", { name: "Weekly design review" }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Weekly design review")).toBeVisible();
});

test("deletes a session without showing a missing-item error", async ({
  page,
}) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  await page.getByRole("button", { name: "Move session to trash" }).click();
  await page.getByRole("button", { name: "Move to trash" }).click();

  await expect(page).toHaveURL("/inbox");
  await expect(page.getByText("Weekly product sync")).not.toBeVisible();
  await expect(page.locator(".shell-operation-error")).not.toBeVisible();

  await page.getByRole("link", { name: "Trash", exact: true }).click();
  await expect(page.getByText("Weekly product sync")).toBeVisible();
  await expect(
    page.getByText("The requested item does not exist."),
  ).not.toBeVisible();
});

test("permanently empties trash only after confirmation", async ({ page }) => {
  await page.goto("/sessions/01KDEMOSESSION1");

  await page.getByRole("button", { name: "Move session to trash" }).click();
  await page.getByRole("button", { name: "Move to trash" }).click();
  await page.getByRole("link", { name: "Trash", exact: true }).click();

  await page.getByRole("button", { name: "Empty trash" }).click();
  const dialog = page.getByRole("dialog", {
    name: "Empty Trash permanently?",
  });
  await expect(dialog).toContainText(
    "This permanently deletes every meeting in Trash. This cannot be undone.",
  );
  await dialog.getByRole("button", { name: "Delete permanently" }).click();

  await expect(dialog).not.toBeVisible();
  await expect(page.getByText("Trash is empty")).toBeVisible();
  await expect(page.getByText("Weekly product sync")).not.toBeVisible();
});
