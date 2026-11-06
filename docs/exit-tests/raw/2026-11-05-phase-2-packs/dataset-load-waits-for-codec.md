# Context pack

Task: Explorer and catalog pages send their first command (dataset.load) as soon as the session synchronises, but the protobuf codec bundle that encodes it registers later, so the transport falls back to JSON and the server refuses it. Make the browser bus hold a command until the codec that encodes it has registered, and make the pages' inline scripts rely on that instead of racing.

## Seeds

- `typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::registerActionCodec` Method 0.90 (text)
- `typescript:web/ui-explorer-codecs/src/index.ts::registerExplorerCodecs` Function 0.83 (text)
- `typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::registerCatalogCodecs` Function 0.71 (text)
- `typescript:torrentfx/web/framework/runtime-web/src/index.ts::ActionCodec` TypeAlias 0.67 (text)
- `typescript:web/ui-explorer-codecs/src/index.ts::encodeExplorerCommand` Function 0.61 (text)
- `typescript:web/ui-explorer-codecs/src/index.ts::CodecBus::registerActionCodec` Method 0.56 (text)
- `typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::encodeCatalogCommand` Function 0.56 (text)
- `typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::actionCodec` Method 0.56 (text)
- `typescript:web/ui-explorer-codecs/src/index.test.ts::encodeExplorerCommand round-trip::falls back to the custom arm for an unknown action` Test 0.54 (text)
- `typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::CodecBus::registerActionCodec` Method 0.54 (text)

Budget 30000 tokens, used 29956. Seeds 10, candidates 259, included 92, left out 167.

## Context

### 1. Function registerExplorerCodecs
`typescript:web/ui-explorer-codecs/src/index.ts::registerExplorerCodecs`  web/ui-explorer-codecs/src/index.ts:112-116  pkg:npm/@causeway/ui-explorer-codecs  score 2.79  
`function registerExplorerCodecs(bus: CodecBus): void`  
Register the explorer command codecs on the bus. Idempotent: `registerActionCodec` is keyed by action name, so calling this twice (or after a transport reconnect) re-installs the same encoders without duplicating or throwing.  
callers 2: `registerExplorerCodecs::registers a codec for every action and is idempotent` (web/ui-explorer-codecs/src/index.test.ts:118), `registerExplorerCodecs::registers a codec for every action and is idempotent` (web/ui-explorer-codecs/src/index.test.ts:123)  
calls 2: `encodeExplorerCommand` (web/ui-explorer-codecs/src/index.ts:114), `CodecBus::registerActionCodec` (web/ui-explorer-codecs/src/index.ts:114)  
importers 1: `web/ui-explorer-codecs/src/index.test.ts` (web/ui-explorer-codecs/src/index.test.ts:14)  
tests 1: `registerExplorerCodecs::registers a codec for every action and is idempotent` (web/ui-explorer-codecs/src/index.test.ts:118)  
uses_types 1: `CodecBus` (web/ui-explorer-codecs/src/index.ts:112)  
web/ui-explorer-codecs/src/index.ts:112-116
```typescript
export function registerExplorerCodecs(bus: CodecBus): void {
  for (const action of EXPLORER_ACTIONS) {
    bus.registerActionCodec(action, (payload) => encodeExplorerCommand(action, payload));
  }
}
```

### 2. Method BrowserBus::registerActionCodec
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::registerActionCodec`  torrentfx/web/framework/runtime-web/src/index.ts:660-662  pkg:npm/@torrentfx/runtime-web  score 2.18  
`registerActionCodec(action: string, codec: ActionCodec): void`  
Register a payload codec for an action. The transport then sends the codec's bytes as the frame payload instead of JSON — used by slices that put a binary protobuf command on the wire. Stored on the bus so it survives transport reconnects.  
calls 1: `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/index.ts:661)  
uses_types 1: `ActionCodec` (torrentfx/web/framework/runtime-web/src/index.ts:660)  
torrentfx/web/framework/runtime-web/src/index.ts:660-662
```typescript
  registerActionCodec(action: string, codec: ActionCodec): void {
    this.#actionCodecs.set(action, codec);
  }
```

### 3. Function encodeCatalogCommand
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::encodeCatalogCommand`  bindings-ts/causeway-catalog-ui/src/codec.ts:31-80  pkg:npm/@causeway/catalog-ui  score 2.00  
`function encodeCatalogCommand(action: string, payload: unknown): Uint8Array`  
Encode a `CatalogCommand` proto for the given catalog action + payload.  
callers 3: `registerCatalogCodecs` (bindings-ts/causeway-catalog-ui/src/codec.ts:89), `encodeCatalogCommand round-trip::throws for an action it does not handle` (web/ui-catalog-status/src/codec.test.ts:76), `decode` (web/ui-catalog-status/src/codec.test.ts:21)  
calls 10: `str` (bindings-ts/causeway-catalog-ui/src/codec.ts:70), `big` (bindings-ts/causeway-catalog-ui/src/codec.ts:48), `str` (bindings-ts/causeway-catalog-ui/src/codec.ts:71) … 7 more  
tests 1: `encodeCatalogCommand round-trip::throws for an action it does not handle` (web/ui-catalog-status/src/codec.test.ts:76)  
bindings-ts/causeway-catalog-ui/src/codec.ts:31-80
```typescript
export function encodeCatalogCommand(action: string, payload: unknown): Uint8Array {
  switch (action) {
    case "catalog.init":
      return toBinary(
        CatalogCommandSchema,
        create(CatalogCommandSchema, { command: { case: "init", value: {} } }),
      );
    case "catalog.get-block": {
      const p = (payload ?? {}) as BlockPayload;
      const sort = p.sortModel?.[0];
      return toBinary(
        CatalogCommandSchema,
        create(CatalogCommandSchema, {
          command: {
            case: "getBlock",
            value: {
              startRow: big(p["start-row"]),
              endRow: big(p["end-row"]),
              requestId: str(p["request-id"]),
              generation: big(p.generation),
              sort:
                sort && str(sort.column).length > 0
                  ? { column: str(sort.column), descending: Boolean(sort.descending) }
                  : undefined,
            },
          },
        }),
      );
    }
    case "catalog.filter": {
      const p = (payload ?? {}) as FilterPayload;
      return toBinary(
        CatalogCommandSchema,
        create(CatalogCommandSchema, {
          command: {
            case: "applyFilter",
            value: {
              q: str(p.q),
              domainL1: str(p.domain_l1),
              domainL2: str(p.domain_l2),
              assetType: str(p.asset_type),
            },
          },
        }),
      );
    }
    default:
      throw new Error(`no catalog codec for action ${action}`);
  }
}
```

### 4. Function registerCatalogCodecs
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::registerCatalogCodecs`  bindings-ts/causeway-catalog-ui/src/codec.ts:87-91  pkg:npm/@causeway/catalog-ui  score 1.98  
`function registerCatalogCodecs(bus: CodecBus): void`  
Register the catalog command codecs on the bus (idempotent).  
callers 1: `boot` (web/ui-catalog-filter/src/index.ts:103)  
calls 2: `encodeCatalogCommand` (bindings-ts/causeway-catalog-ui/src/codec.ts:89), `CodecBus::registerActionCodec` (bindings-ts/causeway-catalog-ui/src/codec.ts:89)  
uses_types 1: `CodecBus` (bindings-ts/causeway-catalog-ui/src/codec.ts:87)  
bindings-ts/causeway-catalog-ui/src/codec.ts:87-91
```typescript
export function registerCatalogCodecs(bus: CodecBus): void {
  for (const action of ["catalog.init", "catalog.get-block", "catalog.filter"]) {
    bus.registerActionCodec(action, (payload) => encodeCatalogCommand(action, payload));
  }
}
```

### 5. Function encodeExplorerCommand
`typescript:web/ui-explorer-codecs/src/index.ts::encodeExplorerCommand`  web/ui-explorer-codecs/src/index.ts:80-83  pkg:npm/@causeway/ui-explorer-codecs  score 1.77  
`function encodeExplorerCommand(action: string, payload: unknown): Uint8Array`  
Encode an `ExplorerCommand` proto for the given explorer action + payload.  
callers 2: `registerExplorerCodecs` (web/ui-explorer-codecs/src/index.ts:114), `decode` (web/ui-explorer-codecs/src/index.test.ts:38)  
calls 1: `commandFor` (web/ui-explorer-codecs/src/index.ts:81)  
importers 1: `web/ui-explorer-codecs/src/index.test.ts` (web/ui-explorer-codecs/src/index.test.ts:14)  
web/ui-explorer-codecs/src/index.ts:80-83
```typescript
export function encodeExplorerCommand(action: string, payload: unknown): Uint8Array {
  const message = create(ExplorerCommandSchema, { command: commandFor(action, payload) });
  return toBinary(ExplorerCommandSchema, message);
}
```

### 6. TypeAlias ActionCodec
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::ActionCodec`  torrentfx/web/framework/runtime-web/src/index.ts:405  pkg:npm/@torrentfx/runtime-web  score 1.45  
`type ActionCodec = (payload: unknown) => Uint8Array;`  
Encodes a dispatch payload to wire bytes for a specific action. When an action has a registered codec, the transport sends those bytes verbatim as the frame payload instead of JSON — the hook a slice uses to put a binary protobuf command on the wire with no JSON fallback.  
used_by 1: `BrowserBus::registerActionCodec` (torrentfx/web/framework/runtime-web/src/index.ts:660)  
torrentfx/web/framework/runtime-web/src/index.ts:405-405
```typescript
export type ActionCodec = (payload: unknown) => Uint8Array;
```

### 7. Method CodecBus::registerActionCodec
`typescript:web/ui-explorer-codecs/src/index.ts::CodecBus::registerActionCodec`  web/ui-explorer-codecs/src/index.ts:103  pkg:npm/@causeway/ui-explorer-codecs  score 1.34  
`registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void`  
callers 1: `registerExplorerCodecs` (web/ui-explorer-codecs/src/index.ts:114)  
web/ui-explorer-codecs/src/index.ts:103-103
```typescript
  registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void;
```

### 8. File web/ui-explorer-codecs/src/index.ts
`file:web/ui-explorer-codecs/src/index.ts`  web/ui-explorer-codecs/src/index.ts  pkg:npm/@causeway/ui-explorer-codecs  score 1.30  via `registerExplorerCodecs` <- CONTAINS (web/ui-explorer-codecs/src/index.ts:112-116, 1.00)  
members 8  
calls 1: `whenBusReady` (web/ui-explorer-codecs/src/index.ts:122)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:647)  
importers 1: `web/ui-explorer-codecs/src/index.test.ts` (web/ui-explorer-codecs/src/index.test.ts:14)  
imports 3: `npm/@bufbuild/protobuf` (web/ui-explorer-codecs/src/index.ts:14), `npm/@causeway/explorer-ui` (web/ui-explorer-codecs/src/index.ts:15), `npm/@torrentfx/runtime-web` (web/ui-explorer-codecs/src/index.ts:16)  

### 9. Function str
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::str`  bindings-ts/causeway-catalog-ui/src/codec.ts:26  pkg:npm/@causeway/catalog-ui  score 1.27  via `encodeCatalogCommand` -> CALLS (bindings-ts/causeway-catalog-ui/src/codec.ts:71, 1.00)  
`str = (value: unknown): string => (typeof value === "string" ? value : "")`  
callers 7: `encodeCatalogCommand` (bindings-ts/causeway-catalog-ui/src/codec.ts:70), `encodeCatalogCommand` (bindings-ts/causeway-catalog-ui/src/codec.ts:71), `encodeCatalogCommand` (bindings-ts/causeway-catalog-ui/src/codec.ts:53) … 4 more  
bindings-ts/causeway-catalog-ui/src/codec.ts:26-26
```typescript
const str = (value: unknown): string => (typeof value === "string" ? value : "");
```

### 10. Method CodecBus::registerActionCodec
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::CodecBus::registerActionCodec`  bindings-ts/causeway-catalog-ui/src/codec.ts:83  pkg:npm/@causeway/catalog-ui  score 1.23  
`registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void`  
callers 1: `registerCatalogCodecs` (bindings-ts/causeway-catalog-ui/src/codec.ts:89)  
bindings-ts/causeway-catalog-ui/src/codec.ts:83-83
```typescript
  registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void;
```

### 11. Class BrowserBus
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus`  torrentfx/web/framework/runtime-web/src/index.ts:529-1127  pkg:npm/@torrentfx/runtime-web  score 1.16  via `BrowserBus::registerActionCodec` <- DEFINES (torrentfx/web/framework/runtime-web/src/index.ts:660-662, 1.00)  
`class BrowserBus`  
members 31  
calls 8: `BrowserBus::rescan` (torrentfx/web/framework/runtime-web/src/index.ts:955), `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:599), `disposeElementBindings` (torrentfx/web/framework/runtime-web/src/index.ts:947) … 5 more  
importers 7: `torrentfx/web/framework/runtime-web/src/effect-connect.ts` (torrentfx/web/framework/runtime-web/src/effect-connect.ts:27), `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:22-29), `torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:17-22) … 4 more  
used_by 44: `BrowserWebSocketTransport::constructor` (torrentfx/web/framework/runtime-web/src/index.ts:1193), `publishBusReady` (torrentfx/web/framework/runtime-web/src/index.ts:2027), `recordingBus` (torrentfx/web/framework/app/src/post-message.test.ts:22) … 41 more  
uses_types 1: `BusEvent` (torrentfx/web/framework/runtime-web/src/index.ts:568)  
torrentfx/web/framework/runtime-web/src/index.ts:529-608, cut
```typescript
export class BrowserBus {
  readonly #store = new Map<string, unknown>();
  readonly #subscriptions = new Map<string, Map<number, Subscription>>();
  readonly #handlers = new Map<string, Map<number, ActionHandler>>();
  readonly #synchronizedCallbacks = new Set<() => void>();
  readonly #scanners: Scanner[] = [];
  readonly #taps = new Set<Tap>();
  readonly #pending = new Map<string, PendingInvoke>();
  readonly #actionCodecs = new Map<string, ActionCodec>();
  readonly #domainEventSubscribers = new Map<string, Map<number, DomainEventSubscriber>>();
  readonly #domainStreamReferences = new Map<string, number>();
  #transport: TransportLike | undefined;
  #nextId = 1;
  #synchronized = false;
  #observerInstalled = false;
  #domObserver: MutationObserver | undefined;

