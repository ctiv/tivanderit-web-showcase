import { test, expect } from "@playwright/test";

test("homepage has title", async ({ page }) => {
  await page.goto("/");

  // Verify the title
  await expect(page).toHaveTitle("Tivander IT");

  // Verify the main heading is visible
  await expect(page.getByRole('heading', { name: 'Omvandlar idéer till digital verklighet' })).toBeVisible();

  // Verify that the navigation links are present
  await expect(page.getByRole('navigation').getByRole('link', { name: 'Vårt uppdrag' })).toBeVisible();
  await expect(page.getByRole('navigation').getByRole('link', { name: 'Vad vi erbjuder' })).toBeVisible();
  await expect(page.getByRole('navigation').getByRole('link', { name: 'Kontakta oss' })).toBeVisible();});
