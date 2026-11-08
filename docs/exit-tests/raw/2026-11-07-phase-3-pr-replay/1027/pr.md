<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `33ced4d72079`, dataset at `904065a7cecb`: **9 files, 38 changed symbols, 134 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (38)**

| Symbol | Kind | Where |
|---|---|---|
| `catalog_scripts` | Function | `apps/causeway-server-web/src/views/platform/catalog.rs:81-105` |
| `tests` | Module | `apps/causeway-server-web/src/views/platform/catalog.rs:108-139` |
| `tests::catalog_init_waits_for_its_codec` | Test | `apps/causeway-server-web/src/views/platform/catalog.rs:114-127` |
| `tests` | Module | `apps/causeway-server-web/src/views/platform/explorer/structured/pipeline_scripts.rs:276-353` |
| `tests::commands_sent_at_load_wait_for_their_codec` | Test | `apps/causeway-server-web/src/views/platform/explorer/structured/pipeline_scripts.rs:324-342` |
| `SEGMENT` | Const | `apps/causeway-server-web/src/views/platform/explorer/structured/pipeline_scripts/actions.rs:4-46` |
| `SEGMENT` | Const | `apps/causeway-server-web/src/views/platform/explorer/structured/pipeline_scripts/ready.rs:4-17` |
| `Window::beginSync` | Method | `apps/causeway-server/tests/browser/page-globals.ts:10` |
| `Window::markSynchronized` | Method | `apps/causeway-server/tests/browser/page-globals.ts:11` |
| `datasetLoads` | Function | `apps/causeway-server/tests/browser/ui-ownership-transport.spec.ts:131-134` |
| `bootstrap` | Function | `torrentfx/web/framework/app/src/index.ts:331-438` |
| `BrowserBus action codec readiness` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:142-301` |
| `BrowserBus action codec readiness::closes registration at DOMContentLoaded, after module scripts have run` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:278-300` |
| `BrowserBus action codec readiness::closes registration at DOMContentLoaded, after module scripts have run::addEventListener` | Method | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:282-284` |
| `BrowserBus action codec readiness::codec` | Function | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:158` |
| `BrowserBus action codec readiness::holds a command until its codec registers after synchronization` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:161-173` |
| `BrowserBus action codec readiness::load` | Function | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:159` |
| `BrowserBus action codec readiness::recordingBus` | Function | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:146-156` |
| `BrowserBus action codec readiness::recordingBus::disconnect` | Method | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:153` |
| `BrowserBus action codec readiness::recordingBus::dispatch` | Method | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:150-152` |
| `BrowserBus action codec readiness::replaces a retained command, so a reconnect re-sends only the latest once` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:200-219` |
| `BrowserBus action codec readiness::reports a codec still missing when registration closes, and sends nothing` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:244-265` |
| `BrowserBus action codec readiness::reports at once a command retained after registration closed` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:267-276` |
| `BrowserBus action codec readiness::resends once per synchronization, never once per codec registration` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:186-198` |
| `BrowserBus action codec readiness::sends at synchronization when the codec registered first` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:175-184` |
| `BrowserBus action codec readiness::sends the latest intent retained while the codec was pending` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:221-230` |
| `BrowserBus action codec readiness::waits per action` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:232-242` |
| `ActionCodecState` | TypeAlias | `torrentfx/web/framework/runtime-web/src/index.ts:415-418` |
| `ActionCodecUnavailable` | TypeAlias | `torrentfx/web/framework/runtime-web/src/index.ts:408-412` |
| `ActionCodecWaiter` | TypeAlias | `torrentfx/web/framework/runtime-web/src/index.ts:420-423` |
| `BrowserBus` | Class | `torrentfx/web/framework/runtime-web/src/index.ts:552-1286` |
| `BrowserBus::actionCodecState` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:739-747` |
| `BrowserBus::closeActionCodecRegistration` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:760-776` |
| `BrowserBus::registerActionCodec` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:686-696` |
| `BrowserBus::retainCommand` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:791-814` |
| `BrowserBus::whenActionCodec` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:713-736` |
| `RetainedCommand` | TypeAlias | `torrentfx/web/framework/runtime-web/src/index.ts:425-428` |
| `closeActionCodecsAtDocumentLoad` | Function | `torrentfx/web/framework/runtime-web/src/index.ts:2216-2228` |


**Affected (134 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (109)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `tests::catalog_script_does_not_reload_on_bfcache_restore` | Test | `apps/causeway-server-web/src/views/platform/catalog.rs:130-138` | 1 | 1.00 | CALLS | `apps/causeway-server-web/src/views/platform/catalog.rs:131` |
| `pipeline_js` | Function | `apps/causeway-server-web/src/views/platform/explorer/structured/pipeline_scripts.rs:260-273` | 1 | 1.00 | REFERENCES | `apps/causeway-server-web/src/views/platform/explorer/structured/pipeline_scripts.rs:266` |
| `catalog` | Function | `apps/causeway-server/src/routes/pages/catalog/list.rs:19-59` | 1 | 1.00 | CALLS | `apps/causeway-server/src/routes/pages/catalog/list.rs:55` |
| `installBlockServer` | Function | `apps-web/showcase/src/stories/data-grid.story.tsx:101-125` | 1 | 1.00 | USES_TYPE | `apps-web/showcase/src/stories/data-grid.story.tsx:104` |
| `PostMessageTransport` | Class | `torrentfx/web/framework/app/src/index.ts:122-180` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/app/src/index.ts:123` |
| `PostMessageTransport::constructor` | Method | `torrentfx/web/framework/app/src/index.ts:130-149` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/app/src/index.ts:130` |
| `Window` | Interface | `torrentfx/web/framework/app/src/index.ts:233-240` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/app/src/index.ts:234` |
| `applyPostMessageToBus` | Function | `torrentfx/web/framework/app/src/index.ts:188-219` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/app/src/index.ts:188` |
| `recordingBus` | Function | `torrentfx/web/framework/app/src/post-message.test.ts:22-39` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/app/src/post-message.test.ts:22` |
| `PostMessageTransport` | Class | `torrentfx/web/framework/runtime-vscode-webview/src/index.ts:79-127` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-vscode-webview/src/index.ts:80` |
| `PostMessageTransport::constructor` | Method | `torrentfx/web/framework/runtime-vscode-webview/src/index.ts:85-90` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-vscode-webview/src/index.ts:85` |
| `BrowserBus Arrow delivery metadata::publishes a correlated delivery beside the legacy dataset signal` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:119-139` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:120` |
| `BrowserBus DOM binding lifecycle::scans an element patch once without observer replay` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:77-94` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:81` |
| `BrowserBus DOM binding lifecycle::unsubscribes a binding when its element leaves the document` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:61-75` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:64` |
| `BrowserBus DOM binding lifecycle::wires a binding added to an existing keyed element by a morph` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:96-115` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:102` |
| `BrowserBus domain stream references::re-subscribes retained streams after transport synchronization` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:40-57` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:42` |
| `BrowserBus domain stream references::subscribes once and releases only after the final consumer` | Test | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:13-38` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:15` |
| `makeStubBus` | Function | `torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:28-50` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:29` |
| `applyFrameSync` | Function | `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:40-99` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:40` |
| `applyFrameToBus` | Function | `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:28-33` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:29` |
| `connectEffect` | Function | `torrentfx/web/framework/runtime-web/src/effect-connect.ts:76-86` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/effect-connect.ts:78` |
| `ArrowDatasetIngest` | Class | `torrentfx/web/framework/runtime-web/src/index.ts:1296-1341` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:1297` |
| `ArrowDatasetIngest::constructor` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:1301-1303` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:1301` |
| `BrowserConnectTransport` | Class | `torrentfx/web/framework/runtime-web/src/index.ts:1715-2022` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:1716` |
| `BrowserConnectTransport::constructor` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:1728-1732` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:1728` |
| `BrowserConnectTransport::dispatch` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:1973-1983` | 1 | 1.00 | CALLS | `torrentfx/web/framework/runtime-web/src/index.ts:1976` |
| `BrowserWebSocketTransport` | Class | `torrentfx/web/framework/runtime-web/src/index.ts:1343-1697` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:1344` |
| `BrowserWebSocketTransport::constructor` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:1352-1356` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:1352` |
| `BrowserWebSocketTransport::dispatch` | Method | `torrentfx/web/framework/runtime-web/src/index.ts:1432-1454` | 1 | 1.00 | CALLS | `torrentfx/web/framework/runtime-web/src/index.ts:1442` |
| `BusReadyCallback` | TypeAlias | `torrentfx/web/framework/runtime-web/src/index.ts:2151` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:2151` |
| `BusReadyGlobal` | TypeAlias | `torrentfx/web/framework/runtime-web/src/index.ts:2153-2156` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:2154` |
| `Scanner` | TypeAlias | `torrentfx/web/framework/runtime-web/src/index.ts:444` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:444` |
| `bindActorToBus` | Function | `torrentfx/web/framework/runtime-web/src/index.ts:2716-2724` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:2718` |
| `bootstrap` | Function | `torrentfx/web/framework/runtime-web/src/index.ts:2091-2114` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:2091` |
| `pipeBusEventsToDiagnostics` | Function | `torrentfx/web/framework/runtime-web/src/index.ts:290-338` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:291` |
| `publishBusReady` | Function | `torrentfx/web/framework/runtime-web/src/index.ts:2186-2203` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:2186` |
| `scanAttributeBindings` | Function | `torrentfx/web/framework/runtime-web/src/index.ts:2063-2089` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:2063` |
| `scanTextBindings` | Function | `torrentfx/web/framework/runtime-web/src/index.ts:2043-2061` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/index.ts:2043` |
| `appendWithConnect` | Function | `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:298-317` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:309` |
| `appendWithWebSocket` | Function | `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:284-296` | 1 | 1.00 | USES_TYPE | `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:285` |

