export class DataMap {
  readonly #map = new Map<number, Record<string, unknown>>();

  get(nodeId: number): Record<string, unknown> | null {
    return this.#map.get(nodeId) ?? null;
  }

  set(nodeId: number, value: Record<string, unknown>): void {
    this.#map.set(nodeId, value);
  }

  merge(nodeId: number, value: Record<string, unknown>): void {
    const existing = this.#map.get(nodeId);
    this.#map.set(nodeId, existing ? { ...existing, ...value } : { ...value });
  }

  has(nodeId: number): boolean {
    return this.#map.has(nodeId);
  }

  delete(nodeId: number): void {
    this.#map.delete(nodeId);
  }

  clear(): void {
    this.#map.clear();
  }

  get size(): number {
    return this.#map.size;
  }
}
