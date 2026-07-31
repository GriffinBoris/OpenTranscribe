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
  await expect(page.locator(".window-titlebar")).not.toHaveAttribute(
    "data-tauri-drag-region",
  );
  const dragRegion = page.locator(".window-titlebar__drag-region");
  await expect(dragRegion).toHaveAttribute("data-tauri-drag-region", "");
  await expect(dragRegion).toHaveCSS("left", "88px");
  await expect(dragRegion).toHaveCSS("right", "120px");
  await expect(page.locator(".window-titlebar__handle")).toBeVisible();
  await expect(page.getByText("Recent sessions")).toBeVisible();
  await expect(
    page
      .getByRole("link", { name: "Product", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("2");
  await expect(
    page
      .getByRole("link", { name: "Customer interviews", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("0");

  await page.keyboard.press("Control+K");
  await expect(
    page.getByRole("dialog", { name: "Search library" }),
  ).toBeVisible();
  await page
    .getByRole("dialog", { name: "Search library" })
    .evaluate(async (dialog) =>
      Promise.all(
        dialog
          .getAnimations({ subtree: true })
          .map((animation) => animation.finished),
      ),
    );
  const librarySearch = page.getByPlaceholder(
    "Search sessions, transcripts, and notes",
  );
  const librarySearchForm = page.locator(".library-search");
  const searchButton = page
    .getByRole("dialog")
    .getByRole("button", { name: "Search" });
  const [formBox, inputBox, buttonBox] = await Promise.all([
    librarySearchForm.boundingBox(),
    librarySearch.boundingBox(),
    searchButton.boundingBox(),
  ]);
  expect(formBox).not.toBeNull();
  expect(inputBox).not.toBeNull();
  expect(buttonBox).not.toBeNull();
  expect(Math.round(inputBox!.x)).toBe(Math.round(formBox!.x));
  expect(Math.round(buttonBox!.x + buttonBox!.width)).toBe(
    Math.round(formBox!.x + formBox!.width),
  );
  expect(Math.round(inputBox!.height)).toBe(Math.round(buttonBox!.height));
  await expect(page.locator(".app-search-input__icon")).toHaveCSS(
    "position",
    "absolute",
  );
  const searchIcon = page.locator(".app-search-input__icon svg");
  await expect(searchIcon).toHaveCSS("width", "16px");
  await expect(searchIcon).toHaveCSS("height", "16px");
  await expect(librarySearch).toHaveCSS("padding-left", "36px");
  await librarySearch.fill("weekly");
  await searchButton.click();
  await expect(
    page
      .getByRole("dialog")
      .getByRole("button", { name: /Weekly product sync/ }),
  ).toBeVisible();
  await expect
    .poll(() =>
      page.locator(".library-search__results").evaluate((results) => {
        const form = results.previousElementSibling;
        return form
          ? Math.round(
              results.getBoundingClientRect().top -
                form.getBoundingClientRect().bottom,
            )
          : 0;
      }),
    )
    .toBe(14);

  await librarySearch.fill("no matching session");
  await searchButton.click();
  const noMatches = page.locator(".library-search + .empty-setting");
  await expect(noMatches).toBeVisible();
  await expect
    .poll(() =>
      page.locator(".library-search").evaluate((form) => {
        const noMatchesBox = form.nextElementSibling?.getBoundingClientRect();
        return noMatchesBox
          ? Math.round(noMatchesBox.top - form.getBoundingClientRect().bottom)
          : 0;
      }),
    )
    .toBe(14);
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
  await expect(dialog.getByLabel("Transcription")).toBeVisible();
  await expect(dialog.getByLabel("Spoken language")).toContainText(
    "Automatic detection",
  );

  await dialog.getByRole("button", { name: "Start recording" }).click();

  await expect(
    page.getByRole("heading", { name: "Quarterly planning review" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop" })).toBeVisible();
});

test("defaults recording to the first project and remembers the last selection", async ({
  page,
}) => {
  await page.goto("/");
  await page
    .getByRole("button", { name: "Recording options", exact: true })
    .click();

  const dialog = page.getByRole("dialog", { name: "Recording options" });
  const project = dialog.getByLabel("Project");
  await expect(project).toContainText("Product");
  await project.click();
  await page.getByRole("option", { name: "Customer interviews" }).click();
  await expect(project).toContainText("Customer interviews");
  await dialog.getByRole("button", { name: "Cancel" }).click();

  await page
    .getByRole("button", { name: "Recording options", exact: true })
    .click();
  await expect(dialog.getByLabel("Project")).toContainText(
    "Customer interviews",
  );
});

test("uses quick workspace motion and respects reduced motion", async ({
  page,
}) => {
  await page.goto("/");

  await page
    .getByRole("button", { name: "Recording options", exact: true })
    .click();

  const dialog = page.getByRole("dialog", { name: "Recording options" });
  await expect(dialog).toHaveCSS("transition-duration", "0.16s, 0.16s");

  await page.evaluate(() => {
    document.documentElement.dataset.reducedMotion = "true";
  });

  await expect
    .poll(() =>
      dialog.evaluate((element) =>
        Number.parseFloat(getComputedStyle(element).transitionDuration),
      ),
    )
    .toBeLessThan(0.001);
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
  await expect(page.getByLabel("Move to project")).toContainText("Product");
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

test("shows storage and shortcut utilities in settings", async ({ page }) => {
  await page.goto("/settings#storage");

  await expect(page.getByRole("heading", { name: "Storage" })).toBeVisible();
  await expect(page.getByText("~/Documents/OpenTranscribe")).toBeVisible();
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

test("uses quieter one-pixel dividers in settings", async ({ page }) => {
  await page.goto("/settings");

  const divider = page.locator(".setting-field").first();
  const control = page.locator(".app-input").first();
  await expect(divider).toBeVisible();
  await expect(control).toBeVisible();

  const [dividerStyle, controlStyle] = await Promise.all([
    divider.evaluate((element) => {
      const style = getComputedStyle(element);
      return { color: style.borderTopColor, width: style.borderTopWidth };
    }),
    control.evaluate((element) => {
      const style = getComputedStyle(element);
      return { color: style.borderTopColor, width: style.borderTopWidth };
    }),
  ]);

  expect(dividerStyle.width).toBe(controlStyle.width);
  expect(dividerStyle.width).toBe("1px");
  expect(dividerStyle.color).not.toBe(controlStyle.color);
});

test("keeps settings navigation within the natural scroll range", async ({
  page,
}) => {
  await page.goto("/settings");

  for (const [name, id] of [
    ["Storage", "storage"],
    ["Local models", "models"],
    ["OpenAI", "openai"],
    ["Shortcuts", "shortcuts"],
  ]) {
    await page.getByRole("button", { name, exact: true }).click();

    await expect
      .poll(() =>
        page.locator(`#${id}`).evaluate((section) => {
          const scrollPane = document.querySelector(".settings-content");
          if (!scrollPane) {
            return 0;
          }

          return Math.round(
            section.getBoundingClientRect().top -
              scrollPane.getBoundingClientRect().top,
          );
        }),
      )
      .toBe(20);
    await expect(page.locator(".settings-content-frame")).toHaveClass(
      /settings-content-frame--scrolled/,
    );
  }

  await page.getByRole("button", { name: "Appearance", exact: true }).click();
  await expect
    .poll(() =>
      page.locator(".settings-content").evaluate((scrollPane) => ({
        atBottom:
          Math.round(scrollPane.scrollTop) ===
          Math.round(scrollPane.scrollHeight - scrollPane.clientHeight),
        trailingSpace: Math.round(
          scrollPane.scrollHeight -
            (document.getElementById("appearance")!.offsetTop +
              document.getElementById("appearance")!.offsetHeight),
        ),
      })),
    )
    .toEqual({
      atBottom: true,
      trailingSpace: 0,
    });

  await page.getByRole("button", { name: "Recording", exact: true }).click();

  await expect
    .poll(() =>
      page
        .locator(".settings-content")
        .evaluate((scrollPane) => scrollPane.scrollTop),
    )
    .toBe(0);
  await expect(page.locator(".settings-content-frame")).not.toHaveClass(
    /settings-content-frame--scrolled/,
  );
});

test("keeps route scrolling inside each workspace", async ({ page }) => {
  await page.goto("/settings");

  await expect(page.locator(".application-main")).toHaveCSS(
    "overflow-y",
    "hidden",
  );
  await expect(page.locator(".settings-nav")).toHaveCSS("overflow-y", "auto");
  await expect(page.locator(".settings-content")).toHaveCSS(
    "overflow-y",
    "auto",
  );
  await expect
    .poll(() =>
      page.locator(".settings-content-frame").evaluate((element) => {
        const style = getComputedStyle(element, "::before");
        return {
          backdropFilter: style.backdropFilter,
          hasGradientMask: style.maskImage.includes("linear-gradient"),
        };
      }),
    )
    .toEqual({
      backdropFilter: "blur(4px)",
      hasGradientMask: true,
    });

  for (const path of ["/inbox", "/projects/01KDEMOPROJECT", "/processing"]) {
    await page.goto(path);
    const scrollFrame = page.locator(
      path === "/processing"
        ? ".processing-page__scroll"
        : ".library-page__scroll",
    );
    await expect(scrollFrame).toHaveCSS("overflow-y", "auto");
  }
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

test("enables system output when macOS capture is ready", async ({ page }) => {
  await page.goto("/settings?systemAudioPermission=granted");

  const systemOutput = page.getByRole("switch", {
    name: "Capture system output",
  });
  await expect(systemOutput).toBeEnabled();
  await expect(systemOutput).not.toBeChecked();

  await systemOutput.click();

  await expect(systemOutput).toBeChecked();
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await expect(page.getByText("System output · On")).toBeVisible();
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

test("keeps processing and session actions inside compact workspaces", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1024, height: 768 });
  await page.goto("/processing");

  const jobs = page.locator(".processing-page__jobs");
  const cancel = page.getByRole("button", { name: "Cancel" });
  const [jobsBox, cancelBox] = await Promise.all([
    jobs.boundingBox(),
    cancel.boundingBox(),
  ]);

  expect(jobsBox).not.toBeNull();
  expect(cancelBox).not.toBeNull();
  expect(cancelBox?.x + cancelBox?.width).toBeLessThanOrEqual(
    jobsBox!.x + jobsBox!.width,
  );

  await page.goto("/sessions/01KDEMOSESSION1");

  const title = page.getByRole("heading", { name: "Weekly product sync" });
  const trash = page.getByRole("button", {
    name: "Move session to trash",
  });
  const [titleBox, trashBox] = await Promise.all([
    title.boundingBox(),
    trash.boundingBox(),
  ]);

  expect(titleBox).not.toBeNull();
  expect(trashBox).not.toBeNull();
  expect(trashBox!.x).toBeGreaterThan(titleBox!.x + titleBox!.width);
  expect(Math.abs(trashBox!.y - titleBox!.y)).toBeLessThan(12);
  await page.setViewportSize({ width: 800, height: 760 });
  await expect(
    page.locator(".sidebar__primary-actions .app-button").first(),
  ).toHaveCSS("width", "38px");
  await expect(
    page.getByText("New recording", { exact: true }),
  ).not.toBeVisible();
});

test("keeps inline form controls on the medium design-token tier", async ({
  page,
}) => {
  await page.goto("/settings#openai");

  const apiKeyForm = page.locator(".openai-key-form");
  const input = apiKeyForm.locator(".app-input");
  const button = apiKeyForm.getByRole("button", { name: "Save to keychain" });
  const mediumControlHeight = await page.evaluate(() =>
    getComputedStyle(document.documentElement)
      .getPropertyValue("--control-height-medium")
      .trim(),
  );

  expect(mediumControlHeight).toBe("38px");
  await expect(input).toHaveCSS("min-height", mediumControlHeight);
  await expect(button).toHaveCSS("min-height", mediumControlHeight);
  await expect
    .poll(async () => {
      const [inputBox, buttonBox] = await Promise.all([
        input.boundingBox(),
        button.boundingBox(),
      ]);
      return [inputBox?.height, buttonBox?.height];
    })
    .toEqual([
      Number.parseFloat(mediumControlHeight),
      Number.parseFloat(mediumControlHeight),
    ]);
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

  const inboxRowProject = page.getByLabel(
    "Move Customer discovery — Rowan to a project or Inbox",
  );
  await expect(inboxRowProject).toContainText("Inbox");
  await inboxRowProject.click();
  await page.getByRole("option", { name: "Product", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).not.toBeVisible();
  await expect(
    page
      .getByRole("link", { name: "Product", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("2");

  await page.getByRole("link", { name: "Product", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).toBeVisible();

  const projectRowProject = page.getByLabel(
    "Move Customer discovery — Rowan to a project or Inbox",
  );
  await expect(projectRowProject).toContainText("Product");
  await projectRowProject.click();
  await page.getByRole("option", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).not.toBeVisible();
  await expect(
    page
      .getByRole("link", { name: "Product", exact: true })
      .locator(".sidebar__project-count"),
  ).toHaveText("1");

  await page.getByRole("link", { name: "Inbox", exact: true }).click();
  await expect(page.getByText("Customer discovery — Rowan")).toBeVisible();
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

  const balancedModel = page.getByRole("group", { name: "Balanced" });
  await balancedModel.getByRole("button", { name: "Download" }).click();
  await expect(
    balancedModel.getByText("Installed", { exact: true }),
  ).toBeVisible();

  await page.getByRole("link", { name: "Home", exact: true }).click();
  await page.getByRole("button", { name: "More recording options" }).click();
  await page
    .getByRole("menuitem", { name: "Record, then transcribe locally" })
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
    .getByRole("option", { name: "Record, then transcribe locally" })
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