  constructor(config: { hydrationElementId?: string } = {}) {
    this.#hydrate(config.hydrationElementId ?? "torrent-hydration");
  }

  /**
   * Subscribe to every observable bus event — sets, ingests,
   * dispatches, handle interceptions, island lifecycle transitions.
   *
   * Single chokepoint for diagnostic overlays, journal/replay UIs,
   * and OTel exporters. A misbehaving tap can't take down the bus
   * (errors are caught and logged); a tap that detaches itself
   * during dispatch doesn't break iteration (a snapshot is taken).
   *
   * Phase 4 of EVOLVE-BROWSERBUS.
   */
  tap(callback: Tap): () => void {
    this.#taps.add(callback);
    return () => {
      this.#taps.delete(callback);
    };
  }

  #emit(event: BusEvent): void {
    if (this.#taps.size === 0) return;
    for (const cb of [...this.#taps]) {
      try {
        cb(event);
      } catch (e) {
        console.warn("[torrentfx] tap threw", e);
      }
    }
  }

  get(name: string): unknown {
    return this.#store.get(name);
  }

  getAll(): Record<string, unknown> {
    return Object.fromEntries(this.#store.entries());
  }

  set(name: string, value: unknown): void {
    this.#emit({ type: "set", name, value, atMs: Date.now() });
    this.#publish(name, value);
  }

  /**
   * Internal store mutation + subscriber fan-out shared by `set` and
   * every `ingest*` method. Callers emit their own typed event before
   * delegating here so taps can distinguish source-of-mutation.
   */
  #publish(name: string, value: unknown): void {
    this.#store.set(name, value);
    const subscribers = this.#subscriptions.get(name);
    if (subscribers === undefined) {
      return;
    }

    for (const callback of subscribers.values()) {
      callback(value);
    }
  }

```

### 12. Test encodeExplorerCommand round-trip::falls back to the custom arm for an unknown action
`typescript:web/ui-explorer-codecs/src/index.test.ts::encodeExplorerCommand round-trip::falls back to the custom arm for an unknown action`  web/ui-explorer-codecs/src/index.test.ts:100-106  pkg:npm/@causeway/ui-explorer-codecs  score 1.13  
`it("falls back to the custom arm for an unknown action", () => { const message = decode("does.not.exist", {}); expect(message.command.case).toBe("custom"); if (message.command.case === "custom") { expect(message.command.value.action).toBe("does.not.exist"); } })`  
calls 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:101)  
tests_for 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:101)  
web/ui-explorer-codecs/src/index.test.ts:100-106
```typescript
  it("falls back to the custom arm for an unknown action", () => {
    const message = decode("does.not.exist", {});
    expect(message.command.case).toBe("custom");
    if (message.command.case === "custom") {
      expect(message.command.value.action).toBe("does.not.exist");
    }
  });
```

### 13. Test registerExplorerCodecs::registers a codec for every action and is idempotent
`typescript:web/ui-explorer-codecs/src/index.test.ts::registerExplorerCodecs::registers a codec for every action and is idempotent`  web/ui-explorer-codecs/src/index.test.ts:110-131  pkg:npm/@causeway/ui-explorer-codecs  score 1.11  via `registerExplorerCodecs` -> TESTED_BY (web/ui-explorer-codecs/src/index.test.ts:118, 1.00)  
`it("registers a codec for every action and is idempotent", () => { const registered = new Map<string, (payload: unknown) => Uint8Array>(); const bus = { registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array) { registered.set(action, codec); }, }; registerExplorerCodecs(bus); ex…`  
members 1  
calls 3: `codec` (web/ui-explorer-codecs/src/index.test.ts:129), `registerExplorerCodecs` (web/ui-explorer-codecs/src/index.test.ts:118), `registerExplorerCodecs` (web/ui-explorer-codecs/src/index.test.ts:123)  
tests_for 2: `registerExplorerCodecs` (web/ui-explorer-codecs/src/index.test.ts:118), `codec` (web/ui-explorer-codecs/src/index.test.ts:129)  
web/ui-explorer-codecs/src/index.test.ts:110-131
```typescript
  it("registers a codec for every action and is idempotent", () => {
    const registered = new Map<string, (payload: unknown) => Uint8Array>();
    const bus = {
      registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array) {
        registered.set(action, codec);
      },
    };

    registerExplorerCodecs(bus);
    expect(registered.size).toBe(EXPLORER_ACTIONS.length);

    // Calling again must not throw or change the registered set (the bus
    // keys codecs by action — a re-register is a no-op overwrite).
    registerExplorerCodecs(bus);
    expect(registered.size).toBe(EXPLORER_ACTIONS.length);

    // The registered codec for an action encodes that action.
    const codec = registered.get("pipeline.undo");
    expect(codec).toBeDefined();
    const message = fromBinary(ExplorerCommandSchema, codec?.({}) ?? new Uint8Array());
    expect(message.command.case).toBe("pipelineUndo");
  });
```

### 14. File bindings-ts/causeway-catalog-ui/src/codec.ts
`file:bindings-ts/causeway-catalog-ui/src/codec.ts`  bindings-ts/causeway-catalog-ui/src/codec.ts  pkg:npm/@causeway/catalog-ui  score 1.09  via `registerCatalogCodecs` <- CONTAINS (bindings-ts/causeway-catalog-ui/src/codec.ts:87-91, 1.00)  
members 8  
imports 2: `npm/@bufbuild/protobuf` (bindings-ts/causeway-catalog-ui/src/codec.ts:12), `bindings-ts/causeway-catalog-ui/src/index.ts` (bindings-ts/causeway-catalog-ui/src/codec.ts:14)  

### 15. Method BrowserBus::actionCodec
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::actionCodec`  torrentfx/web/framework/runtime-web/src/index.ts:665-667  pkg:npm/@torrentfx/runtime-web  score 1.09  
`actionCodec(action: string): ActionCodec | undefined`  
The registered payload codec for `action`, if any.  
callers 2: `BrowserConnectTransport::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:1815), `BrowserWebSocketTransport::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:1281)  
calls 1: `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:666)  
torrentfx/web/framework/runtime-web/src/index.ts:665-667
```typescript
  actionCodec(action: string): ActionCodec | undefined {
    return this.#actionCodecs.get(action);
  }
```

### 16. TypeAlias CodecBus
`typescript:web/ui-explorer-codecs/src/index.ts::CodecBus`  web/ui-explorer-codecs/src/index.ts:102-104  pkg:npm/@causeway/ui-explorer-codecs  score 1.08  via `CodecBus::registerActionCodec` <- DEFINES (web/ui-explorer-codecs/src/index.ts:103, 1.00)  
`type CodecBus = { registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void; };`  
members 1  
used_by 1: `registerExplorerCodecs` (web/ui-explorer-codecs/src/index.ts:112)  
web/ui-explorer-codecs/src/index.ts:102-104
```typescript
type CodecBus = {
  registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void;
};
```

### 17. TypeAlias CodecBus
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::CodecBus`  bindings-ts/causeway-catalog-ui/src/codec.ts:82-84  pkg:npm/@causeway/catalog-ui  score 0.98  via `CodecBus::registerActionCodec` <- DEFINES (bindings-ts/causeway-catalog-ui/src/codec.ts:83, 1.00)  
`type CodecBus = { registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void; };`  
members 1  
used_by 1: `registerCatalogCodecs` (bindings-ts/causeway-catalog-ui/src/codec.ts:87)  
bindings-ts/causeway-catalog-ui/src/codec.ts:82-84
```typescript
type CodecBus = {
  registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void;
};
```

### 18. File web/ui-explorer-codecs/src/index.test.ts
`file:web/ui-explorer-codecs/src/index.test.ts`  web/ui-explorer-codecs/src/index.test.ts  pkg:npm/@causeway/ui-explorer-codecs  score 0.94  via `registerExplorerCodecs` <- IMPORTS (web/ui-explorer-codecs/src/index.test.ts:14, 1.00)  
members 3  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:646)  
imports 6: `npm/@bufbuild/protobuf` (web/ui-explorer-codecs/src/index.test.ts:10), `npm/@causeway/explorer-ui` (web/ui-explorer-codecs/src/index.test.ts:11), `registerExplorerCodecs` (web/ui-explorer-codecs/src/index.test.ts:14) … 3 more  

### 19. Test encodeExplorerCommand round-trip
`typescript:web/ui-explorer-codecs/src/index.test.ts::encodeExplorerCommand round-trip`  web/ui-explorer-codecs/src/index.test.ts:41-107  pkg:npm/@causeway/ui-explorer-codecs  score 0.92  via `encodeExplorerCommand round-trip::falls back to the custom arm for an unknown action` <- DEFINES (web/ui-explorer-codecs/src/index.test.ts:100-106, 1.00)  
`describe("encodeExplorerCommand round-trip", () => { it("covers every action in EXPLORER_ACTIONS with an expected case", () => { // Guards against an action being added to the codec without a test. for (const action of EXPLORER_ACTIONS) { expect(EXPECTED_CASE).toHaveProperty(action); } }); for (cons…`  
members 6  
calls 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:51)  
tests_for 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:51)  
web/ui-explorer-codecs/src/index.test.ts:41-107
```typescript
describe("encodeExplorerCommand round-trip", () => {
  it("covers every action in EXPLORER_ACTIONS with an expected case", () => {
    // Guards against an action being added to the codec without a test.
    for (const action of EXPLORER_ACTIONS) {
      expect(EXPECTED_CASE).toHaveProperty(action);
    }
  });

  for (const action of EXPLORER_ACTIONS) {
    it(`encodes ${action} to a decodable command with the right oneof case`, () => {
      const message = decode(action, {});
      expect(message.command.case).toBe(EXPECTED_CASE[action]);
    });
  }

  it("carries datasetLoad.dataset through the wire, and never a file path", () => {
    const message = decode("dataset.load", {
      dataset: "credit_risk_features",
      "file-path": "s3://bucket/key.parquet",
    });
    expect(message.command.case).toBe("datasetLoad");
    if (message.command.case === "datasetLoad") {
      expect(message.command.value.dataset).toBe("credit_risk_features");
      expect(message.command.value.filePath).toBe("");
    }
  });

  it("carries dataGetBlock bounds and correlation through the wire", () => {
    const message = decode("data.get-block", {
      "start-row": 0,
      "end-row": 100,
      "request-id": "explorer-request-9",
      generation: 5,
    });
    expect(message.command.case).toBe("dataGetBlock");
    if (message.command.case === "dataGetBlock") {
      expect(message.command.value.startRow).toBe(0n);
      expect(message.command.value.endRow).toBe(100n);
      expect(message.command.value.requestId).toBe("explorer-request-9");
      expect(message.command.value.generation).toBe(5n);
    }
  });

  it("carries pipelineApply.operation through the wire", () => {
    const message = decode("pipeline.apply", { operation: '{"op":"sort"}' });
    expect(message.command.case).toBe("pipelineApply");
    if (message.command.case === "pipelineApply") {
      expect(message.command.value.operation).toBe('{"op":"sort"}');
    }
  });

  it("encodes profile.close as the custom arm tagged with the action", () => {
    const message = decode("profile.close", {});
    expect(message.command.case).toBe("custom");
    if (message.command.case === "custom") {
      expect(message.command.value.action).toBe("profile.close");
    }
  });

  it("falls back to the custom arm for an unknown action", () => {
    const message = decode("does.not.exist", {});
    expect(message.command.case).toBe("custom");
    if (message.command.case === "custom") {
      expect(message.command.value.action).toBe("does.not.exist");
    }
  });
});
```

### 20. File torrentfx/web/framework/runtime-web/src/index.ts
`file:torrentfx/web/framework/runtime-web/src/index.ts`  torrentfx/web/framework/runtime-web/src/index.ts  pkg:npm/@torrentfx/runtime-web  score 0.78  via `BrowserBus` <- CONTAINS (torrentfx/web/framework/runtime-web/src/index.ts:529-1127, 1.00)  
members 78  
designs 2: `docs/adrs/ADR-fxd4p0h3-component-state-styling.md` (docs/adrs/ADR-fxd4p0h3-component-state-styling.md:34), `docs/rfd/RFD-fd0t8b1v-ui-state-signaling-and-session-transport.md` (docs/rfd/RFD-fd0t8b1v-ui-state-signaling-and-session-transport.md:161)  
importers 9: `torrentfx/web/framework/runtime-web/src/arrow-append.ts` (torrentfx/web/framework/runtime-web/src/arrow-append.ts:21), `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:5), `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:15) … 6 more  
imports 10: `npm/@causeway/session` (torrentfx/web/framework/runtime-web/src/index.ts:1-5), `torrentfx/web/framework/runtime-web/src/arrow-append.ts` (torrentfx/web/framework/runtime-web/src/index.ts:34), `npm/@connectrpc/connect-web` (torrentfx/web/framework/runtime-web/src/index.ts:7) … 7 more  

