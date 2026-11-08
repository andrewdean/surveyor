<!-- surveyor-pr -->
### Surveyor: what this change touches

Measured against `c6028d860273`, dataset at `ed9a46459f40`: **1 files, 14 changed symbols, 60 affected nodes within 3 hops, 5 rules, 0 violations, 0 failed**; **gate passes**.

**Changed symbols (14)**

| Symbol | Kind | Where |
|---|---|---|
| `Backlog` | Struct | `apps/causeway-server/src/composition/state/session_streams.rs:175-177` |
| `Backlog#impl` | Impl | `apps/causeway-server/src/composition/state/session_streams.rs:179-194` |
| `Backlog::pushed` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:187-189` |
| `Backlog::taken` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:191-193` |
| `ConnectionSink#impl` | Impl | `apps/causeway-server/src/composition/state/session_streams.rs:210-275` |
| `ConnectionSink::offer_replies` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:247-262` |
| `SessionStreams#impl` | Impl | `apps/causeway-server/src/composition/state/session_streams.rs:304-476` |
| `SessionStreams::reply` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:400-414` |
| `SessionStreams::subscribe` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:325-353` |
| `tests` | Module | `apps/causeway-server/src/composition/state/session_streams.rs:583-994` |
| `tests::a_tab_paused_past_the_grace_keeps_its_replies_while_nothing_waits_on_it` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:907-925` |
| `tests::an_eviction_watcher_sees_the_eviction` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:962-979` |
| `tests::eviction_releases_a_producer_waiting_on_the_evicted_tab` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:943-959` |
| `tests::overflow_replies` | Function | `apps/causeway-server/src/composition/state/session_streams.rs:937-940` |

Removed symbols: `tests::stall_and_evict` (Function, `apps/causeway-server/src/composition/state/session_streams.rs:958-962` at base), `Backlog::new` (Method, `apps/causeway-server/src/composition/state/session_streams.rs:182-189` at base), `Backlog::now_ms` (Method, `apps/causeway-server/src/composition/state/session_streams.rs:191-193` at base), `Backlog::stalled` (Method, `apps/causeway-server/src/composition/state/session_streams.rs:196-201` at base), `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` (Test, `apps/causeway-server/src/composition/state/session_streams.rs:932-945` at base).

**Affected (60 nodes within 3 hops; confidence is the product along the path, floor 0.25)**

_Code symbols (56)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `ConnectionSink` | Struct | `apps/causeway-server/src/composition/state/session_streams.rs:198-208` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:206` |
| `ConnectionSink::enqueue` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:264-274` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:272` |
| `SessionFeed` | Struct | `apps/causeway-server/src/composition/state/session_streams.rs:526-531` | 1 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:528` |
| `SessionFeed::took` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:553-560` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:558` |
| `tests::a_frame_larger_than_the_budget_passes_alone` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:663-682` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:665` |
| `tests::a_full_queue_makes_the_producer_wait` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:779-793` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:781` |
| `tests::a_reply_never_waits_on_a_full_queue_and_keeps_its_place` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:869-888` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:877` |
| `tests::a_slow_consumer_receives_every_frame_in_order` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:752-776` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:754` |
| `tests::a_slow_wide_connection_never_holds_more_than_its_budget` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:685-721` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:688` |
| `tests::a_tab_paused_for_less_than_the_grace_receives_every_reply` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:891-899` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:895` |
| `tests::a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:724-749` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:726` |
| `tests::a_tab_that_stops_draining_is_evicted_and_the_others_carry_on` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:796-832` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:798` |
| `tests::an_eviction_watcher_ends_when_the_connection_closes` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:982-993` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:984` |
| `tests::data_frames_are_bounded_in_bytes_not_only_in_frames` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:634-660` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:636` |
| `tests::dropping_the_last_feed_forgets_the_session_and_frees_waiting_producers` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:835-852` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:837` |
| `tests::replies_past_the_backlog_bound_evict_the_tab` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:928-934` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:931` |
| `tests::sessions_are_isolated` | Test | `apps/causeway-server/src/composition/state/session_streams.rs:855-866` | 1 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:857` |
| `AppState::session_subscribe` | Method | `apps/causeway-server/src/composition/state.rs:902-904` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state.rs:902` |
| `RealtimeRuntime::session_subscribe` | Method | `apps/causeway-server/src/composition/state/realtime_runtime.rs:299-301` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/realtime_runtime.rs:299` |
| `Registration` | Struct | `apps/causeway-server/src/composition/state/session_streams.rs:569-573` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:572` |
| `SessionFeed#impl` | Impl | `apps/causeway-server/src/composition/state/session_streams.rs:533-566` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:533-566` |
| `SessionFeed::recv` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:543-549` | 2 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:547` |
| `SessionStreams` | Struct | `apps/causeway-server/src/composition/state/session_streams.rs:287-296` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:290` |
| `SessionStreams::evict` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:422-430` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:422` |
| `SessionStreams::lock` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:468-475` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:468` |
| `SessionStreams::send` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:370-394` | 2 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:378` |
| `SessionStreams::sinks` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:417-419` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:417` |
| `ConnectionOutbox` | Struct | `apps/causeway-server/src/realtime.rs:92-95` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/realtime.rs:94` |
| `subscribe_session` | Function | `apps/causeway-server/src/realtime.rs:30-32` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/realtime.rs:30` |
| `Shell::ran_on` | Method | `apps/causeway-server/src/transport/session_rpc/binding_tests.rs:159-165` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/transport/session_rpc/binding_tests.rs:159` |
| `next_settlement` | Function | `apps/causeway-server/src/transport/session_rpc/binding_tests.rs:408-414` | 2 | 1.00 | USES_TYPE | `apps/causeway-server/src/transport/session_rpc/binding_tests.rs:408` |
| `RealtimeRuntime` | Struct | `apps/causeway-server/src/composition/state/realtime_runtime.rs:58-78` | 3 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/realtime_runtime.rs:60` |
| `Registration#impl(Drop)` | Impl | `apps/causeway-server/src/composition/state/session_streams.rs:575-580` | 3 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:575-580` |
| `SessionSender` | Struct | `apps/causeway-server/src/composition/state/session_streams.rs:480-483` | 3 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:481` |
| `SessionStreams#impl(Default)` | Impl | `apps/causeway-server/src/composition/state/session_streams.rs:298-302` | 3 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:298-302` |
| `SessionStreams::connections` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:462-466` | 3 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:463` |
| `SessionStreams::held_data_bytes` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:453-458` | 3 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:454` |
| `SessionStreams::live_sessions` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:446-448` | 3 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:447` |
| `SessionStreams::unregister` | Method | `apps/causeway-server/src/composition/state/session_streams.rs:432-441` | 3 | 1.00 | CALLS | `apps/causeway-server/src/composition/state/session_streams.rs:433` |
| `tests::budgeted` | Function | `apps/causeway-server/src/composition/state/session_streams.rs:602-606` | 3 | 1.00 | USES_TYPE | `apps/causeway-server/src/composition/state/session_streams.rs:602` |

… and 16 more in the JSON.

_Files and packages (4)_

| Node | Kind | Where | Depth | Confidence | Via | Evidence |
|---|---|---|---|---|---|---|
| `apps/causeway-server/src/composition/state.rs` | File | `apps/causeway-server/src/composition/state.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/composition/state.rs:42-45` |
| `apps/causeway-server/src/composition/state/realtime_runtime.rs` | File | `apps/causeway-server/src/composition/state/realtime_runtime.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/composition/state/realtime_runtime.rs:12` |
| `apps/causeway-server/src/realtime.rs` | File | `apps/causeway-server/src/realtime.rs` | 2 | 1.00 | IMPORTS | `apps/causeway-server/src/realtime.rs:24` |
| `apps/causeway-server/src/ws/tests/delivery.rs` | File | `apps/causeway-server/src/ws/tests/delivery.rs` | 3 | 1.00 | IMPORTS | `apps/causeway-server/src/ws/tests/delivery.rs:14` |

The dataset holds no nodes of these kinds, so they cannot appear: contracts and topics, workloads.

**Invariants (5 rules, 0 violations, 0 failed)**

| Rule | Verdict | Findings |
|---|---|---|
| `component-cross-deps` | ok | 0 |
| `server-rpc-contract-deps` | ok | 0 of 0 allowed |
| `server-rpc-service-impls` | ok | 0 of 0 allowed |
| `island-cross-references` | ok | 0 of 0 allowed |
| `runtime-env-resource-ctors` | ok | 0 |

<sub>surveyor 0.1.0 · repository `causeway` · 65376 nodes, 277630 edges · 1581 ms</sub>
