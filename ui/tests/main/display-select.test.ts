/// @vitest-environment happy-dom

import { describe, expect, it } from 'vite-plus/test';

import { findDisplay, renderDisplaySelect, selectedDisplay } from '../../src/main/display-select';
import type { DisplayInfo } from '../../src/types/rust';

const labels = { automatic: 'Automatic', primary: 'Primary', disconnected: 'Disconnected' };
const builtIn: DisplayInfo = {
  name: 'Monitor #1',
  label: 'Built-in Retina Display',
  primary: true,
  x: 0,
  y: 33,
  width: 1470,
  height: 859,
  scale: 2,
};
const external: DisplayInfo = {
  name: 'Monitor #2',
  label: 'U27P10',
  primary: false,
  x: -719,
  y: -1410,
  width: 2560,
  height: 1410,
  scale: 2,
};

function optionTexts(select: HTMLSelectElement) {
  return [...select.options].map((option) => option.textContent);
}

describe('display select', () => {
  it('offers automatic placement and every connected display by its system name', () => {
    const select = document.createElement('select');

    renderDisplaySelect(select, [builtIn, external], null, labels);

    expect(optionTexts(select)).toEqual([
      'Automatic',
      'Built-in Retina Display · Primary',
      'U27P10',
    ]);
    expect(selectedDisplay(select)).toBeNull();
  });

  it('adds the size when the system has no name to tell displays apart', () => {
    const select = document.createElement('select');
    const unnamed = { ...external, label: external.name };

    renderDisplaySelect(select, [unnamed], null, labels);

    expect(optionTexts(select)[1]).toBe('Monitor #2 · 2560 × 1410');
  });

  it('stores the stable name while showing the system name', () => {
    const select = document.createElement('select');

    renderDisplaySelect(select, [builtIn, external], 'Monitor #2', labels);

    expect(selectedDisplay(select)).toBe('Monitor #2');
  });

  it('selects the primary display for a frame stored as primary', () => {
    const select = document.createElement('select');

    renderDisplaySelect(select, [builtIn, external], 'primary', labels);

    expect(selectedDisplay(select)).toBe('Monitor #1');
  });

  it('exposes the selected label so a truncated menu can still be read', () => {
    const select = document.createElement('select');

    renderDisplaySelect(select, [builtIn, external], 'Monitor #1', labels);

    expect(select.title).toBe('Built-in Retina Display · Primary');
  });

  it('keeps a disconnected saved display selected instead of showing automatic', () => {
    const select = document.createElement('select');

    renderDisplaySelect(select, [builtIn], 'Monitor #2', labels);

    expect(optionTexts(select).at(-1)).toBe('Monitor #2 · Disconnected');
    expect(selectedDisplay(select)).toBe('Monitor #2');
  });

  it('keeps the option elements when nothing changed so an open dropdown stays open', () => {
    const select = document.createElement('select');
    renderDisplaySelect(select, [builtIn, external], null, labels);
    const option = select.options[1];

    renderDisplaySelect(select, [builtIn, external], null, labels);

    expect(select.options[1]).toBe(option);
  });

  it('rebuilds the options when a display is connected', () => {
    const select = document.createElement('select');
    renderDisplaySelect(select, [builtIn], null, labels);

    renderDisplaySelect(select, [builtIn, external], null, labels);

    expect(select.options).toHaveLength(3);
  });

  it('finds the primary display from the name frames store', () => {
    expect(findDisplay([external, builtIn], 'primary')).toBe(builtIn);
    expect(findDisplay([external, builtIn], 'Monitor #2')).toBe(external);
    expect(findDisplay([external, builtIn], 'Monitor #9')).toBeUndefined();
  });

  it('falls back to the first display when the system reports no primary', () => {
    expect(findDisplay([external], 'primary')).toBe(external);
    expect(findDisplay([], 'primary')).toBeUndefined();
  });
});