### 21. Function decode
`typescript:web/ui-explorer-codecs/src/index.test.ts::decode`  web/ui-explorer-codecs/src/index.test.ts:37-39  pkg:npm/@causeway/ui-explorer-codecs  score 0.75  via `encodeExplorerCommand` <- CALLS (web/ui-explorer-codecs/src/index.test.ts:38, 1.00)  
`function decode(action: string, payload: unknown)`  
callers 6: `encodeExplorerCommand round-trip::carries pipelineApply.operation through the wire` (web/ui-explorer-codecs/src/index.test.ts:85), `encodeExplorerCommand round-trip::carries datasetLoad.dataset through the wire, and never a file path` (web/ui-explorer-codecs/src/index.test.ts:57), `encodeExplorerCommand round-trip::carries dataGetBlock bounds and correlation through the wire` (web/ui-explorer-codecs/src/index.test.ts:69) … 3 more  
calls 1: `encodeExplorerCommand` (web/ui-explorer-codecs/src/index.test.ts:38)  
tests 6: `encodeExplorerCommand round-trip::falls back to the custom arm for an unknown action` (web/ui-explorer-codecs/src/index.test.ts:101), `encodeExplorerCommand round-trip` (web/ui-explorer-codecs/src/index.test.ts:51), `encodeExplorerCommand round-trip::carries pipelineApply.operation through the wire` (web/ui-explorer-codecs/src/index.test.ts:85) … 3 more  
web/ui-explorer-codecs/src/index.test.ts:37-39
```typescript
function decode(action: string, payload: unknown) {
  return fromBinary(ExplorerCommandSchema, encodeExplorerCommand(action, payload));
}
```

### 22. Function big
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::big`  bindings-ts/causeway-catalog-ui/src/codec.ts:27-28  pkg:npm/@causeway/catalog-ui  score 0.73  via `encodeCatalogCommand` -> CALLS (bindings-ts/causeway-catalog-ui/src/codec.ts:50, 1.00)  
`big = (value: unknown): bigint => typeof value === "number" && Number.isFinite(value) ? BigInt(Math.trunc(value)) : 0n`  
callers 3: `encodeCatalogCommand` (bindings-ts/causeway-catalog-ui/src/codec.ts:48), `encodeCatalogCommand` (bindings-ts/causeway-catalog-ui/src/codec.ts:50), `encodeCatalogCommand` (bindings-ts/causeway-catalog-ui/src/codec.ts:47)  
bindings-ts/causeway-catalog-ui/src/codec.ts:27-28
```typescript
const big = (value: unknown): bigint =>
  typeof value === "number" && Number.isFinite(value) ? BigInt(Math.trunc(value)) : 0n;
```

### 23. Method BrowserBus::set
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::set`  torrentfx/web/framework/runtime-web/src/index.ts:587-590  pkg:npm/@torrentfx/runtime-web  score 0.65  via `BrowserBus::registerActionCodec` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:661, 1.00)  
`set(name: string, value: unknown): void`  
callers 40: `webSocketArrowFrame` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:482), `BrowserBus::handle` (torrentfx/web/framework/runtime-web/src/index.ts:650), `BrowserBus::subscribe` (torrentfx/web/framework/runtime-web/src/index.ts:615) … 37 more  
tests 5: `pipePageLifecycleToDiagnostics::drops the `hidden` lifecycle tag (not a PagePhase` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:351), `BrowserBus DOM binding lifecycle::unsubscribes a binding when its element leaves the document` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:62), `pipePageLifecycleToDiagnostics::emits a page.lifecycle record on each tag change` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:312) … 2 more  
torrentfx/web/framework/runtime-web/src/index.ts:587-590
```typescript
  set(name: string, value: unknown): void {
    this.#emit({ type: "set", name, value, atMs: Date.now() });
    this.#publish(name, value);
  }
```

### 24. Function commandFor
`typescript:web/ui-explorer-codecs/src/index.ts::commandFor`  web/ui-explorer-codecs/src/index.ts:33-77  pkg:npm/@causeway/ui-explorer-codecs  score 0.60  via `encodeExplorerCommand` -> CALLS (web/ui-explorer-codecs/src/index.ts:81, 1.00)  
`function commandFor(action: string, payload: unknown): CommandCase`  
callers 1: `encodeExplorerCommand` (web/ui-explorer-codecs/src/index.ts:81)  
calls 7: `big` (web/ui-explorer-codecs/src/index.ts:51), `str` (web/ui-explorer-codecs/src/index.ts:43), `big` (web/ui-explorer-codecs/src/index.ts:49) … 4 more  
uses_types 1: `CommandCase` (web/ui-explorer-codecs/src/index.ts:33)  
web/ui-explorer-codecs/src/index.ts:33-77
```typescript
function commandFor(action: string, payload: unknown): CommandCase {
  const p = (payload ?? {}) as Dict;
  switch (action) {
    case "explorer.init":
      return { case: "init", value: {} };
    case "explorer.prewarm":
      return { case: "prewarm", value: {} };
    case "dataset.load":
      // A dataset is named by its catalog entry; the retired `filePath` is
      // never sent (the server refuses a command that sets it).
      return { case: "datasetLoad", value: { dataset: str(p[DATASET]) } };
    case "data.get-block":
      return {
        case: "dataGetBlock",
        value: {
          startRow: big(p[START_ROW]),
          endRow: big(p[END_ROW]),
          requestId: str(p[REQUEST_ID]),
          generation: big(p[GENERATION]),
        },
      };
    case "pipeline.apply":
      return { case: "pipelineApply", value: { operation: str(p[OPERATION]) } };
    case "pipeline.load":
      return { case: "pipelineLoad", value: { operation: str(p[OPERATION]) } };
    case "pipeline.undo":
      return { case: "pipelineUndo", value: {} };
    case "pipeline.clear":
      return { case: "pipelineClear", value: {} };
    case "pipeline.save":
      return { case: "pipelineSave", value: {} };
    case "profile.run":
      return { case: "profileRun", value: {} };
    case "profile.run-server":
      return { case: "profileRunServer", value: {} };
    case "profile.cancel-server":
      return { case: "profileCancelServer", value: {} };
    case "error.dismiss":
      return { case: "errorDismiss", value: {} };
    case "profile.close":
      return { case: "custom", value: { action, payloadJson: new Uint8Array() } };
    default:
      return { case: "custom", value: { action, payloadJson: new Uint8Array() } };
  }
}
```

### 25. Test encodeCatalogCommand round-trip::throws for an action it does not handle
`typescript:web/ui-catalog-status/src/codec.test.ts::encodeCatalogCommand round-trip::throws for an action it does not handle`  web/ui-catalog-status/src/codec.test.ts:75-77  pkg:npm/@causeway/ui-catalog-status  score 0.50  via `encodeCatalogCommand` -> TESTED_BY (web/ui-catalog-status/src/codec.test.ts:76, 1.00)  
`it("throws for an action it does not handle", () => { expect(() => encodeCatalogCommand("catalog.unknown", {})).toThrow(/no catalog codec/); })`  
calls 1: `encodeCatalogCommand` (web/ui-catalog-status/src/codec.test.ts:76)  
tests_for 1: `encodeCatalogCommand` (web/ui-catalog-status/src/codec.test.ts:76)  
web/ui-catalog-status/src/codec.test.ts:75-77
```typescript
  it("throws for an action it does not handle", () => {
    expect(() => encodeCatalogCommand("catalog.unknown", {})).toThrow(/no catalog codec/);
  });
```

### 26. Function boot
`typescript:web/ui-catalog-filter/src/index.ts::boot`  web/ui-catalog-filter/src/index.ts:96-207  pkg:npm/@causeway/ui-catalog-filter  score 0.49  via `registerCatalogCodecs` <- CALLS (web/ui-catalog-filter/src/index.ts:103, 1.00)  
`function boot(bus: FilterBus): void`  
members 4  
callers 1: `web/ui-catalog-filter/src/index.ts` (web/ui-catalog-filter/src/index.ts:212)  
calls 22: `dispatchFilter` (web/ui-catalog-filter/src/index.ts:193), `systemIds` (web/ui-catalog-filter/src/index.ts:117), `dispatchFilter` (web/ui-catalog-filter/src/index.ts:121) … 19 more  
uses_types 3: `CatalogFilterCommand` (web/ui-catalog-filter/src/index.ts:119), `FilterBus` (web/ui-catalog-filter/src/index.ts:96), `TaxonomyNode` (web/ui-catalog-filter/src/index.ts:179)  
web/ui-catalog-filter/src/index.ts:96-175, cut
```typescript
function boot(bus: FilterBus): void {
  // Ensure the catalog command codecs are on the bus before the first
  // `catalog.filter` dispatch below. catalog-status registers the same
  // codecs from the shared module too; `registerActionCodec` is keyed by
  // action, so the second registration is a harmless overwrite. This is
  // what removes the old hidden dependency on catalog-status.js loading
  // first.
  registerCatalogCodecs(bus);

  const l1Sel = document.getElementById("catalog-l1") as HTMLSelectElement | null;
  const l2Sel = document.getElementById("catalog-l2") as HTMLSelectElement | null;
  const atSel = document.getElementById("catalog-asset-type") as HTMLSelectElement | null;
  const searchEl = document.getElementById("catalog-search") as HTMLInputElement | null;
  const chipsEl = document.getElementById("catalog-filter-chips");
  // Not the catalog page — the bundle is loaded platform-wide but inert
  // unless the filter controls are present.
  if (!l1Sel || !l2Sel || !atSel || !searchEl || !chipsEl) return;

  createRoot((dispose) => {
    const store = createMviStore(catalogFilterDecider, {
      clock: systemClock,
      ids: systemIds(),
      emitEffect: () => {},
      runCommand: (command: CatalogFilterCommand) => {
        if (command.type === "SendFilter") {
          dispatchFilter(bus, command.filter);
        }
        return undefined;
      },
    });

    // ── DOM → intents ────────────────────────────────────────────────
    const onL1Change = () => store.accept({ type: "L1Selected" as const, path: l1Sel.value });
    const onL2Change = () => store.accept({ type: "L2Selected" as const, path: l2Sel.value });
    const onAssetTypeChange = () =>
      store.accept({ type: "AssetTypeSelected" as const, value: atSel.value });
    l1Sel.addEventListener("change", onL1Change);
    l2Sel.addEventListener("change", onL2Change);
    atSel.addEventListener("change", onAssetTypeChange);
    let searchTimer: ReturnType<typeof setTimeout> | undefined;
    const onSearchInput = () => {
      clearTimeout(searchTimer);
      searchTimer = setTimeout(
        () => store.accept({ type: "SearchChanged", q: searchEl.value }),
        200,
      );
    };
    searchEl.addEventListener("input", onSearchInput);

    // ── state → view (fine-grained effects) ──────────────────────────
    createEffect(() => fillOptions(l1Sel, l1Options(store.state)));
    createEffect(() => {
      if (l1Sel.value !== store.state.l1) l1Sel.value = store.state.l1;
    });
    createEffect(() => fillOptions(l2Sel, l2Options(store.state)));
    createEffect(() => {
      if (l2Sel.value !== store.state.l2) l2Sel.value = store.state.l2;
    });
    createEffect(() => {
      if (atSel.value !== store.state.assetType) atSel.value = store.state.assetType;
    });
    createEffect(() => {
      if (searchEl.value !== store.state.q) searchEl.value = store.state.q;
    });
    createEffect(() =>
      renderChips(chipsEl, activeChips(store.state), () => store.accept({ type: "ClearFilters" })),
    );

    // ── Taxonomy: ConnectRPC → intent (ADR 0016) ─────────────────────
    // Replaces the GET /api/taxonomy fetch with TaxonomyService.GetTaxonomy.
    // The generated message is camelCase; map it to the decider's
    // snake_case TaxonomyNode (an empty parent_path means top-level).
    const taxonomyClient = createClient(
      TaxonomyService,
      createConnectTransport({ baseUrl: window.location.origin }),
    );
    const taxonomyAbort = new AbortController();
    taxonomyClient
      .getTaxonomy({}, { signal: taxonomyAbort.signal })
      .then((res) =>
```

### 27. Method BrowserBus::get
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::get`  torrentfx/web/framework/runtime-web/src/index.ts:579-581  pkg:npm/@torrentfx/runtime-web  score 0.45  via `BrowserBus::actionCodec` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:666, 1.00)  
`get(name: string): unknown`  
callers 31: `BrowserBus::invoke` (torrentfx/web/framework/runtime-web/src/index.ts:878), `BrowserBus::subscribe` (torrentfx/web/framework/runtime-web/src/index.ts:612), `BrowserBus::handle` (torrentfx/web/framework/runtime-web/src/index.ts:645) … 28 more  
calls 1: `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:580)  
torrentfx/web/framework/runtime-web/src/index.ts:579-581
```typescript
  get(name: string): unknown {
    return this.#store.get(name);
  }
```

### 28. Function str
`typescript:web/ui-explorer-codecs/src/index.ts::str`  web/ui-explorer-codecs/src/index.ts:25  pkg:npm/@causeway/ui-explorer-codecs  score 0.45  via `web/ui-explorer-codecs/src/index.ts` -> CONTAINS (web/ui-explorer-codecs/src/index.ts:25, 1.00)  
`str = (value: unknown): string => (typeof value === "string" ? value : "")`  
callers 4: `commandFor` (web/ui-explorer-codecs/src/index.ts:43), `commandFor` (web/ui-explorer-codecs/src/index.ts:50), `commandFor` (web/ui-explorer-codecs/src/index.ts:55) … 1 more  
web/ui-explorer-codecs/src/index.ts:25-25
```typescript
const str = (value: unknown): string => (typeof value === "string" ? value : "");
```

### 29. Test registerExplorerCodecs
`typescript:web/ui-explorer-codecs/src/index.test.ts::registerExplorerCodecs`  web/ui-explorer-codecs/src/index.test.ts:109-132  pkg:npm/@causeway/ui-explorer-codecs  score 0.44  via `registerExplorerCodecs::registers a codec for every action and is idempotent` <- DEFINES (web/ui-explorer-codecs/src/index.test.ts:110-131, 1.00)  
`describe("registerExplorerCodecs", () => { it("registers a codec for every action and is idempotent", () => { const registered = new Map<string, (payload: unknown) => Uint8Array>(); const bus = { registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array) { registered.set(action, co…`  
members 1  
web/ui-explorer-codecs/src/index.test.ts:109-132
```typescript
describe("registerExplorerCodecs", () => {
  it("registers a codec for every action and is idempotent", () => {
    const registered = new Map<string, (payload: unknown) => Uint8Array>();
    const bus = {
      registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array) {
        registered.set(action, codec);
      },
    };

    registerExplorerCodecs(bus);
    expect(registered.size).toBe(EXPLORER_ACTIONS.length);

    // Calling again must not throw or change the registered set (the bus
    // keys codecs by action — a re-register is a no-op overwrite).
    registerExplorerCodecs(bus);
    expect(registered.size).toBe(EXPLORER_ACTIONS.length);

    // The registered codec for an action encodes that action.
    const codec = registered.get("pipeline.undo");
    expect(codec).toBeDefined();
    const message = fromBinary(ExplorerCommandSchema, codec?.({}) ?? new Uint8Array());
    expect(message.command.case).toBe("pipelineUndo");
  });
});
```