… and 69 more in the JSON.

_Files and packages (25)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `apps-web/showcase/src/main.tsx` | File | `apps-web/showcase/src/main.tsx` | 1 | 1.00 | USES_TYPE | `apps-web/showcase/src/main.tsx:19` |
| `apps/causeway-server/tests/browser/ui-ownership-transport.spec.ts` | File | `apps/causeway-server/tests/browser/ui-ownership-transport.spec.ts` | 1 | 1.00 | CALLS | `apps/causeway-server/tests/browser/ui-ownership-transport.spec.ts:139` |
| `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts` | File | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts` | 1 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/src/browser-bus.test.ts:5-10` |
| `torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts` | File | `torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts` | 1 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/src/diagnostics-bridge.test.ts:17-22` |
| `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts` | File | `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts` | 1 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.test.ts:15` |
| `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts` | File | `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts` | 1 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/src/effect-bus-adapter.ts:15` |
| `torrentfx/web/framework/runtime-web/src/effect-connect.test.ts` | File | `torrentfx/web/framework/runtime-web/src/effect-connect.test.ts` | 1 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/src/effect-connect.test.ts:15` |
| `torrentfx/web/framework/runtime-web/src/effect-connect.ts` | File | `torrentfx/web/framework/runtime-web/src/effect-connect.ts` | 1 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/src/effect-connect.ts:27` |
| `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts` | File | `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts` | 1 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/src/transport-conformance.test.ts:22-29` |
| `apps/causeway-server/src/routes/pages.rs` | File | `apps/causeway-server/src/routes/pages.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages.rs:24-26` |
| `apps/causeway-server/src/routes/pages/catalog.rs` | File | `apps/causeway-server/src/routes/pages/catalog.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/routes/pages/catalog.rs:11` |
| `torrentfx/web/framework/app/src/post-message.test.ts` | File | `torrentfx/web/framework/app/src/post-message.test.ts` | 2 | 1.00 | IMPORTS | `torrentfx/web/framework/app/src/post-message.test.ts:14` |
| `torrentfx/web/framework/runtime-web/scripts/bench-codec.ts` | File | `torrentfx/web/framework/runtime-web/scripts/bench-codec.ts` | 2 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/scripts/bench-codec.ts:17` |
| `torrentfx/web/framework/runtime-web/scripts/demo-reconnect.ts` | File | `torrentfx/web/framework/runtime-web/scripts/demo-reconnect.ts` | 2 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/scripts/demo-reconnect.ts:17` |
| `torrentfx/web/framework/runtime-web/src/effect-connect.realws.test.ts` | File | `torrentfx/web/framework/runtime-web/src/effect-connect.realws.test.ts` | 2 | 1.00 | IMPORTS | `torrentfx/web/framework/runtime-web/src/effect-connect.realws.test.ts:33` |
| `apps/causeway-server-web/src/views/platform/explorer/mod.rs` | File | `apps/causeway-server-web/src/views/platform/explorer/mod.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server-web/src/views/platform/explorer/mod.rs:44` |
| `apps/causeway-server-web/src/views/platform/explorer/structured.rs` | File | `apps/causeway-server-web/src/views/platform/explorer/structured.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server-web/src/views/platform/explorer/structured.rs:20` |
| `web/ui-catalog-filter/src/index.ts` | File | `web/ui-catalog-filter/src/index.ts` | 3 | 1.00 | CALLS | `web/ui-catalog-filter/src/index.ts:212` |
| `web/ui-catalog-status/src/index.ts` | File | `web/ui-catalog-status/src/index.ts` | 3 | 1.00 | CALLS | `web/ui-catalog-status/src/index.ts:132` |
| `web/ui-explorer-codecs/src/index.ts` | File | `web/ui-explorer-codecs/src/index.ts` | 3 | 1.00 | CALLS | `web/ui-explorer-codecs/src/index.ts:122` |
| `web/ui-presence/src/index.ts` | File | `web/ui-presence/src/index.ts` | 3 | 1.00 | CALLS | `web/ui-presence/src/index.ts:69` |
| `apps/causeway-server-web/src/views/platform/create_product/assets.rs` | File | `apps/causeway-server-web/src/views/platform/create_product/assets.rs` | 2 | 0.33 | IMPORTS | `apps/causeway-server-web/src/views/platform/create_product/assets.rs:9` |
| `apps/causeway-server-web/src/views/platform/request/assets.rs` | File | `apps/causeway-server-web/src/views/platform/request/assets.rs` | 2 | 0.33 | IMPORTS | `apps/causeway-server-web/src/views/platform/request/assets.rs:9` |
| `tests` | Module | `apps/causeway-server-web/src/views/platform/create_product/assets.rs:12-32` | 2 | 0.33 | IMPORTS | `apps/causeway-server-web/src/views/platform/create_product/assets.rs:13` |
| `tests` | Module | `apps/causeway-server-web/src/views/platform/request/assets.rs:12-35` | 2 | 0.33 | IMPORTS | `apps/causeway-server-web/src/views/platform/request/assets.rs:13` |

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 62321 nodes, 264572 edges · 1865 ms</sub>
