export interface ConfirmOptions {
  title: string;
  message: string[];
  confirm: string;
  cancel: string;
}

function createButton(className: string, label: string) {
  const button = document.createElement('button');
  button.type = 'button';
  button.className = `secondary-button ${className}`;
  button.textContent = label;
  return button;
}

/** Ask before an action that cannot be undone; dismissing the dialog declines it. */
export function confirmAction(root: HTMLElement, options: ConfirmOptions) {
  const dialog = document.createElement('dialog');
  dialog.className = 'community-dialog confirm-dialog';
  dialog.setAttribute('aria-labelledby', 'confirm-dialog-title');
  const title = document.createElement('h2');
  title.id = 'confirm-dialog-title';
  title.textContent = options.title;
  const header = document.createElement('div');
  header.className = 'community-dialog-header';
  header.append(title);
  const content = document.createElement('div');
  content.className = 'confirm-dialog-content';
  content.append(
    ...options.message.map((line) => {
      const paragraph = document.createElement('p');
      paragraph.textContent = line;
      return paragraph;
    }),
  );
  const cancel = createButton('confirm-dialog-cancel', options.cancel);
  const accept = createButton('confirm-dialog-accept', options.confirm);
  const actions = document.createElement('div');
  actions.className = 'confirm-dialog-actions';
  actions.append(cancel, accept);
  dialog.append(header, content, actions);

  return new Promise<boolean>((resolve) => {
    let confirmed = false;
    accept.addEventListener('click', () => {
      confirmed = true;
      dialog.close();
    });
    cancel.addEventListener('click', () => dialog.close());
    dialog.addEventListener('close', () => {
      dialog.remove();
      resolve(confirmed);
    });
    root.append(dialog);
    dialog.showModal();
  });
}