### 30. Package npm/@causeway/ui-explorer-codecs
`pkg:npm/@causeway/ui-explorer-codecs`  web/ui-explorer-codecs/package.json:2  pkg:npm/@causeway/ui-explorer-codecs  score 0.42  via `web/ui-explorer-codecs/src/index.test.ts` <- CONTAINS (web/ui-explorer-codecs/package.json:2, 1.00)  
`npm/@causeway/ui-explorer-codecs`  
members 7  
dependencies 6: `npm/@torrentfx/runtime-web` (web/ui-explorer-codecs/package.json:20), `npm/vitest` (web/ui-explorer-codecs/package.json:25), `npm/vite` (web/ui-explorer-codecs/package.json:24) … 3 more  

### 31. Test encodeExplorerCommand round-trip::carries datasetLoad.dataset through the wire, and never a file path
`typescript:web/ui-explorer-codecs/src/index.test.ts::encodeExplorerCommand round-trip::carries datasetLoad.dataset through the wire, and never a file path`  web/ui-explorer-codecs/src/index.test.ts:56-66  pkg:npm/@causeway/ui-explorer-codecs  score 0.40  via `decode` <- CALLS (web/ui-explorer-codecs/src/index.test.ts:57, 1.00)  
`it("carries datasetLoad.dataset through the wire, and never a file path", () => { const message = decode("dataset.load", { dataset: "credit_risk_features", "file-path": "s3://bucket/key.parquet", }); expect(message.command.case).toBe("datasetLoad"); if (message.command.case === "datasetLoad") { expe…`  
calls 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:57)  
tests_for 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:57)  
web/ui-explorer-codecs/src/index.test.ts:56-66
```typescript
  it("carries datasetLoad.dataset through the wire, and never a file path", () => {
    const message = decode("dataset.load", {
      dataset: "credit_risk_features",
      "file-path": "s3://bucket/key.parquet",
    });
    expect(message.command.case).toBe("datasetLoad");
    if (message.command.case === "datasetLoad") {
      expect(message.command.value.dataset).toBe("credit_risk_features");
      expect(message.command.value.filePath).toBe("");
    }
  });
```

### 32. Function big
`typescript:web/ui-explorer-codecs/src/index.ts::big`  web/ui-explorer-codecs/src/index.ts:26-27  pkg:npm/@causeway/ui-explorer-codecs  score 0.40  via `web/ui-explorer-codecs/src/index.ts` -> CONTAINS (web/ui-explorer-codecs/src/index.ts:26-27, 1.00)  
`big = (value: unknown): bigint => typeof value === "number" && Number.isFinite(value) ? BigInt(Math.trunc(value)) : 0n`  
callers 3: `commandFor` (web/ui-explorer-codecs/src/index.ts:51), `commandFor` (web/ui-explorer-codecs/src/index.ts:49), `commandFor` (web/ui-explorer-codecs/src/index.ts:48)  
web/ui-explorer-codecs/src/index.ts:26-27
```typescript
const big = (value: unknown): bigint =>
  typeof value === "number" && Number.isFinite(value) ? BigInt(Math.trunc(value)) : 0n;
```

### 33. Test encodeExplorerCommand round-trip::encodes profile.close as the custom arm tagged with the action
`typescript:web/ui-explorer-codecs/src/index.test.ts::encodeExplorerCommand round-trip::encodes profile.close as the custom arm tagged with the action`  web/ui-explorer-codecs/src/index.test.ts:92-98  pkg:npm/@causeway/ui-explorer-codecs  score 0.38  via `decode` <- CALLS (web/ui-explorer-codecs/src/index.test.ts:93, 1.00)  
`it("encodes profile.close as the custom arm tagged with the action", () => { const message = decode("profile.close", {}); expect(message.command.case).toBe("custom"); if (message.command.case === "custom") { expect(message.command.value.action).toBe("profile.close"); } })`  
calls 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:93)  
tests_for 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:93)  
web/ui-explorer-codecs/src/index.test.ts:92-98
```typescript
  it("encodes profile.close as the custom arm tagged with the action", () => {
    const message = decode("profile.close", {});
    expect(message.command.case).toBe("custom");
    if (message.command.case === "custom") {
      expect(message.command.value.action).toBe("profile.close");
    }
  });
```

### 34. Function decode
`typescript:web/ui-catalog-status/src/codec.test.ts::decode`  web/ui-catalog-status/src/codec.test.ts:20-22  pkg:npm/@causeway/ui-catalog-status  score 0.38  via `encodeCatalogCommand` <- CALLS (web/ui-catalog-status/src/codec.test.ts:21, 1.00)  
`function decode(action: string, payload: unknown)`  
callers 4: `encodeCatalogCommand round-trip` (web/ui-catalog-status/src/codec.test.ts:27), `encodeCatalogCommand round-trip::omits sort when no sort column is present` (web/ui-catalog-status/src/codec.test.ts:52), `encodeCatalogCommand round-trip::carries applyFilter fields (snake_case payload → proto) through the wire` (web/ui-catalog-status/src/codec.test.ts:60) … 1 more  
calls 1: `encodeCatalogCommand` (web/ui-catalog-status/src/codec.test.ts:21)  
tests 4: `encodeCatalogCommand round-trip` (web/ui-catalog-status/src/codec.test.ts:27), `encodeCatalogCommand round-trip::carries applyFilter fields (snake_case payload → proto) through the wire` (web/ui-catalog-status/src/codec.test.ts:60), `encodeCatalogCommand round-trip::omits sort when no sort column is present` (web/ui-catalog-status/src/codec.test.ts:52) … 1 more  
web/ui-catalog-status/src/codec.test.ts:20-22
```typescript
function decode(action: string, payload: unknown) {
  return fromBinary(CatalogCommandSchema, encodeCatalogCommand(action, payload));
}
```

### 35. Package npm/@bufbuild/protobuf
`pkg:npm/@bufbuild/protobuf`  pkg:npm/@bufbuild/protobuf  score 0.37  via `web/ui-explorer-codecs/src/index.test.ts` -> IMPORTS (web/ui-explorer-codecs/src/index.test.ts:10, 1.00)  
dependents 20: `npm/@causeway/catalog` (bindings-ts/causeway-catalog/package.json:14), `npm/@causeway/ui-catalog-status` (web/ui-catalog-status/package.json:18), `npm/causeway-monorepo` (package.json:8) … 17 more  
importers 136: `services-node/relay-grpc/src/health.ts` (services-node/relay-grpc/src/health.ts:3), `services-node/relay-grpc/gen/causeway/catalog/v2/details_pb.ts` (services-node/relay-grpc/gen/causeway/catalog/v2/details_pb.ts:7), `web/ui-semantic-graph/src/codec.test.ts` (web/ui-semantic-graph/src/codec.test.ts:1) … 133 more  

### 36. Test encodeExplorerCommand round-trip::carries pipelineApply.operation through the wire
`typescript:web/ui-explorer-codecs/src/index.test.ts::encodeExplorerCommand round-trip::carries pipelineApply.operation through the wire`  web/ui-explorer-codecs/src/index.test.ts:84-90  pkg:npm/@causeway/ui-explorer-codecs  score 0.37  via `decode` <- CALLS (web/ui-explorer-codecs/src/index.test.ts:85, 1.00)  
`it("carries pipelineApply.operation through the wire", () => { const message = decode("pipeline.apply", { operation: '{"op":"sort"}' }); expect(message.command.case).toBe("pipelineApply"); if (message.command.case === "pipelineApply") { expect(message.command.value.operation).toBe('{"op":"sort"}'); …`  
calls 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:85)  
tests_for 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:85)  
web/ui-explorer-codecs/src/index.test.ts:84-90
```typescript
  it("carries pipelineApply.operation through the wire", () => {
    const message = decode("pipeline.apply", { operation: '{"op":"sort"}' });
    expect(message.command.case).toBe("pipelineApply");
    if (message.command.case === "pipelineApply") {
      expect(message.command.value.operation).toBe('{"op":"sort"}');
    }
  });
```

### 37. Test encodeExplorerCommand round-trip::carries dataGetBlock bounds and correlation through the wire
`typescript:web/ui-explorer-codecs/src/index.test.ts::encodeExplorerCommand round-trip::carries dataGetBlock bounds and correlation through the wire`  web/ui-explorer-codecs/src/index.test.ts:68-82  pkg:npm/@causeway/ui-explorer-codecs  score 0.37  via `decode` <- CALLS (web/ui-explorer-codecs/src/index.test.ts:69, 1.00)  
`it("carries dataGetBlock bounds and correlation through the wire", () => { const message = decode("data.get-block", { "start-row": 0, "end-row": 100, "request-id": "explorer-request-9", generation: 5, }); expect(message.command.case).toBe("dataGetBlock"); if (message.command.case === "dataGetBlock")…`  
calls 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:69)  
tests_for 1: `decode` (web/ui-explorer-codecs/src/index.test.ts:69)  
web/ui-explorer-codecs/src/index.test.ts:68-82
```typescript
  it("carries dataGetBlock bounds and correlation through the wire", () => {
    const message = decode("data.get-block", {
      "start-row": 0,
      "end-row": 100,
      "request-id": "explorer-request-9",
      generation: 5,
    });
    expect(message.command.case).toBe("dataGetBlock");
    if (message.command.case === "dataGetBlock") {
      expect(message.command.value.startRow).toBe(0n);
      expect(message.command.value.endRow).toBe(100n);
      expect(message.command.value.requestId).toBe("explorer-request-9");
      expect(message.command.value.generation).toBe(5n);
    }
  });
```

### 38. TypeAlias CommandCase
`typescript:web/ui-explorer-codecs/src/index.ts::CommandCase`  web/ui-explorer-codecs/src/index.ts:31  pkg:npm/@causeway/ui-explorer-codecs  score 0.36  via `web/ui-explorer-codecs/src/index.ts` -> CONTAINS (web/ui-explorer-codecs/src/index.ts:31, 1.00)  
`type CommandCase = MessageInitShape<typeof ExplorerCommandSchema>["command"];`  
used_by 1: `commandFor` (web/ui-explorer-codecs/src/index.ts:33)  
web/ui-explorer-codecs/src/index.ts:31-31
```typescript
type CommandCase = MessageInitShape<typeof ExplorerCommandSchema>["command"];
```

### 39. Method BrowserConnectTransport::dispatch
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserConnectTransport::dispatch`  torrentfx/web/framework/runtime-web/src/index.ts:1814-1824  pkg:npm/@torrentfx/runtime-web  score 0.35  via `BrowserBus::actionCodec` <- CALLS (torrentfx/web/framework/runtime-web/src/index.ts:1815, 1.00)  
`dispatch(action: string, payload: unknown): void`  
Upstream Action: bus.Envelope field 4, framed identically to the WS transport, carried as ClientFrame.frame over the unary Send.  
callers 2: `createBrowserLifecycleActor::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:2449), `createSocketSessionActor::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:2514)  
calls 6: `encodeProtoField` (torrentfx/web/framework/runtime-web/src/index.ts:1823), `encodeProtoBytesField` (torrentfx/web/framework/runtime-web/src/index.ts:1821), `codec` (torrentfx/web/framework/runtime-web/src/index.ts:1817) … 3 more  
torrentfx/web/framework/runtime-web/src/index.ts:1814-1824
```typescript
  dispatch(action: string, payload: unknown): void {
    const codec = this.#bus.actionCodec(action);
    const payloadBytes = codec
      ? codec(payload)
      : new TextEncoder().encode(JSON.stringify(payload ?? null));
    const inner = concatBytes([
      encodeProtoStringField(1, action),
      encodeProtoBytesField(2, payloadBytes),
    ]);
    this.#send(encodeProtoField(4, inner), "");
  }
