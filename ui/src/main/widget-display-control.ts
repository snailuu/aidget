import { createElement, Monitor } from 'lucide';

import type { DisplayInfo } from '../types/rust';
import { renderDisplaySelect, selectedDisplay, type DisplaySelectLabels } from './display-select';

/** The monitor icon on a widget card; a transparent select gives it the native menu. */
export function createWidgetDisplayControl(onChange: (display: string | null) => void) {
  const root = document.createElement('label');
  root.className = 'widget-icon-button widget-display-button';
  const select = document.createElement('select');
  select.addEventListener('change', () => onChange(selectedDisplay(select)));
  root.append(
    createElement(Monitor, { width: 16, height: 16, 'stroke-width': 1.8, 'aria-hidden': 'true' }),
    select,
  );

  return {
    root,
    update(
      pinned: string | null,
      displays: DisplayInfo[],
      labels: DisplaySelectLabels,
      title: string,
    ) {
      root.hidden = displays.length < 2 && pinned === null;
      root.dataset.pinned = String(pinned !== null);
      root.title = title;
      select.setAttribute('aria-label', title);
      renderDisplaySelect(select, displays, pinned, labels);
    },
  };
}
