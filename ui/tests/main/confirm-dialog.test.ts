/// @vitest-environment happy-dom

import { beforeEach, describe, expect, it, vi } from 'vite-plus/test';

import { confirmAction } from '../../src/main/confirm-dialog';

const options = {
  title: 'Move all widgets?',
  message: ['3 widgets will move.', '1 widget follows the default again.'],
  confirm: 'Move all',
  cancel: 'Cancel',
};

function buttons() {
  return [...document.querySelectorAll<HTMLButtonElement>('dialog button')];
}

beforeEach(() => {
  document.body.replaceChildren();
  vi.spyOn(HTMLDialogElement.prototype, 'showModal').mockImplementation(function (
    this: HTMLDialogElement,
  ) {
    this.open = true;
  });
  vi.spyOn(HTMLDialogElement.prototype, 'close').mockImplementation(function (
    this: HTMLDialogElement,
  ) {
    this.open = false;
    this.dispatchEvent(new Event('close'));
  });
});

describe('confirm dialog', () => {
  it('shows the question and every line of the message', () => {
    void confirmAction(document.body, options);

    const dialog = document.querySelector('dialog')!;
    expect(dialog.open).toBe(true);
    expect(dialog.textContent).toContain('Move all widgets?');
    expect(dialog.textContent).toContain('3 widgets will move.');
    expect(dialog.textContent).toContain('1 widget follows the default again.');
  });

  it('resolves true only when confirmed, and removes itself', async () => {
    const answer = confirmAction(document.body, options);

    buttons()
      .find((button) => button.textContent === 'Move all')!
      .click();

    await expect(answer).resolves.toBe(true);
    expect(document.querySelector('dialog')).toBeNull();
  });

  it('resolves false when cancelled', async () => {
    const answer = confirmAction(document.body, options);

    buttons()
      .find((button) => button.textContent === 'Cancel')!
      .click();

    await expect(answer).resolves.toBe(false);
  });

  it('resolves false when dismissed with Escape', async () => {
    const answer = confirmAction(document.body, options);

    document.querySelector('dialog')!.close();

    await expect(answer).resolves.toBe(false);
  });
});
