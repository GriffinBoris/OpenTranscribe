import { expect, test } from "./fixtures";

test("offers a newly created project in recording options", async ({
  page,
}) => {
  await page.goto("/");

  await page.getByRole("button", { name: "New project" }).click();
  const projectDialog = page.getByRole("dialog", { name: "New project" });
  await projectDialog.getByLabel("Project name").fill("Launch planning");
  await projectDialog.getByRole("button", { name: "Create project" }).click();
  await expect(projectDialog).not.toBeVisible();

  await page
    .getByRole("button", { name: "Recording options", exact: true })
    .click();

  const recordingDialog = page.getByRole("dialog", {
    name: "Recording options",
  });
  const project = recordingDialog.getByLabel("Project");
  await project.click();
  await expect(
    page.getByRole("option", { name: "Launch planning", exact: true }),
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

test("uses the configured transcription mode for the Home recording button", async ({
  page,
}) => {
  await page.goto("/settings#models");

  const balancedModel = page.getByRole("group", { name: "Balanced" });
  await balancedModel.getByRole("button", { name: "Download" }).click();
  await expect(
    balancedModel.getByText("Installed", { exact: true }),
  ).toBeVisible();

  await page.getByRole("button", { name: "Recording", exact: true }).click();
  await page.getByLabel("Default recording action").click();
  await page
    .getByRole("option", { name: "Record, then transcribe locally" })
    .click();
  await page.getByRole("link", { name: "Home", exact: true }).click();

  const stop = page.getByRole("button", { name: "Stop" });
  await page.getByRole("button", { name: "New recording" }).click();
  await expect(stop).toBeVisible();
  await stop.click();

  await expect(page.getByText("Waiting to start")).toBeVisible();
});

test("keeps the recording when automatic transcription cannot start", async ({
  page,
}) => {
  await page.goto("/settings?localModels=none");
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

test("keeps setup progress while configuring transcription", async ({
  page,
}) => {
  await page.goto("/?firstRun=1");

  await page.getByRole("button", { name: "Continue" }).click();
  await page.getByRole("button", { name: "Continue" }).click();
  await expect(
    page.getByRole("heading", { name: "Transcription" }),
  ).toBeVisible();

  await page.getByLabel("Transcription").click();
  await page.getByRole("option", { name: "Transcribe locally" }).click();
  await page.getByRole("button", { name: "Choose a local model" }).click();

  await expect(page).toHaveURL(/\/settings#models$/);
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await expect(
    page.getByRole("heading", { name: "Transcription" }),
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
