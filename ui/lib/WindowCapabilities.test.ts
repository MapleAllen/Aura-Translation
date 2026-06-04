import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { describe, expect, it } from 'vitest';

type CapabilityFile = {
  permissions: string[];
};

function readDefaultCapability(): CapabilityFile {
  return JSON.parse(
    readFileSync(resolve(process.cwd(), 'src-tauri/capabilities/default.json'), 'utf8'),
  ) as CapabilityFile;
}

describe('Tauri window capabilities', () => {
  it('allows the settings close button to close its window', () => {
    const capability = readDefaultCapability();

    expect(capability.permissions).toContain('core:window:allow-close');
  });
});
