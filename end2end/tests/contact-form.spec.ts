import { test, expect, type Page } from '@playwright/test';

const CONTACT_PAGE_PATH = '/#contact';

const locators = (page: Page) => ({
  messageInput: page.locator('#message'),
  emailInput: page.locator('#email'),
  termsCheckbox: page.locator('#terms'),
  submitButton: page.getByTestId('contact-form-submit'),
  successMessage: page.locator('.success-message'),
  form: page.getByTestId('contact-form'),
  messageError: page.locator('#message-error'),
  emailError: page.locator('#email-error'),
  termsError: page.locator('#terms-error'),
  generalError: page.locator('p.error-message.general-error'), 
  messageHint: page.locator('div.form-field:has(#message) p.hint-message'), 
  emailHint: page.locator('div.form-field:has(#email) p.hint-message'), 
});

async function fillForm(locator: ReturnType<typeof locators>, options: {message?:string; email?:string; terms?:boolean} = {}) {
    const {
      message = 'Detta är ett giltigt testmeddelande.',
      email = 'valid.user@example.com',
      terms = true,
    } = options;

    await locator.termsCheckbox.setChecked(terms);
    await locator.messageInput.clear();
    await locator.messageInput.pressSequentially(message);
    await locator.emailInput.clear();
    await locator.emailInput.pressSequentially(email);
    await locator.emailInput.blur();
}

let locator: ReturnType<typeof locators>;

test.beforeEach(async ({ page }) => {
  locator = locators(page);

  // Navigate to form and wait for it to render.
  await page.goto(CONTACT_PAGE_PATH);
  await expect(locator.messageInput).toBeVisible();
});

test.describe('Contact Form Functionality', () => {

  test.beforeEach(async ({ page }) => {
    // Validate that wasm is loaded and running.
    await expect(locator.form).toBeAttached();
  });

  test('renders correctly initially', async ({ page }) => {
    await expect(locator.emailInput).toBeVisible();
    await expect(locator.termsCheckbox).toBeVisible();
    await expect(locator.submitButton).toBeVisible();
    await expect(locator.submitButton).toBeDisabled();
    await expect(locator.successMessage).toBeHidden();
  });

  test('submits successfully with valid data', async ({ page }) => {
    await fillForm(locator);

    // Verify that the button is now enabled
    let submitButton = locator.submitButton;
    await expect(submitButton).toBeEnabled();

    // Click submit
    await submitButton.click();

    // Verify that the button is disabled
    await expect(submitButton).toBeDisabled();

    // Verify that the success message is displayed
    const successMessage = locator.successMessage;
    await expect(successMessage).toBeVisible();
    await expect(successMessage).toHaveText(
      'Ditt meddelande är mottaget. Vi återkopplar snart.'
    );

    // Verify that the form has been cleared and remains visible,
    await expect(locator.form).toBeVisible();
    await expect(locator.messageInput).toHaveValue('');
    await expect(locator.emailInput).toHaveValue('');
    await expect(locator.termsCheckbox).not.toBeChecked();  });

 test.describe('Client-Side Validation', () => {
    test('disables submit button if message is empty', async () => {
      await fillForm(locator, { message: '' });
      await expect(locator.submitButton).toBeDisabled();
    });

    test('disables submit button if email is invalid', async () => {
      await fillForm(locator, { email: 'invalid-email' });
      await expect(locator.submitButton).toBeDisabled();
    });

    test('disables submit button if terms are not accepted', async () => {
      await fillForm(locator, { terms: false });
      await expect(locator.submitButton).toBeDisabled();
    });

    test('shows hint for invalid email', async () => {
      await fillForm(locator, { email: 'invalid' });

      await expect(locator.emailHint).toBeVisible();
      await expect(locator.emailHint).toHaveText('Ange en giltig email.');
      await expect(locator.emailError).toBeHidden();

      await fillForm(locator);
      await expect(locator.emailHint).toBeHidden();
    });

    test('shows hint for empty message', async () => {
      await fillForm(locator, { message: '' });

      await expect(locator.messageHint).toBeVisible();
      await expect(locator.messageHint).toHaveText('Meddelandet får inte vara tomt.');
      await expect(locator.emailError).toBeHidden();

      await fillForm(locator);
      await expect(locator.messageHint).toBeHidden();
    });
  });
 
  test.describe('Server-Side Validation', () => {
    test('shows general error for message too long', async () => {
      const longMessage = 'a'.repeat(5001);
      await fillForm(locator, { message: longMessage });
      await expect(locator.submitButton).toBeEnabled();

      await locator.submitButton.click();

      await expect(locator.messageError).toBeVisible(); // Changed from generalError
      await expect(locator.messageError).toHaveText('Meddelandet är för långt (max 5000 tecken).'); // Changed from generalError
      await expect(locator.successMessage).toBeHidden();
    });

    test('shows general error for email too long', async () => {
      const longEmail = 'a'.repeat(245) + '@example.com';
      await fillForm(locator, { email: longEmail });
      await expect(locator.submitButton).toBeEnabled();

      await locator.submitButton.click();
      
      await expect(locator.emailError).toBeVisible(); // Changed from generalError
      await expect(locator.emailError).toHaveText('E-postadressen är för lång (max 254 tecken).'); // Changed from generalError
      await expect(locator.successMessage).toBeHidden();
    });
  });
});

// --- Test cases with JavaScript Disabled ---
test.describe('Contact Form Functionality (JavaScript Disabled)', () => {
  test.use({ javaScriptEnabled: false });

  test('submits successfully with valid data (JS disabled)', async ({ page }) => {
    await fillForm(locator);
    await locator.submitButton.click();

    // Wait for full page navigation/reload after submit
    await page.waitForLoadState('domcontentloaded');

    // Verify that the success message is displayed
    const successMessage = locator.successMessage;
    await expect(successMessage).toBeVisible();
    await expect(successMessage).toHaveText(
      'Ditt meddelande är mottaget. Vi återkopplar snart.'
    );
  });

  test('shows server-side validation error for missing email (JS disabled)', async ({
    page,
  }) => {
    await fillForm(locator, { email: "" });
    await locator.submitButton.click();

    // Wait for full page navigation/reload after submit
    await page.waitForLoadState('domcontentloaded');

    // Verify error message for email
    const emailError = locator.emailError;
    await expect(emailError).toBeVisible();
    await expect(emailError).toHaveText('Ange en email-adress.');

    // Verify that input has error class
    await expect(locator.emailInput).toHaveClass(/error/);

    // Verify that success is *not* displayed
    await expect(locator.successMessage).toBeHidden();
  });
});