```

### 40. Test encodeCatalogCommand round-trip
`typescript:web/ui-catalog-status/src/codec.test.ts::encodeCatalogCommand round-trip`  web/ui-catalog-status/src/codec.test.ts:24-78  pkg:npm/@causeway/ui-catalog-status  score 0.35  via `encodeCatalogCommand round-trip::throws for an action it does not handle` <- DEFINES (web/ui-catalog-status/src/codec.test.ts:75-77, 1.00)  
`describe("encodeCatalogCommand round-trip", () => { for (const [action, expected] of Object.entries(EXPECTED_CASE)) { it(`encodes ${action} to a decodable command with case ${expected}`, () => { const message = decode(action, {}); expect(message.command.case).toBe(expected); }); } it("carries getBlo…`  
members 4  
calls 1: `decode` (web/ui-catalog-status/src/codec.test.ts:27)  
tests_for 1: `decode` (web/ui-catalog-status/src/codec.test.ts:27)  
web/ui-catalog-status/src/codec.test.ts:24-78
```typescript
describe("encodeCatalogCommand round-trip", () => {
  for (const [action, expected] of Object.entries(EXPECTED_CASE)) {
    it(`encodes ${action} to a decodable command with case ${expected}`, () => {
      const message = decode(action, {});
      expect(message.command.case).toBe(expected);
    });
  }

  it("carries getBlock row bounds and sort through the wire", () => {
    const message = decode("catalog.get-block", {
      "start-row": 0,
      "end-row": 50,
      "request-id": "catalog-request-7",
      generation: 3,
      sortModel: [{ column: "name", descending: true }],
    });
    expect(message.command.case).toBe("getBlock");
    if (message.command.case === "getBlock") {
      expect(message.command.value.startRow).toBe(0n);
      expect(message.command.value.endRow).toBe(50n);
      expect(message.command.value.requestId).toBe("catalog-request-7");
      expect(message.command.value.generation).toBe(3n);
      expect(message.command.value.sort?.column).toBe("name");
      expect(message.command.value.sort?.descending).toBe(true);
    }
  });

  it("omits sort when no sort column is present", () => {
    const message = decode("catalog.get-block", { "start-row": 0, "end-row": 50 });
    expect(message.command.case).toBe("getBlock");
    if (message.command.case === "getBlock") {
      expect(message.command.value.sort).toBeUndefined();
    }
  });

  it("carries applyFilter fields (snake_case payload → proto) through the wire", () => {
    const message = decode("catalog.filter", {
      q: "revenue",
      domain_l1: "finance",
      domain_l2: "finance.reporting",
      asset_type: "table",
    });
    expect(message.command.case).toBe("applyFilter");
    if (message.command.case === "applyFilter") {
      expect(message.command.value.q).toBe("revenue");
      expect(message.command.value.domainL1).toBe("finance");
      expect(message.command.value.domainL2).toBe("finance.reporting");
      expect(message.command.value.assetType).toBe("table");
    }
  });

  it("throws for an action it does not handle", () => {
    expect(() => encodeCatalogCommand("catalog.unknown", {})).toThrow(/no catalog codec/);
  });
});
```

### 41. File web/ui-catalog-filter/src/index.ts
`file:web/ui-catalog-filter/src/index.ts`  web/ui-catalog-filter/src/index.ts  pkg:npm/@causeway/ui-catalog-filter  score 0.33  via `boot` <- CONTAINS (web/ui-catalog-filter/src/index.ts:96-207, 1.00)  
members 5  
calls 2: `boot` (web/ui-catalog-filter/src/index.ts:212), `whenBusReady` (web/ui-catalog-filter/src/index.ts:212)  
imports 16: `npm/@torrentfx/runtime-web` (web/ui-catalog-filter/src/index.ts:18), `web/ui-catalog-filter/src/lifecycle.ts` (web/ui-catalog-filter/src/index.ts:31), `activeChips` (web/ui-catalog-filter/src/index.ts:21-30) … 13 more  

### 42. Class BrowserConnectTransport
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserConnectTransport`  torrentfx/web/framework/runtime-web/src/index.ts:1556-1863  pkg:npm/@torrentfx/runtime-web  score 0.33  via `BrowserConnectTransport::dispatch` <- DEFINES (torrentfx/web/framework/runtime-web/src/index.ts:1814-1824, 1.00)  
`class BrowserConnectTransport implements TransportLike`  
ConnectRPC transport — the WebSocket alternative (ADR 0016 Phase 3). (doc continues, 843 bytes)  
members 5  
calls 51: `readProtoOneof` (torrentfx/web/framework/runtime-web/src/index.ts:1653), `decodeFrameworkControl` (torrentfx/web/framework/runtime-web/src/index.ts:1760), `connectDeliveryMode` (torrentfx/web/framework/runtime-web/src/index.ts:1804) … 48 more  
importers 1: `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:22-29)  
used_by 6: `browser transport conformance::reconnects ConnectRPC, sends init after each hello, and replays subscriptions once` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:234), `bootstrap` (torrentfx/web/framework/app/src/index.ts:345), `appendWithConnect` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:312) … 3 more  
uses_types 3: `BrowserBus` (torrentfx/web/framework/runtime-web/src/index.ts:1557), `ArrowChunkHeader` (torrentfx/web/framework/runtime-web/src/index.ts:1800), `ArrowDatasetIngest` (torrentfx/web/framework/runtime-web/src/index.ts:1559)  
torrentfx/web/framework/runtime-web/src/index.ts:1556-1635, cut
```typescript
export class BrowserConnectTransport implements TransportLike {
  readonly #bus: BrowserBus;
  readonly #url: string;
  readonly #arrow: ArrowDatasetIngest;
  #client: Client<typeof SessionService> | undefined;
  #abort: AbortController | undefined;
  /** Connection id minted in the server_hello frame; echoed on each Send. */
  #connectionId = "";
  #reconnectTimer: ReturnType<typeof setTimeout> | undefined;
  #reconnectAttempt = 0;
  #subscriptionGeneration = 0;
  #stopped = true;

  constructor(bus: BrowserBus, url: string) {
    this.#bus = bus;
    this.#url = url;
    this.#arrow = new ArrowDatasetIngest(bus);
  }

  connect(): void {
    if (!this.#stopped) {
      return;
    }
    this.#stopped = false;
    this.#client = createClient(SessionService, createConnectTransport({ baseUrl: this.#url }));
    this.#bus.connectTransport(this);
    this.#bus.beginSync();
    this.#startSubscribe();
  }

  #startSubscribe(): void {
    if (this.#stopped || this.#client === undefined || this.#abort !== undefined) {
      return;
    }
    this.#abort = new AbortController();
    this.#connectionId = "";
    this.#subscriptionGeneration += 1;
    const generation = this.#subscriptionGeneration;
    void this.#runSubscribe(this.#abort.signal).finally(() => {
      if (this.#stopped || generation !== this.#subscriptionGeneration) {
        return;
      }
      this.#abort = undefined;
      this.#bus.beginSync();
      this.#scheduleReconnect();
    });
  }

  async #runSubscribe(signal: AbortSignal): Promise<void> {
    const client = this.#client;
    if (client === undefined) {
      return;
    }
    try {
      // Empty session_id: the server derives identity from the HttpOnly
      // session cookie (ADR 0016). islands subscribe via Send, like WS.
      for await (const frame of client.subscribe({ afterSeq: 0n }, { signal })) {
        if (frame.body.case === "control") {
          this.#applyEnvelope(frame.body.value.envelope);
          const cid = this.#bus.get("transport.connection-id");
          if (typeof cid === "string" && cid !== "" && cid !== this.#connectionId) {
            this.#connectionId = cid;
            this.#sendInit();
          }
        } else if (frame.body.case === "arrow") {
          this.#applyArrow(frame.body.value);
        }
      }
    } catch (error) {
      if (!signal.aborted) {
        this.#bus.ingestSignals({
          "transport.error.code": "subscribe_failed",
          "transport.error.message": String(error),
        });
      }
    }
  }

  #scheduleReconnect(): void {
    if (this.#stopped || this.#reconnectTimer !== undefined) {
```

### 43. TypeAlias BusEvent
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BusEvent`  torrentfx/web/framework/runtime-web/src/index.ts:457-489  pkg:npm/@torrentfx/runtime-web  score 0.33  via `BrowserBus` -> USES_TYPE (torrentfx/web/framework/runtime-web/src/index.ts:568, 1.00)  
`type BusEvent = | { type: "set"; name: string; value: unknown; atMs: number } | { type: "ingest"; source: "signals" | "arrow" | "elements" | "command" | "domain-event"; name?: string; atMs: number; } | { type: "dispatch"; action: string; payload: unknown; atMs: number } | { type: "handle"; action: s…`  
Observable bus event emitted by `BrowserBus.tap(cb)`. Mirrors the `BusEvent` type the islands SDK exports — convergence point for the SDK and BrowserBus journal/inspector work tracked in `EVOLVE-BROWSERBUS.md` phase 4. (doc continues, 355 bytes)  
importers 1: `torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:17-22)  
used_by 4: `makeStubBus::tap` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:34), `Tap` (torrentfx/web/framework/runtime-web/src/index.ts:491), `makeStubBus` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:30) … 1 more  
uses_types 3: `TraceId` (torrentfx/web/framework/runtime-web/src/index.ts:476), `CorrelationIds` (torrentfx/web/framework/runtime-web/src/index.ts:477), `IslandPhase` (torrentfx/web/framework/runtime-web/src/index.ts:470)  
torrentfx/web/framework/runtime-web/src/index.ts:457-489
```typescript
export type BusEvent =
  | { type: "set"; name: string; value: unknown; atMs: number }
  | {
      type: "ingest";
      source: "signals" | "arrow" | "elements" | "command" | "domain-event";
      name?: string;
      atMs: number;
    }
  | { type: "dispatch"; action: string; payload: unknown; atMs: number }
  | { type: "handle"; action: string; payload: unknown; atMs: number }
  | {
      type: "island.transition";
      islandId: string;
      phase: IslandPhase;
      atMs: number;
    }
  | {
      type: "invoke.start";
      kind: string;
      requestId: TraceId;
      correlation: CorrelationIds;
      via: "ws" | "direct";
      atMs: number;
    }
  | {
      type: "invoke.complete";
      kind: string;
      requestId: TraceId;
      correlation: CorrelationIds;
      outcome: "ok" | "error" | "cancel" | "timeout";
      durationMs: number;
      atMs: number;
    };
```

### 44. File torrentfx/web/framework/runtime-web/src/browser-bus.test.ts
`file:torrentfx/web/framework/runtime-web/src/browser-bus.test.ts`  torrentfx/web/framework/runtime-web/src/browser-bus.test.ts  pkg:npm/@torrentfx/runtime-web  score 0.32  via `BrowserBus` <- IMPORTS (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:5, 1.00)  
members 3  
imports 4: `torrentfx/web/framework/runtime-web/src/index.ts` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:5), `npm/vitest` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:3), `scanTextBindings` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:5) … 1 more  

### 45. Package npm/@causeway/explorer-ui
`pkg:npm/@causeway/explorer-ui`  bindings-ts/causeway-explorer-ui/package.json:2  pkg:npm/@causeway/explorer-ui  score 0.30  via `web/ui-explorer-codecs/src/index.test.ts` -> IMPORTS (web/ui-explorer-codecs/src/index.test.ts:11, 1.00)  
`npm/@causeway/explorer-ui`  
members 5  
dependents 1: `npm/@causeway/ui-explorer-codecs` (web/ui-explorer-codecs/package.json:19)  
dependencies 2: `npm/typescript` (bindings-ts/causeway-explorer-ui/package.json:17), `npm/@bufbuild/protobuf` (bindings-ts/causeway-explorer-ui/package.json:14)  
importers 2: `web/ui-explorer-codecs/src/index.test.ts` (web/ui-explorer-codecs/src/index.test.ts:11), `web/ui-explorer-codecs/src/index.ts` (web/ui-explorer-codecs/src/index.ts:15)  

### 46. Class BrowserWebSocketTransport
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserWebSocketTransport`  torrentfx/web/framework/runtime-web/src/index.ts:1184-1538  pkg:npm/@torrentfx/runtime-web  score 0.30  via `BrowserWebSocketTransport::dispatch` <- DEFINES (torrentfx/web/framework/runtime-web/src/index.ts:1273-1295, 1.00)  
`class BrowserWebSocketTransport implements TransportLike`  
members 5  
calls 55: `BrowserBus::disconnectTransport` (torrentfx/web/framework/runtime-web/src/index.ts:1414), `tryReadArrowChunkHeader` (torrentfx/web/framework/runtime-web/src/index.ts:1478), `readProtoBytes` (torrentfx/web/framework/runtime-web/src/index.ts:1350) … 52 more  
importers 1: `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:22-29)  
used_by 6: `bootstrap` (torrentfx/web/framework/app/src/index.ts:338), `browser transport conformance::reconnects WebSocket and replays each retained subscription once` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:190), `appendWithWebSocket` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:287) … 3 more  
uses_types 2: `ArrowDatasetIngest` (torrentfx/web/framework/runtime-web/src/index.ts:1187), `BrowserBus` (torrentfx/web/framework/runtime-web/src/index.ts:1185)  
torrentfx/web/framework/runtime-web/src/index.ts:1184-1263, cut
```typescript
export class BrowserWebSocketTransport implements TransportLike {
  readonly #bus: BrowserBus;
  readonly #url: string;
  readonly #arrow: ArrowDatasetIngest;
  #socket: WebSocket | undefined;
  #reconnectTimer: ReturnType<typeof setTimeout> | undefined;
  #reconnectAttempt = 0;
  #stopped = true;

  constructor(bus: BrowserBus, url: string) {
    this.#bus = bus;
    this.#url = url;
    this.#arrow = new ArrowDatasetIngest(bus);
  }

  connect(): void {
    if (!this.#stopped) {
      return;
    }
    this.#stopped = false;
    this.#openSocket();
  }

  #openSocket(): void {
    if (this.#stopped || this.#socket !== undefined) {
      return;
    }

    const socket = new WebSocket(this.#url);
    this.#socket = socket;
    socket.binaryType = "arraybuffer";

    socket.addEventListener("open", () => {
      if (this.#socket !== socket || this.#stopped) {
        return;
      }
      this.#bus.connectTransport(this);
      this.#bus.beginSync();
      this.#sendInit();
    });

    socket.addEventListener("message", (event) => {
      if (this.#socket !== socket || this.#stopped) {
        return;
      }
      if (!(event.data instanceof ArrayBuffer)) {
        return;
      }

      const frame = new Uint8Array(event.data);
      if (frame.length === 0) {
        return;
      }

      if (frame[0] === 0x01) {
        this.#handleProtoFrame(frame.subarray(1));
        return;
      }

      if (frame[0] === 0x02) {
        this.#handleArrowFrame(frame.subarray(1));
      }
    });

    socket.addEventListener("close", () => {
      if (this.#socket !== socket) {
        return;
      }
      this.#socket = undefined;
      if (this.#stopped) {
        return;
      }
      this.#bus.beginSync();
      this.#scheduleReconnect();
    });
  }

  #scheduleReconnect(): void {
    if (this.#stopped || this.#reconnectTimer !== undefined) {
      return;
```

### 47. Method BrowserBus::ingestSignals
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::ingestSignals`  torrentfx/web/framework/runtime-web/src/index.ts:996-1001  pkg:npm/@torrentfx/runtime-web  score 0.29  via `BrowserBus` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:1122, 1.00)  
`ingestSignals(signals: Record<string, unknown>): void`  
callers 26: `applyFrameSync` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:83), `BrowserConnectTransport` (torrentfx/web/framework/runtime-web/src/index.ts:1661), `applyFrameSync` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:94) … 23 more  
torrentfx/web/framework/runtime-web/src/index.ts:996-1001
```typescript
  ingestSignals(signals: Record<string, unknown>): void {
    for (const [name, value] of Object.entries(signals)) {
      this.#emit({ type: "ingest", source: "signals", name, atMs: Date.now() });
      this.#publish(name, value);
    }
  }
