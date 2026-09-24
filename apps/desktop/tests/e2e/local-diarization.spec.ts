import { expect, test } from "./fixtures";

test("downloads Nemotron independently and reruns speaker recognition without Whisper", async ({
  page,
}) => {
  await page.goto("/settings?transcript=1#models");
  const diarizer = page.getByRole("group", {
    name: "Nemotron 3 speaker recognition",
    exact: true,
  });
  const best = page.getByRole("group", { name: "Best", exact: true });
  await diarizer.getByRole("button", { name: "Download", exact: true }).click();
  await expect(diarizer.getByText("Installed", { exact: true })).toBeVisible();
  await expect(best.getByText("Not installed", { exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await page.getByText("Weekly product sync", { exact: true }).click();
  const rerun = page.getByRole("button", {
    name: "Re-identify speakers",
    exact: true,
  });
  await expect(
    page.getByText("These corrected meeting notes should stay unchanged."),
  ).toBeVisible();
  await expect(rerun).toBeEnabled();
  await rerun.click();
  await expect(rerun).toBeDisabled();
  await expect(
    page.getByText("Identifying speakers locally", { exact: true }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(
    page.getByText("These corrected meeting notes should stay unchanged."),
  ).toBeVisible();
  await expect(rerun).toBeEnabled();
  await rerun.click();
  await expect(rerun).toBeDisabled();
});

test("offers speaker recognition alongside a small speech model", async ({
  page,
}) => {
  await page.goto("/settings?transcript=1#models");
  for (const name of ["Fast", "Nemotron 3 speaker recognition"]) {
    const model = page.getByRole("group", { name, exact: true });
    await model.getByRole("button", { name: "Download", exact: true }).click();
    await expect(model.getByText("Installed", { exact: true })).toBeVisible();
  }
  await page.getByRole("link", { name: "Home", exact: true }).click();
  await page.getByText("Weekly product sync", { exact: true }).click();
  const option = page.getByRole("checkbox", {
    name: "Identify speakers with Nemotron 3",
  });
  await expect(option).not.toBeChecked();
  for (const width of [1200, 900, 800]) {
    await page.setViewportSize({ width, height: 850 });
    await expect
      .poll(() =>
        page.locator(".session-header").evaluate((header) => {
          const bounds = header.getBoundingClientRect();
          const metadata = header
            .querySelector(".session-header__metadata")!
            .getBoundingClientRect();
          const controls = header
            .querySelector(".session-header__controls")!
            .getBoundingClientRect();
          const separated =
            metadata.bottom <= controls.top || metadata.right <= controls.left;
          return (
            separated &&
            controls.right <= bounds.right &&
            header.scrollWidth <= header.clientWidth
          );
        }),
      )
      .toBe(true);
  }
  await option.check();
  await page.getByRole("button", { name: "Retranscribe", exact: true }).click();
  await expect(option).toBeDisabled();
  await expect(
    page.getByRole("button", { name: "Re-identify speakers", exact: true }),
  ).toBeDisabled();
});
