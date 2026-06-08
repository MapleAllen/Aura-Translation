import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

describe('tauri macOS config', () => {
  it('limits the adaptation build to the app bundle on macOS', () => {
    const file = join(process.cwd(), 'src-tauri', 'tauri.macos.conf.json');
    const config = JSON.parse(readFileSync(file, 'utf8'));

    expect(config.bundle?.targets).toEqual(['app']);
  });
});