```

### 48. Method BrowserBus::rescan
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::rescan`  torrentfx/web/framework/runtime-web/src/index.ts:928-941  pkg:npm/@torrentfx/runtime-web  score 0.29  via `BrowserBus` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:955, 1.00)  
`rescan(root: Element = document.documentElement): void`  
callers 8: `BrowserBus::ingestElements` (torrentfx/web/framework/runtime-web/src/index.ts:1047), `morphFixture` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:396), `BrowserBus DOM binding lifecycle::scans an element patch once without observer replay` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:78) … 5 more  
tests 3: `BrowserBus DOM binding lifecycle::scans an element patch once without observer replay` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:78), `BrowserBus DOM binding lifecycle::unsubscribes a binding when its element leaves the document` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:61), `BrowserBus DOM binding lifecycle::wires a binding added to an existing keyed element by a morph` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:99)  
torrentfx/web/framework/runtime-web/src/index.ts:928-941
```typescript
  rescan(root: Element = document.documentElement): void {
    for (const scanner of this.#scanners) {
      scanner(root, this);
    }

    if (this.#observerInstalled || typeof MutationObserver === "undefined") {
      return;
    }

    this.#observerInstalled = true;
    const observer = new MutationObserver((mutations) => this.#processDomMutations(mutations));
    this.#domObserver = observer;
    observer.observe(document.documentElement, { childList: true, subtree: true });
  }
```

### 49. Method BrowserWebSocketTransport::dispatch
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserWebSocketTransport::dispatch`  torrentfx/web/framework/runtime-web/src/index.ts:1273-1295  pkg:npm/@torrentfx/runtime-web  score 0.29  via `BrowserBus::actionCodec` <- CALLS (torrentfx/web/framework/runtime-web/src/index.ts:1281, 1.00)  
`dispatch(action: string, payload: unknown): void`  
callers 2: `createSocketSessionActor::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:2514), `createBrowserLifecycleActor::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:2449)  
calls 9: `codec` (torrentfx/web/framework/runtime-web/src/index.ts:1283), `encodeProtoField` (torrentfx/web/framework/runtime-web/src/index.ts:1290), `concatBytes` (torrentfx/web/framework/runtime-web/src/index.ts:1286) … 6 more  
torrentfx/web/framework/runtime-web/src/index.ts:1273-1295
```typescript
  dispatch(action: string, payload: unknown): void {
    if (this.#socket?.readyState !== WebSocket.OPEN) {
      return;
    }

    // A registered codec puts binary bytes (e.g. a protobuf command) on
    // the wire; otherwise the payload is JSON. Slices that adopt a binary
    // contract register a codec, leaving no JSON fallback for their actions.
    const codec = this.#bus.actionCodec(action);
    const payloadBytes = codec
      ? codec(payload)
      : new TextEncoder().encode(JSON.stringify(payload ?? null));

    const inner = concatBytes([
      encodeProtoStringField(1, action),
      encodeProtoBytesField(2, payloadBytes),
    ]);
    const envelope = encodeProtoField(4, inner);
    const frame = new Uint8Array(1 + envelope.length);
    frame[0] = 0x01;
    frame.set(envelope, 1);
    this.#socket.send(frame);
  }
```

### 50. File web/ui-catalog-status/src/codec.test.ts
`file:web/ui-catalog-status/src/codec.test.ts`  web/ui-catalog-status/src/codec.test.ts  pkg:npm/@causeway/ui-catalog-status  score 0.28  via `decode` <- CONTAINS (web/ui-catalog-status/src/codec.test.ts:20-22, 1.00)  
members 3  
imports 3: `npm/@causeway/catalog-ui` (web/ui-catalog-status/src/codec.test.ts:9), `npm/vitest` (web/ui-catalog-status/src/codec.test.ts:11), `npm/@bufbuild/protobuf` (web/ui-catalog-status/src/codec.test.ts:8)  

### 51. Function whenBusReady
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::whenBusReady`  torrentfx/web/framework/runtime-web/src/index.ts:2009-2020  pkg:npm/@torrentfx/runtime-web  score 0.28  via `web/ui-explorer-codecs/src/index.ts` -> CALLS (web/ui-explorer-codecs/src/index.ts:122, 1.00)  
`function whenBusReady(callback: BusReadyCallback): void`  
Run `callback` against the framework bus, now if it is already published or as soon as `bootstrap` publishes it. Deterministic and load-order-independent — the replacement for ad-hoc `window.__bus` polling. The callback runs exactly once.  
callers 6: `web/ui-presence/src/index.ts` (web/ui-presence/src/index.ts:69), `web/ui-explorer-codecs/src/index.ts` (web/ui-explorer-codecs/src/index.ts:122), `OntologyExplorerElement::connectedCallback` (web/ui-ontology/src/index.ts:74) … 3 more  
calls 3: `busReadyGlobal` (torrentfx/web/framework/runtime-web/src/index.ts:2010), `DiagnosticsRecorder::push` (torrentfx/web/framework/runtime-web/src/index.ts:2019), `createDiagnosticsRecorder::push` (torrentfx/web/framework/runtime-web/src/index.ts:2019)  
uses_types 1: `BusReadyCallback` (torrentfx/web/framework/runtime-web/src/index.ts:2009)  
torrentfx/web/framework/runtime-web/src/index.ts:2009-2020
```typescript
export function whenBusReady(callback: BusReadyCallback): void {
  const g = busReadyGlobal();
  if (g === undefined) {
    return;
  }
  if (g.__bus !== undefined) {
    callback(g.__bus);
    return;
  }
  g.__busReadyQueue ??= [];
  g.__busReadyQueue.push(callback);
}
```

### 52. Test codec
`typescript:web/ui-semantic-mapping/src/codec.test.ts::codec`  web/ui-semantic-mapping/src/codec.test.ts:21-123  pkg:npm/@causeway/ui-semantic-mapping  score 0.27  via `registerExplorerCodecs::registers a codec for every action and is idempotent` -> CALLS (web/ui-explorer-codecs/src/index.test.ts:129, 1.00)  
`describe("codec", () => { it("converts bindings, mappings, and candidates by identity", () => { const binding = toBinding( create(AttributeBindingSchema, { column: { iri: ` ${iri("col")} ` }, term: { iri: iri("term") }, dataset: { iri: iri("ds") }, columnName: "risk_rtg", termKey: "RiskRating", isPr…`  
members 3  
callers 4: `registerCatalogCodecs::registers a codec for every handled action and is idempotent` (web/ui-catalog-status/src/codec.test.ts:101), `registerExplorerCodecs::registers a codec for every action and is idempotent` (web/ui-explorer-codecs/src/index.test.ts:129), `BrowserConnectTransport::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:1817) … 1 more  
tests 2: `registerCatalogCodecs::registers a codec for every handled action and is idempotent` (web/ui-catalog-status/src/codec.test.ts:101), `registerExplorerCodecs::registers a codec for every action and is idempotent` (web/ui-explorer-codecs/src/index.test.ts:129)  

### 53. Test BrowserBus DOM binding lifecycle::unsubscribes a binding when its element leaves the document
`typescript:torrentfx/web/framework/runtime-web/src/browser-bus.test.ts::BrowserBus DOM binding lifecycle::unsubscribes a binding when its element leaves the document`  torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:56-70  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus::set` -> TESTED_BY (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:62, 1.00)  
`it("unsubscribes a binding when its element leaves the document", async () => { document.body.innerHTML = '<span id="status" data-text="job.status"></span>'; const element = document.querySelector("#status") as HTMLElement; const bus = new BrowserBus(); bus.registerScanner(scanTextBindings); bus.res…`  
calls 4: `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:67), `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:62), `BrowserBus::rescan` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:61) … 1 more  
tests_for 3: `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:62), `BrowserBus::registerScanner` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:60), `BrowserBus::rescan` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:61)  
uses_types 1: `BrowserBus` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:59)  

### 54. Function disposeElementBindings
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::disposeElementBindings`  torrentfx/web/framework/runtime-web/src/index.ts:2062-2073  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:947, 1.00)  
`function disposeElementBindings(root: Element): void`  
callers 1: `BrowserBus` (torrentfx/web/framework/runtime-web/src/index.ts:947)  
calls 7: `querySelfAndDescendants` (torrentfx/web/framework/runtime-web/src/index.ts:2063), `createBrowserLifecycleActor::get` (torrentfx/web/framework/runtime-web/src/index.ts:2064), `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:2064) … 4 more  

### 55. File torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts
`file:torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts`  torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus` <- IMPORTS (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:17-22, 1.00)  
members 4  
calls 1: `traceId` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:52)  
imports 9: `BrowserBus` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:17-22), `pipePageLifecycleToDiagnostics` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:268), `BusEvent` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:17-22) … 6 more  
uses_types 1: `TraceId` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:52)  

### 56. File torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts
`file:torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts`  torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus` <- IMPORTS (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts:15, 1.00)  
members 5  
imports 9: `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts:13), `Frame` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts:14), `applyFrameSync` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts:13) … 6 more  

### 57. File torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts
`file:torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts`  torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus` <- IMPORTS (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:15, 1.00)  
members 2  
importers 3: `torrentfx/web/framework/runtime-web/scripts/bench-codec.ts` (torrentfx/web/framework/runtime-web/scripts/bench-codec.ts:17), `torrentfx/web/framework/runtime-web/src/effect-connect.ts` (torrentfx/web/framework/runtime-web/src/effect-connect.ts:24), `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts:13)  
imports 5: `torrentfx/web/framework/runtime-web/src/index.ts` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:15), `npm/effect` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:12), `BrowserBus` (torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:15) … 2 more  

### 58. File torrentfx/web/framework/runtime-web/src/effect-connect.test.ts
`file:torrentfx/web/framework/runtime-web/src/effect-connect.test.ts`  torrentfx/web/framework/runtime-web/src/effect-connect.test.ts  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus` <- IMPORTS (torrentfx/web/framework/runtime-web/src/effect-connect.test.ts:15, 1.00)  
members 12  
imports 6: `connectEffect` (torrentfx/web/framework/runtime-web/src/effect-connect.test.ts:14), `torrentfx/web/framework/runtime-web/src/effect-connect.ts` (torrentfx/web/framework/runtime-web/src/effect-connect.test.ts:14), `torrentfx/web/framework/runtime-web/src/index.ts` (torrentfx/web/framework/runtime-web/src/effect-connect.test.ts:15) … 3 more  

### 59. File torrentfx/web/framework/runtime-web/src/effect-connect.ts
`file:torrentfx/web/framework/runtime-web/src/effect-connect.ts`  torrentfx/web/framework/runtime-web/src/effect-connect.ts  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus` <- IMPORTS (torrentfx/web/framework/runtime-web/src/effect-connect.ts:27, 1.00)  
members 3  
importers 4: `torrentfx/web/framework/runtime-web/src/effect-connect.realws.test.ts` (torrentfx/web/framework/runtime-web/src/effect-connect.realws.test.ts:33), `torrentfx/web/framework/runtime-web/src/effect-connect.test.ts` (torrentfx/web/framework/runtime-web/src/effect-connect.test.ts:14), `torrentfx/web/framework/runtime-web/scripts/demo-reconnect.ts` (torrentfx/web/framework/runtime-web/scripts/demo-reconnect.ts:17) … 1 more  
imports 11: `BrowserBus` (torrentfx/web/framework/runtime-web/src/effect-connect.ts:27), `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts` (torrentfx/web/framework/runtime-web/src/effect-connect.ts:24), `applyFrameToBus` (torrentfx/web/framework/runtime-web/src/effect-connect.ts:24) … 8 more  

### 60. File torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts
`file:torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts`  torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus` <- IMPORTS (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:22-29, 1.00)  
members 34  
designs 1: `docs/rfd/RFD-fd0t8b1v-ui-state-signaling-and-session-transport.md` (docs/rfd/RFD-fd0t8b1v-ui-state-signaling-and-session-transport.md:162)  
imports 10: `npm/vitest` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:5), `ArrowDelivery` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:22-29), `BrowserWebSocketTransport` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:22-29) … 7 more  
uses_types 1: `ChunkLayout` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:42)  

### 61. Test pipePageLifecycleToDiagnostics::emits a page.lifecycle record on each tag change
`typescript:torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts::pipePageLifecycleToDiagnostics::emits a page.lifecycle record on each tag change`  torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:306-326  pkg:npm/@torrentfx/runtime-web  score 0.27  via `BrowserBus::set` -> TESTED_BY (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:312, 1.00)  
`it("emits a page.lifecycle record on each tag change", () => { const { actor, set } = makeStubLifecycle({ tag: "loading", readyState: "loading" }); const buffer = new RingBuffer<DiagnosticEvent>(16); const recorder = createDiagnosticsRecorder(buffer); pipePageLifecycleToDiagnostics(actor, recorder, …`  
calls 8: `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:313), `makeStubLifecycle` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:307), `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:314) … 5 more  
tests_for 6: `makeStubLifecycle` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:307), `pipePageLifecycleToDiagnostics` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:310), `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:312) … 3 more  
uses_types 1: `RingBuffer` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:308)  

### 62. TypeAlias Dict
`typescript:web/ui-explorer-codecs/src/index.ts::Dict`  web/ui-explorer-codecs/src/index.ts:29  pkg:npm/@causeway/ui-explorer-codecs  score 0.27  via `web/ui-explorer-codecs/src/index.ts` -> CONTAINS (web/ui-explorer-codecs/src/index.ts:29, 1.00)  
`type Dict = Record<string, unknown>;`  
web/ui-explorer-codecs/src/index.ts:29-29
```typescript
type Dict = Record<string, unknown>;
```

