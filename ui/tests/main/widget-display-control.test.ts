/// @vitest-environment happy-dom

import { describe, expect, it, vi } from 'vite-plus/test';

import { createWidgetDisplayControl } from '../../src/main/widget-display-control';
import type { DisplayInfo } from '../../src/types/rust';

const labels = { automatic: 'Follow default', primary: 'Primary', disconnected: 'Disconnected' };

function display(name: string, primary = false): DisplayInfo {
  return { name, label: name, primary, x: 0, y: 0, width: 1920, height: 1080, scale: 1 };
}

const two = [display('Monitor #1', true), display('Monitor #2')];

describe('widget display control', () => {
  it('stays out of the way on a single display', () => {
    const control = createWidgetDisplayControl(() => {});

    control.update(null, [display('Monitor #1', true)], labels, 'Display');

    expect(control.root.hidden).toBe(true);
  });

  it('stays available for a widget still pinned to a display that left', () => {
    const control = createWidgetDisplayControl(() => {});

    control.update('Monitor #2', [display('Monitor #1', true)], labels, 'Display');

    expect(control.root.hidden).toBe(false);
  });

  it('marks a widget that has its own display', () => {
    const control = createWidgetDisplayControl(() => {});

    control.update('Monitor #2', two, labels, 'Display');

    expect(control.root.dataset.pinned).toBe('true');
    control.update(null, two, labels, 'Display');
    expect(control.root.dataset.pinned).toBe('false');
  });

  it('reports the chosen display, and null for following the default', () => {
    const onChange = vi.fn();
    const control = createWidgetDisplayControl(onChange);
    control.update(null, two, labels, 'Display');
    const select = control.root.querySelector('select')!;

    select.value = 'Monitor #2';
    select.dispatchEvent(new Event('change'));
    select.value = '';
    select.dispatchEvent(new Event('change'));

    expect(onChange.mock.calls).toEqual([['Monitor #2'], [null]]);
  });
});
