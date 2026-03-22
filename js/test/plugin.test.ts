import { describe, it } from 'node:test';
import assert from 'node:assert/strict';
import { definePlugin } from '../src/plugin.ts';

describe('definePlugin', () => {
  it('returns the definition unchanged (identity)', () => {
    const def = {
      meta: { name: 'my-plugin' },
      createOnce() { return {}; },
    };
    const result = definePlugin(def);
    assert.strictEqual(result, def);
  });

  it('throws if meta.name is missing', () => {
    assert.throws(
      () => definePlugin({ meta: {} as { name: string }, createOnce() { return {}; } }),
      /meta\.name/,
    );
  });

  it('throws if meta is absent entirely', () => {
    assert.throws(
      () => definePlugin({ createOnce() { return {}; } } as Parameters<typeof definePlugin>[0]),
      /meta\.name/,
    );
  });

  it('throws if createOnce is missing', () => {
    assert.throws(
      () => definePlugin({ meta: { name: 'x' } } as Parameters<typeof definePlugin>[0]),
      /createOnce/,
    );
  });

  it('throws if createOnce is not a function', () => {
    assert.throws(
      () => definePlugin({ meta: { name: 'x' }, createOnce: 42 } as unknown as Parameters<typeof definePlugin>[0]),
      /createOnce/,
    );
  });

  it('works with a minimal valid definition', () => {
    const def = definePlugin({
      meta: { name: 'minimal' },
      createOnce() { return {}; },
    });
    assert.equal(def.meta.name, 'minimal');
    assert.equal(typeof def.createOnce, 'function');
  });

  it('preserves optional meta fields', () => {
    const def = definePlugin({
      meta: { name: 'full', version: '1.0.0', description: 'A plugin' },
      createOnce() { return {}; },
    });
    assert.equal(def.meta.version, '1.0.0');
    assert.equal(def.meta.description, 'A plugin');
  });
});