### 63. Method registerExplorerCodecs::registers a codec for every action and is idempotent::registerActionCodec
`typescript:web/ui-explorer-codecs/src/index.test.ts::registerExplorerCodecs::registers a codec for every action and is idempotent::registerActionCodec`  web/ui-explorer-codecs/src/index.test.ts:113-115  pkg:npm/@causeway/ui-explorer-codecs  score 0.26  via `registerExplorerCodecs::registers a codec for every action and is idempotent` -> DEFINES (web/ui-explorer-codecs/src/index.test.ts:113-115, 1.00)  
`registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array)`  

### 64. Package npm/@torrentfx/runtime-web
`pkg:npm/@torrentfx/runtime-web`  torrentfx/web/framework/runtime-web/package.json:2  pkg:npm/@torrentfx/runtime-web  score 0.26  via `web/ui-explorer-codecs/src/index.ts` -> IMPORTS (web/ui-explorer-codecs/src/index.ts:16, 1.00)  
`npm/@torrentfx/runtime-web`  
members 35  
dependents 15: `npm/@causeway/ui-explorer-codecs` (web/ui-explorer-codecs/package.json:20), `npm/@causeway/showcase` (apps-web/showcase/package.json:27), `npm/@causeway/ui-roadmap` (web/ui-roadmap/package.json:22) … 12 more  
dependencies 15: `npm/@connectrpc/connect-web` (torrentfx/web/framework/runtime-web/package.json:21), `npm/apache-arrow` (torrentfx/web/framework/runtime-web/package.json:22), `npm/@torrentfx/core` (torrentfx/web/framework/runtime-web/package.json:36) … 12 more  
importers 22: `web/ui-catalog-filter/src/index.ts` (web/ui-catalog-filter/src/index.ts:18), `web/ui-semantic-mapping/src/index.ts` (web/ui-semantic-mapping/src/index.ts:6), `web/ui-request/src/index.ts` (web/ui-request/src/index.ts:13) … 19 more  

### 65. Test pipePageLifecycleToDiagnostics::collapses repeated identical tags into a single emission
`typescript:torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts::pipePageLifecycleToDiagnostics::collapses repeated identical tags into a single emission`  torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:328-343  pkg:npm/@torrentfx/runtime-web  score 0.26  via `BrowserBus::set` -> TESTED_BY (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:334, 1.00)  
`it("collapses repeated identical tags into a single emission", () => { const { actor, set } = makeStubLifecycle({ tag: "loading", readyState: "loading" }); const buffer = new RingBuffer<DiagnosticEvent>(16); const recorder = createDiagnosticsRecorder(buffer); pipePageLifecycleToDiagnostics(actor, re…`  
calls 8: `makeStubLifecycle` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:329), `createDiagnosticsRecorder` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:331), `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:334) … 5 more  
tests_for 6: `createDiagnosticsRecorder` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:331), `pipePageLifecycleToDiagnostics` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:332), `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:334) … 3 more  
uses_types 1: `RingBuffer` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:330)  

### 66. Document Design log
`doc:docs/DESIGN-LOG.md`  docs/DESIGN-LOG.md:1-3800  pkg:npm/causeway-monorepo  score 0.26  via `web/ui-explorer-codecs/src/index.test.ts` -> DOCUMENTED_BY (docs/DESIGN-LOG.md:646, 1.00)  
`Design log`  
<!-- Generated by `mise run governance:design-log` from `Design:` commit trailers (RFD-vjzs0qzv). Do not edit; regenerate. -->  
headings: ADR 25frqh42: ADR-0023: Immutable composite gateway policy bundles
RFD 2n2tcfe2: Operational resilience: stuck work, poison records, and the operator's surface
ADR bn90xr5m: Authentication types make illegal callers unrepresentable
RFD d4v9pg0h: Identity, authorization, and enforcement
RFD fpg4tnq2: Ephemeral infrastructure for CI integration tests
RFD j1jabh49: Environment, scope and secret confi …  
documents_for 1183: `torrentfx/packages-node/causeway-platform/src/secret.ts` (docs/DESIGN-LOG.md:1726), `crates/session-domain/src/session.rs` (docs/DESIGN-LOG.md:621), `apps/causeway-server/src/routes/pages/catalog/list/tests.rs` (docs/DESIGN-LOG.md:729) … 1180 more  

### 67. Test BrowserBus DOM binding lifecycle::wires a binding added to an existing keyed element by a morph
`typescript:torrentfx/web/framework/runtime-web/src/browser-bus.test.ts::BrowserBus DOM binding lifecycle::wires a binding added to an existing keyed element by a morph`  torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:91-110  pkg:npm/@torrentfx/runtime-web  score 0.26  via `BrowserBus::set` -> TESTED_BY (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:100, 1.00)  
`it("wires a binding added to an existing keyed element by a morph", () => { document.body.innerHTML = ` <section id="panel"> <span id="status">unbound</span> </section>`; const status = document.querySelector("#status") as Element; const bus = new BrowserBus(); bus.registerScanner(scanTextBindings);…`  
calls 4: `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:100), `BrowserBus::registerScanner` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:98), `BrowserBus::ingestElements` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:102) … 1 more  
tests_for 4: `BrowserBus::registerScanner` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:98), `BrowserBus::rescan` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:99), `BrowserBus::ingestElements` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:102) … 1 more  
uses_types 1: `BrowserBus` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:97)  

### 68. Test pipePageLifecycleToDiagnostics::drops the `hidden` lifecycle tag (not a PagePhase
`typescript:torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts::pipePageLifecycleToDiagnostics::drops the `hidden` lifecycle tag (not a PagePhase`  torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:345-359  pkg:npm/@torrentfx/runtime-web  score 0.24  via `BrowserBus::set` -> TESTED_BY (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:351, 1.00)  
`it("drops the `hidden` lifecycle tag (not a PagePhase)", () => { const { actor, set } = makeStubLifecycle({ tag: "interactive" }); const buffer = new RingBuffer<DiagnosticEvent>(16); const recorder = createDiagnosticsRecorder(buffer); pipePageLifecycleToDiagnostics(actor, recorder); set({ tag: "hidd…`  
calls 7: `makeStubLifecycle` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:346), `pipePageLifecycleToDiagnostics` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:349), `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:352) … 4 more  
tests_for 6: `createDiagnosticsRecorder` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:348), `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:351), `pipePageLifecycleToDiagnostics` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:349) … 3 more  
uses_types 1: `RingBuffer` (torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:347)  

### 69. Method BrowserBus::invoke
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::invoke`  torrentfx/web/framework/runtime-web/src/index.ts:785-890  pkg:npm/@torrentfx/runtime-web  score 0.24  via `BrowserBus` -> DEFINES (torrentfx/web/framework/runtime-web/src/index.ts:785-890, 1.00)  
`invoke<TRes = unknown>( kind: string, payload: unknown = null, opts: InvokeOptions = {}, ): Promise<TRes>`  
Correlated request/reply over the bus. Sends a `Command` envelope and resolves when the matching `Event` arrives. Honors `signal` to abort (sends a `command.cancel` Command back over the wire) and `timeoutMs` to time out. (doc continues, 598 bytes)  
members 2  
calls 17: `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:878), `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:805), `BrowserBus::invoke::finish` (torrentfx/web/framework/runtime-web/src/index.ts:865) … 14 more  
uses_types 3: `TraceId` (torrentfx/web/framework/runtime-web/src/index.ts:802), `InvokeOptions` (torrentfx/web/framework/runtime-web/src/index.ts:788), `CorrelationIds` (torrentfx/web/framework/runtime-web/src/index.ts:804)  

### 70. TypeAlias FilterBus
`typescript:web/ui-catalog-filter/src/index.ts::FilterBus`  web/ui-catalog-filter/src/index.ts:35-38  pkg:npm/@causeway/ui-catalog-filter  score 0.24  via `boot` -> USES_TYPE (web/ui-catalog-filter/src/index.ts:96, 1.00)  
`type FilterBus = { dispatch(action: string, payload?: unknown): void; registerActionCodec(action: string, codec: (payload: unknown) => Uint8Array): void; };`  
members 2  
used_by 2: `dispatchFilter` (web/ui-catalog-filter/src/index.ts:40), `boot` (web/ui-catalog-filter/src/index.ts:96)  

### 71. Test encodeCatalogCommand round-trip::omits sort when no sort column is present
`typescript:web/ui-catalog-status/src/codec.test.ts::encodeCatalogCommand round-trip::omits sort when no sort column is present`  web/ui-catalog-status/src/codec.test.ts:51-57  pkg:npm/@causeway/ui-catalog-status  score 0.23  via `decode` <- CALLS (web/ui-catalog-status/src/codec.test.ts:52, 1.00)  
`it("omits sort when no sort column is present", () => { const message = decode("catalog.get-block", { "start-row": 0, "end-row": 50 }); expect(message.command.case).toBe("getBlock"); if (message.command.case === "getBlock") { expect(message.command.value.sort).toBeUndefined(); } })`  
calls 1: `decode` (web/ui-catalog-status/src/codec.test.ts:52)  
tests_for 1: `decode` (web/ui-catalog-status/src/codec.test.ts:52)  

### 72. Test encodeCatalogCommand round-trip::carries applyFilter fields (snake_case payload → proto) through the wire
`typescript:web/ui-catalog-status/src/codec.test.ts::encodeCatalogCommand round-trip::carries applyFilter fields (snake_case payload → proto) through the wire`  web/ui-catalog-status/src/codec.test.ts:59-73  pkg:npm/@causeway/ui-catalog-status  score 0.23  via `decode` <- CALLS (web/ui-catalog-status/src/codec.test.ts:60, 1.00)  
`it("carries applyFilter fields (snake_case payload → proto) through the wire", () => { const message = decode("catalog.filter", { q: "revenue", domain_l1: "finance", domain_l2: "finance.reporting", asset_type: "table", }); expect(message.command.case).toBe("applyFilter"); if (message.command.case …`  
calls 1: `decode` (web/ui-catalog-status/src/codec.test.ts:60)  
tests_for 1: `decode` (web/ui-catalog-status/src/codec.test.ts:60)  

### 73. Test encodeCatalogCommand round-trip::carries getBlock row bounds and sort through the wire
`typescript:web/ui-catalog-status/src/codec.test.ts::encodeCatalogCommand round-trip::carries getBlock row bounds and sort through the wire`  web/ui-catalog-status/src/codec.test.ts:32-49  pkg:npm/@causeway/ui-catalog-status  score 0.23  via `decode` <- CALLS (web/ui-catalog-status/src/codec.test.ts:33, 1.00)  
`it("carries getBlock row bounds and sort through the wire", () => { const message = decode("catalog.get-block", { "start-row": 0, "end-row": 50, "request-id": "catalog-request-7", generation: 3, sortModel: [{ column: "name", descending: true }], }); expect(message.command.case).toBe("getBlock"); if …`  
calls 1: `decode` (web/ui-catalog-status/src/codec.test.ts:33)  
tests_for 1: `decode` (web/ui-catalog-status/src/codec.test.ts:33)  

### 74. TypeAlias BlockPayload
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::BlockPayload`  bindings-ts/causeway-catalog-ui/src/codec.ts:18-24  pkg:npm/@causeway/catalog-ui  score 0.22  via `bindings-ts/causeway-catalog-ui/src/codec.ts` -> CONTAINS (bindings-ts/causeway-catalog-ui/src/codec.ts:18-24, 1.00)  
`type BlockPayload = { "start-row"?: number; "end-row"?: number; "request-id"?: string; generation?: number; sortModel?: SortEntry[]; };`  

### 75. TypeAlias FilterPayload
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::FilterPayload`  bindings-ts/causeway-catalog-ui/src/codec.ts:16  pkg:npm/@causeway/catalog-ui  score 0.22  via `bindings-ts/causeway-catalog-ui/src/codec.ts` -> CONTAINS (bindings-ts/causeway-catalog-ui/src/codec.ts:16, 1.00)  
`type FilterPayload = { q?: string; domain_l1?: string; domain_l2?: string; asset_type?: string };`  

### 76. TypeAlias SortEntry
`typescript:bindings-ts/causeway-catalog-ui/src/codec.ts::SortEntry`  bindings-ts/causeway-catalog-ui/src/codec.ts:17  pkg:npm/@causeway/catalog-ui  score 0.22  via `bindings-ts/causeway-catalog-ui/src/codec.ts` -> CONTAINS (bindings-ts/causeway-catalog-ui/src/codec.ts:17, 1.00)  
`type SortEntry = { column?: string; descending?: boolean };`  
bindings-ts/causeway-catalog-ui/src/codec.ts:17-17
```typescript
type SortEntry = { column?: string; descending?: boolean };
```

### 77. Function encodeProtoField
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::encodeProtoField`  torrentfx/web/framework/runtime-web/src/index.ts:2364-2366  pkg:npm/@torrentfx/runtime-web  score 0.22  via `BrowserConnectTransport::dispatch` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:1823, 1.00)  
`function encodeProtoField(fieldNumber: number, data: Uint8Array): Uint8Array`  
callers 8: `BrowserConnectTransport::sendInvoke` (torrentfx/web/framework/runtime-web/src/index.ts:1835), `BrowserConnectTransport::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:1823), `BrowserWebSocketTransport` (torrentfx/web/framework/runtime-web/src/index.ts:1338) … 5 more  
calls 3: `encodeVarint` (torrentfx/web/framework/runtime-web/src/index.ts:2365), `encodeVarint` (torrentfx/web/framework/runtime-web/src/index.ts:2365), `concatBytes` (torrentfx/web/framework/runtime-web/src/index.ts:2365)  

### 78. Package npm/vitest
`pkg:npm/vitest`  pkg:npm/vitest  score 0.22  via `web/ui-explorer-codecs/src/index.test.ts` -> IMPORTS (web/ui-explorer-codecs/src/index.test.ts:12, 1.00)  
dependents 21: `npm/@causeway/ui-vendor` (web/ui-vendor/package.json:20), `npm/@causeway/ui-presence` (web/ui-presence/package.json:26), `npm/@causeway/framework` (torrentfx/web/framework/app/package.json:27) … 18 more  
importers 74: `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:5), `services-node/relay-grpc/tests/config.test.ts` (services-node/relay-grpc/tests/config.test.ts:6), `torrentfx/web/framework/core/vitest.config.ts` (torrentfx/web/framework/core/vitest.config.ts:1) … 71 more  

### 79. Function encodeProtoBytesField
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::encodeProtoBytesField`  torrentfx/web/framework/runtime-web/src/index.ts:2372-2374  pkg:npm/@torrentfx/runtime-web  score 0.22  via `BrowserConnectTransport::dispatch` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:1821, 1.00)  
`function encodeProtoBytesField(fieldNumber: number, value: Uint8Array): Uint8Array`  
callers 4: `BrowserConnectTransport::sendInvoke` (torrentfx/web/framework/runtime-web/src/index.ts:1831), `BrowserConnectTransport::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:1821), `BrowserWebSocketTransport::sendInvoke` (torrentfx/web/framework/runtime-web/src/index.ts:1309) … 1 more  
calls 1: `encodeProtoField` (torrentfx/web/framework/runtime-web/src/index.ts:2373)  

### 80. Function encodeProtoStringField
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::encodeProtoStringField`  torrentfx/web/framework/runtime-web/src/index.ts:2368-2370  pkg:npm/@torrentfx/runtime-web  score 0.22  via `BrowserConnectTransport::dispatch` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:1820, 1.00)  
`function encodeProtoStringField(fieldNumber: number, value: string): Uint8Array`  
callers 8: `BrowserWebSocketTransport::sendInvoke` (torrentfx/web/framework/runtime-web/src/index.ts:1314), `BrowserWebSocketTransport::sendInvoke` (torrentfx/web/framework/runtime-web/src/index.ts:1310), `BrowserConnectTransport::sendInvoke` (torrentfx/web/framework/runtime-web/src/index.ts:1833) … 5 more  
calls 1: `encodeProtoField` (torrentfx/web/framework/runtime-web/src/index.ts:2369)  

### 81. TypeAlias CatalogFilterCommand
`typescript:web/ui-catalog-filter/src/decider.ts::CatalogFilterCommand`  web/ui-catalog-filter/src/decider.ts:47  pkg:npm/@causeway/ui-catalog-filter  score 0.22  via `boot` -> USES_TYPE (web/ui-catalog-filter/src/index.ts:119, 1.00)  
`type CatalogFilterCommand = { readonly type: "SendFilter"; readonly filter: Filter };`  
importers 1: `web/ui-catalog-filter/src/index.ts` (web/ui-catalog-filter/src/index.ts:21-30)  
used_by 2: `boot` (web/ui-catalog-filter/src/index.ts:119), `send` (web/ui-catalog-filter/src/decider.ts:65)  
uses_types 1: `Filter` (web/ui-catalog-filter/src/decider.ts:47)  

### 82. Method BrowserBus::acquireDomainStream
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::acquireDomainStream`  torrentfx/web/framework/runtime-web/src/index.ts:689-712  pkg:npm/@torrentfx/runtime-web  score 0.19  via `BrowserBus` -> DEFINES (torrentfx/web/framework/runtime-web/src/index.ts:689-712, 1.00)  
`acquireDomainStream(stream: string): () => void`  
Retain a server domain stream until the returned release runs.  
callers 5: `browser transport conformance::reconnects ConnectRPC, sends init after each hello, and replays subscriptions once` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:231), `BrowserBus domain stream references::subscribes once and releases only after the final consumer` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:19), `BrowserBus domain stream references::subscribes once and releases only after the final consumer` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:20) … 2 more  
calls 6: `BrowserBus::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:706), `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:690), `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:702) … 3 more  
tests 4: `browser transport conformance::reconnects WebSocket and replays each retained subscription once` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:187), `browser transport conformance::reconnects ConnectRPC, sends init after each hello, and replays subscriptions once` (torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:231), `BrowserBus domain stream references::re-subscribes retained streams after transport synchronization` (torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:44) … 1 more  

### 83. Package npm/@causeway/catalog-ui
`pkg:npm/@causeway/catalog-ui`  bindings-ts/causeway-catalog-ui/package.json:2  pkg:npm/@causeway/catalog-ui  score 0.19  via `bindings-ts/causeway-catalog-ui/src/codec.ts` <- CONTAINS (bindings-ts/causeway-catalog-ui/package.json:2, 1.00)  
`npm/@causeway/catalog-ui`  
members 6  
dependents 2: `npm/@causeway/ui-catalog-status` (web/ui-catalog-status/package.json:19), `npm/@causeway/ui-catalog-filter` (web/ui-catalog-filter/package.json:18)  
dependencies 2: `npm/typescript` (bindings-ts/causeway-catalog-ui/package.json:21), `npm/@bufbuild/protobuf` (bindings-ts/causeway-catalog-ui/package.json:18)  
importers 3: `web/ui-catalog-status/src/codec.test.ts` (web/ui-catalog-status/src/codec.test.ts:9), `web/ui-catalog-status/src/index.ts` (web/ui-catalog-status/src/index.ts:21), `web/ui-catalog-filter/src/index.ts` (web/ui-catalog-filter/src/index.ts:14)  

### 84. Function concatBytes
`typescript:torrentfx/web/framework/runtime-web/src/arrow-append.ts::concatBytes`  torrentfx/web/framework/runtime-web/src/arrow-append.ts:221-229  pkg:npm/@torrentfx/runtime-web  score 0.19  via `BrowserConnectTransport::dispatch` -> CALLS (torrentfx/web/framework/runtime-web/src/index.ts:1819, 1.00)  
`function concatBytes(parts: readonly Uint8Array[]): Uint8Array`  
callers 5: `BrowserWebSocketTransport::sendInvoke` (torrentfx/web/framework/runtime-web/src/index.ts:1307), `encodeProtoField` (torrentfx/web/framework/runtime-web/src/index.ts:2365), `BrowserConnectTransport::dispatch` (torrentfx/web/framework/runtime-web/src/index.ts:1819) … 2 more  
calls 1: `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/arrow-append.ts:225)  

