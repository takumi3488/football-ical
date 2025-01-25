import { expect, test } from '@playwright/test';

const origin = process.env.TARGET_ORIGIN as string;
const teams = [
  { url: "https://soccer.yahoo.co.jp/jleague/team/136", name: "ヴィッセル神戸" },
  { url: "https://soccer.yahoo.co.jp/ws/category/eng/teams/4075/info?gk=52", name: "リヴァプール" },
  { url: "https://soccer.yahoo.co.jp/japan/category/men/teams/142/schedule", name: "日本" },
]

test('got health check', async ({ page }) => {
  await page.goto(origin);

  for (const team of teams) {
    await page.fill('input[name="url"]', team.url);
    await page.click('button[type="submit"]');
    await page.waitForSelector(`tr:has-text("${team.name}")`);
    const text = await (await page.waitForSelector('input[name="url"]')).textContent();
    expect(text).toBe("");
    await page.check(`tr:has-text("${team.name}") input[type="checkbox"]`);
    expect(await page.isChecked(`tr:has-text("${team.name}") input[type="checkbox"]`)).toBe(true);
    await page.reload();
    await page.waitForSelector(`tr:has-text("${team.name}") input[type="checkbox"]`);
    await page.click(`tr:has-text("${team.name}") input[type="checkbox"]`);
    expect(await page.isChecked(`tr:has-text("${team.name}") input[type="checkbox"]`)).toBe(false);
    await page.reload();
    await page.waitForSelector(`tr:has-text("${team.name}") input[type="checkbox"]`);
    expect(await page.isChecked(`tr:has-text("${team.name}") input[type="checkbox"]`)).toBe(false);
  }
});