### 85. TypeAlias TaxonomyNode
`typescript:web/ui-catalog-filter/src/decider.ts::TaxonomyNode`  web/ui-catalog-filter/src/decider.ts:13-18  pkg:npm/@causeway/ui-catalog-filter  score 0.18  via `boot` -> USES_TYPE (web/ui-catalog-filter/src/index.ts:179, 1.00)  
`type TaxonomyNode = { readonly path: string; readonly label: string; readonly parent_path?: string | null; readonly sort_order?: number | null; };`  
importers 2: `web/ui-catalog-filter/src/decider.test.ts` (web/ui-catalog-filter/src/decider.test.ts:4-12), `web/ui-catalog-filter/src/index.ts` (web/ui-catalog-filter/src/index.ts:21-30)  
used_by 2: `bySortOrder` (web/ui-catalog-filter/src/decider.ts:132), `boot` (web/ui-catalog-filter/src/index.ts:179)  

### 86. Function boot::onAssetTypeChange
`typescript:web/ui-catalog-filter/src/index.ts::boot::onAssetTypeChange`  web/ui-catalog-filter/src/index.ts:130-131  pkg:npm/@causeway/ui-catalog-filter  score 0.18  via `boot` -> DEFINES (web/ui-catalog-filter/src/index.ts:130-131, 1.00)  
`onAssetTypeChange = () => store.accept({ type: "AssetTypeSelected" as const, value: atSel.value })`  
calls 2: `createMviEngine::accept` (web/ui-catalog-filter/src/index.ts:131), `MviEngine::accept` (web/ui-catalog-filter/src/index.ts:131)  

### 87. Function boot::onL1Change
`typescript:web/ui-catalog-filter/src/index.ts::boot::onL1Change`  web/ui-catalog-filter/src/index.ts:128  pkg:npm/@causeway/ui-catalog-filter  score 0.18  via `boot` -> DEFINES (web/ui-catalog-filter/src/index.ts:128, 1.00)  
`onL1Change = () => store.accept({ type: "L1Selected" as const, path: l1Sel.value })`  
── DOM → intents ────────────────────────────────────────────────  
calls 2: `MviEngine::accept` (web/ui-catalog-filter/src/index.ts:128), `createMviEngine::accept` (web/ui-catalog-filter/src/index.ts:128)  

### 88. Function boot::onL2Change
`typescript:web/ui-catalog-filter/src/index.ts::boot::onL2Change`  web/ui-catalog-filter/src/index.ts:129  pkg:npm/@causeway/ui-catalog-filter  score 0.18  via `boot` -> DEFINES (web/ui-catalog-filter/src/index.ts:129, 1.00)  
`onL2Change = () => store.accept({ type: "L2Selected" as const, path: l2Sel.value })`  
calls 2: `MviEngine::accept` (web/ui-catalog-filter/src/index.ts:129), `createMviEngine::accept` (web/ui-catalog-filter/src/index.ts:129)  

### 89. Function boot::onSearchInput
`typescript:web/ui-catalog-filter/src/index.ts::boot::onSearchInput`  web/ui-catalog-filter/src/index.ts:136-142  pkg:npm/@causeway/ui-catalog-filter  score 0.18  via `boot` -> DEFINES (web/ui-catalog-filter/src/index.ts:136-142, 1.00)  
`onSearchInput = () => { clearTimeout(searchTimer); searchTimer = setTimeout( () => store.accept({ type: "SearchChanged", q: searchEl.value }), 200, ); }`  
calls 2: `MviEngine::accept` (web/ui-catalog-filter/src/index.ts:139), `createMviEngine::accept` (web/ui-catalog-filter/src/index.ts:139)  

### 90. Method BrowserBus::onDomainEvent
`typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::onDomainEvent`  torrentfx/web/framework/runtime-web/src/index.ts:675-686  pkg:npm/@torrentfx/runtime-web  score 0.18  via `BrowserBus` -> DEFINES (torrentfx/web/framework/runtime-web/src/index.ts:675-686, 1.00)  
`onDomainEvent(stream: string, callback: DomainEventSubscriber): () => void`  
Subscribe to server-pushed domain events on `stream` (bus.proto DomainEvent, RFD-0113). Returns an unsubscribe function. Consumers apply `classifyEnvelope` against the envelope's `seq` to dedupe and detect gaps (item 10).  
calls 3: `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/index.ts:682), `BrowserBus::get` (torrentfx/web/framework/runtime-web/src/index.ts:676), `BrowserBus::set` (torrentfx/web/framework/runtime-web/src/index.ts:679)  
uses_types 1: `DomainEventSubscriber` (torrentfx/web/framework/runtime-web/src/index.ts:675)  

### 91. File bindings-ts/causeway-catalog-ui/src/index.ts
`file:bindings-ts/causeway-catalog-ui/src/index.ts`  bindings-ts/causeway-catalog-ui/src/index.ts  pkg:npm/@causeway/catalog-ui  score 0.17  via `bindings-ts/causeway-catalog-ui/src/codec.ts` -> IMPORTS (bindings-ts/causeway-catalog-ui/src/codec.ts:14, 1.00)  
importers 1: `bindings-ts/causeway-catalog-ui/src/codec.ts` (bindings-ts/causeway-catalog-ui/src/codec.ts:14)  

### 92. File torrentfx/web/framework/runtime-web/src/arrow-append.ts
`file:torrentfx/web/framework/runtime-web/src/arrow-append.ts`  torrentfx/web/framework/runtime-web/src/arrow-append.ts  pkg:npm/@torrentfx/runtime-web  score 0.17  via `torrentfx/web/framework/runtime-web/src/index.ts` -> IMPORTS (torrentfx/web/framework/runtime-web/src/index.ts:34, 1.00)  
members 9  
importers 2: `torrentfx/web/framework/runtime-web/src/index.ts` (torrentfx/web/framework/runtime-web/src/index.ts:34), `torrentfx/web/framework/runtime-web/src/arrow-append.test.ts` (torrentfx/web/framework/runtime-web/src/arrow-append.test.ts:13)  
imports 3: `torrentfx/web/framework/runtime-web/src/index.ts` (torrentfx/web/framework/runtime-web/src/arrow-append.ts:21), `npm/apache-arrow` (torrentfx/web/framework/runtime-web/src/arrow-append.ts:19), `ArrowChunkHeader` (torrentfx/web/framework/runtime-web/src/arrow-append.ts:21)  

## Left out

28 included nodes have no source window for want of budget.  
- `doc:docs/rfd/RFD-fd0t8b1v-ui-state-signaling-and-session-transport.md` Design 0.17 (over budget, 396 tokens)
- `typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserBus::subscribe` Method 0.16 (over budget, 304 tokens)
- `typescript:torrentfx/web/framework/runtime-web/src/index.ts::BrowserWebSocketTransport::sendInvoke` Method 0.16 (over budget, 272 tokens)
- and 164 more

## Unresolved references in the seeds' files

- `BigInt` at bindings-ts/causeway-catalog-ui/src/codec.ts:28 (no-declaration)
- `isFinite` at bindings-ts/causeway-catalog-ui/src/codec.ts:28 (no-declaration)
- `trunc` at bindings-ts/causeway-catalog-ui/src/codec.ts:28 (no-declaration)
- `takeRecords` at torrentfx/web/framework/runtime-web/src/index.ts:1029 (no-declaration)
- `from` at torrentfx/web/framework/runtime-web/src/index.ts:1035 (no-declaration)
- `querySelectorAll` at torrentfx/web/framework/runtime-web/src/index.ts:1035 (no-declaration)
- `takeRecords` at torrentfx/web/framework/runtime-web/src/index.ts:1044 (no-declaration)
- `reload` at torrentfx/web/framework/runtime-web/src/index.ts:1057 (no-declaration)
- `getAttribute` at torrentfx/web/framework/runtime-web/src/index.ts:1069 (no-declaration)
- `setAttribute` at torrentfx/web/framework/runtime-web/src/index.ts:1077 (no-declaration)

