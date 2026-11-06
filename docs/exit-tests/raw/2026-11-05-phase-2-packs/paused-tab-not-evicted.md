# Context pack

Task: Over Connect, a browser tab that pauses reading its session stream is thrown out before it resumes, so the slow-consumer case "arrow frames are never dropped" fails on main. A reply must never evict a tab for pausing while nothing is waiting on it; only a tab over its backlog, or stalled while a reply waits, may go.

## Seeds

- `rust:apps/causeway-server/src/composition/state/session_streams.rs::REPLY_BACKLOG_BYTES` Const 0.90 (text)
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::reply` Method 0.86 (text)
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::stall_and_evict` Function 0.76 (text)
- `rust:apps/causeway-server/src/realtime.rs::reply_to_session` Function 0.71 (text)
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::ConnectionSink::offer_replies` Method 0.71 (text)
- `rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` Test 0.70 (text)
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` Test 0.65 (text)
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_tab_paused_for_less_than_the_grace_receives_every_reply` Test 0.62 (text)
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::replies_past_the_backlog_bound_evict_the_tab` Test 0.61 (text)
- `rust:apps/causeway-server/src/composition/state.rs::AppState::reply_to_session` Method 0.59 (text)

Budget 30000 tokens, used 29975. Seeds 10, candidates 382, included 83, left out 602.

## Context

### 1. Method SessionStreams::reply
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::reply`  apps/causeway-server/src/composition/state/session_streams.rs:430-444  pkg:cargo/causeway-server  score 5.09  
`pub(super) fn reply(&self, session_id: &str, frames: &[Vec<u8>])`  
Delivers the apply loop's reply `frames`, in order, to every (doc continues, 274 bytes)  
callers 9: `tests::stall_and_evict` (apps/causeway-server/src/composition/state/session_streams.rs:959), `tests::stall_and_evict` (apps/causeway-server/src/composition/state/session_streams.rs:961), `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` (apps/causeway-server/src/composition/state/session_streams.rs:940) … 6 more  
calls 4: `SessionId::from` (apps/causeway-server/src/composition/state/session_streams.rs:434), `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:436), `SessionStreams::sinks` (apps/causeway-server/src/composition/state/session_streams.rs:435) … 1 more  
references 1: `Delivery::TooSlow` (apps/causeway-server/src/composition/state/session_streams.rs:436)  
tests 4: `tests::replies_past_the_backlog_bound_evict_the_tab` (apps/causeway-server/src/composition/state/session_streams.rs:951), `tests::a_reply_never_waits_on_a_full_queue_and_keeps_its_place` (apps/causeway-server/src/composition/state/session_streams.rs:907), `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` (apps/causeway-server/src/composition/state/session_streams.rs:937) … 1 more  
apps/causeway-server/src/composition/state/session_streams.rs:430-444
```rust
    pub(super) fn reply(&self, session_id: &str, frames: &[Vec<u8>]) {
        if frames.is_empty() {
            return;
        }
        let session_id = SessionId::from(session_id);
        for sink in self.sinks(&session_id) {
            if let Delivery::TooSlow = sink.offer_replies(frames, self.grace) {
                self.evict(
                    &session_id,
                    &sink,
                    "a reply found it stalled or over its backlog",
                );
            }
        }
    }
```

### 2. Test tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all`  apps/causeway-server/src/composition/state/session_streams.rs:932-945  pkg:cargo/causeway-server  score 2.77  
`async fn a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all()`  
calls 13: `SessionStreams::connections` (apps/causeway-server/src/composition/state/session_streams.rs:944), `SessionStreams::with_grace` (apps/causeway-server/src/composition/state/session_streams.rs:933), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:941) … 10 more  
tests_for 6: `SessionStreams::with_grace` (apps/causeway-server/src/composition/state/session_streams.rs:933), `SessionStreams::connections` (apps/causeway-server/src/composition/state/session_streams.rs:944), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:938) … 3 more  
apps/causeway-server/src/composition/state/session_streams.rs:932-945
```rust
    async fn a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all() {
        let streams = Arc::new(SessionStreams::with_grace(Duration::from_millis(100)));
        let mut stalled = streams.subscribe("s");
        let mut live = streams.subscribe("s");

        streams.reply("s", &[frame(1)]);
        assert_eq!(live.recv().await, Ok(frame(1)));
        tokio::time::sleep(Duration::from_millis(150)).await;
        streams.reply("s", &[frame(2)]);
        assert_eq!(live.recv().await, Ok(frame(2)), "the live tab keeps up");

        assert_eq!(stalled.recv().await, Err(FellBehind));
        assert_eq!(streams.connections("s"), 1, "only the live tab is served");
    }
```

### 3. Function tests::stall_and_evict
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::stall_and_evict`  apps/causeway-server/src/composition/state/session_streams.rs:958-962  pkg:cargo/causeway-server  score 2.61  
`async fn stall_and_evict(streams: &SessionStreams)`  
Stalls the only tab of `s` for longer than `streams`' grace, then (doc continues, 90 bytes)  
callers 2: `tests::an_eviction_watcher_sees_the_eviction` (apps/causeway-server/src/composition/state/session_streams.rs:994), `tests::eviction_releases_a_producer_waiting_on_the_evicted_tab` (apps/causeway-server/src/composition/state/session_streams.rs:976)  
calls 4: `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:959), `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:961), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:961) … 1 more  
tests 2: `tests::eviction_releases_a_producer_waiting_on_the_evicted_tab` (apps/causeway-server/src/composition/state/session_streams.rs:976), `tests::an_eviction_watcher_sees_the_eviction` (apps/causeway-server/src/composition/state/session_streams.rs:994)  
uses_types 1: `SessionStreams` (apps/causeway-server/src/composition/state/session_streams.rs:958)  
apps/causeway-server/src/composition/state/session_streams.rs:958-962
```rust
    async fn stall_and_evict(streams: &SessionStreams) {
        streams.reply("s", &[frame(0)]);
        tokio::time::sleep(Duration::from_millis(150)).await;
        streams.reply("s", &[frame(1)]);
    }
```

### 4. Method ConnectionSink::offer_replies
`rust:apps/causeway-server/src/composition/state/session_streams.rs::ConnectionSink::offer_replies`  apps/causeway-server/src/composition/state/session_streams.rs:275-291  pkg:cargo/causeway-server  score 2.44  
`fn offer_replies(&self, frames: &[Vec<u8>], grace: Duration) -> Delivery`  
Queues reply `frames` in order without waiting, unless the (doc continues, 126 bytes)  
callers 1: `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:436)  
calls 3: `ConnectionSink::enqueue` (apps/causeway-server/src/composition/state/session_streams.rs:283), `Backlog::stalled` (apps/causeway-server/src/composition/state/session_streams.rs:279), `Backlog::would_exceed_replies` (apps/causeway-server/src/composition/state/session_streams.rs:279)  
references 4: `Lane::Reply` (apps/causeway-server/src/composition/state/session_streams.rs:283), `Delivery::TooSlow` (apps/causeway-server/src/composition/state/session_streams.rs:281), `Delivery::Queued` (apps/causeway-server/src/composition/state/session_streams.rs:288) … 1 more  
uses_types 1: `Delivery` (apps/causeway-server/src/composition/state/session_streams.rs:275)  
apps/causeway-server/src/composition/state/session_streams.rs:275-291
```rust
    fn offer_replies(&self, frames: &[Vec<u8>], grace: Duration) -> Delivery {
        let offered = frames.iter().try_for_each(|frame| {
            if self.data_slots.is_closed() {
                ControlFlow::Break(Delivery::Gone)
            } else if self.backlog.stalled(grace) || self.backlog.would_exceed_replies(frame.len())
            {
                ControlFlow::Break(Delivery::TooSlow)
            } else {
                self.enqueue(frame.clone(), Lane::Reply);
                ControlFlow::Continue(())
            }
        });
        match offered {
            ControlFlow::Continue(()) => Delivery::Queued,
            ControlFlow::Break(delivery) => delivery,
        }
    }
```

### 5. Function tests::frame
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::frame`  apps/causeway-server/src/composition/state/session_streams.rs:625-627  pkg:cargo/causeway-server  score 2.43  via `tests::stall_and_evict` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:961, 1.00)  
`fn frame(n: usize) -> Vec<u8>`  
callers 37: `tests::a_full_queue_makes_the_producer_wait` (apps/causeway-server/src/composition/state/session_streams.rs:813), `tests::dropping_the_last_feed_forgets_the_session_and_frees_waiting_producers` (apps/causeway-server/src/composition/state/session_streams.rs:873), `tests::replies_past_the_backlog_bound_evict_the_tab` (apps/causeway-server/src/composition/state/session_streams.rs:952) … 34 more  
tests 14: `tests::a_frame_larger_than_the_budget_passes_alone` (apps/causeway-server/src/composition/state/session_streams.rs:703), `tests::a_slow_consumer_receives_every_frame_in_order` (apps/causeway-server/src/composition/state/session_streams.rs:791), `tests::eviction_releases_a_producer_waiting_on_the_evicted_tab` (apps/causeway-server/src/composition/state/session_streams.rs:969) … 11 more  
apps/causeway-server/src/composition/state/session_streams.rs:625-627
```rust
    fn frame(n: usize) -> Vec<u8> {
        n.to_be_bytes().to_vec()
    }
```

### 6. Test tests::replies_past_the_backlog_bound_evict_the_tab
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::replies_past_the_backlog_bound_evict_the_tab`  apps/causeway-server/src/composition/state/session_streams.rs:948-954  pkg:cargo/causeway-server  score 2.36  
`async fn replies_past_the_backlog_bound_evict_the_tab()`  
calls 6: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:952), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:953), `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:951) … 3 more  
tests_for 5: `SessionStreams::default` (apps/causeway-server/src/composition/state/session_streams.rs:949), `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:951), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:953) … 2 more  
apps/causeway-server/src/composition/state/session_streams.rs:948-954
```rust
    async fn replies_past_the_backlog_bound_evict_the_tab() {
        let streams = Arc::new(SessionStreams::default());
        let mut paused = streams.subscribe("s");
        streams.reply("s", &[vec![0; REPLY_BACKLOG_BYTES]]);
        streams.reply("s", &[frame(1)]);
        assert_eq!(paused.recv().await, Err(FellBehind));
    }
```

### 7. Test a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:368-404  pkg:cargo/causeway-server  score 2.18  
`async fn a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs()`  
A tab that stops reading does not hold up the session's command replies: (doc continues, 172 bytes)  
calls 8: `send_to_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:381), `Shell::ran_on` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:400), `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:370) … 5 more  
references 2: `TAG_PROTO` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:380), `CONNECTION_QUEUE_CAPACITY` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:375)  
tests_for 7: `Shell::ran_on` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:400), `Shell::start` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:369), `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:396) … 4 more  
apps/causeway-server/src/transport/session_rpc/binding_tests.rs:368-404
```rust
async fn a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs() {
    let shell = Shell::start().await;
    let (connection, _stream) = shell.subscribe(&shell.alice, "alice-session").await;
    // A tab that is never read: it stalls once its queue is full.
    let _stalled = crate::realtime::subscribe_session(&shell.state, "alice-session");
    let mut live = crate::realtime::subscribe_session(&shell.state, "alice-session");

    let capacity = crate::composition::state::CONNECTION_QUEUE_CAPACITY;
    let producer = {
        let state = shell.state.clone();
        tokio::spawn(async move {
            for n in 0..capacity {
                let frame = [&[protocol_wire::TAG_PROTO][..], &n.to_be_bytes()].concat();
                crate::realtime::send_to_session(&state, "alice-session", frame).await;
            }
        })
    };
    for _ in 0..capacity {
        live.recv().await.expect("the live tab keeps up");
    }
    producer
        .await
        .expect("the producer fills the stalled queue");

    // The first reply reaches the live tab either way; the apply loop being
    // held up on the stalled tab shows in the reply to the next command.
    for command in ["first", "second"] {
        shell
            .send(bearer(Some(&shell.alice)), &connection.to_string())
            .await
            .expect("the Send is admitted");
        assert!(
            Shell::ran_on(&mut live).await,
            "the {command} reply reaches the live tab without waiting on the stalled one"
        );
    }
}
```

### 8. Test tests::a_tab_paused_for_less_than_the_grace_receives_every_reply
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_tab_paused_for_less_than_the_grace_receives_every_reply`  apps/causeway-server/src/composition/state/session_streams.rs:921-929  pkg:cargo/causeway-server  score 2.07  
`async fn a_tab_paused_for_less_than_the_grace_receives_every_reply()`  
calls 6: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:927), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:923), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:927) … 3 more  
tests_for 5: `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:923), `SessionStreams::with_grace` (apps/causeway-server/src/composition/state/session_streams.rs:922), `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:925) … 2 more  
apps/causeway-server/src/composition/state/session_streams.rs:921-929
```rust
    async fn a_tab_paused_for_less_than_the_grace_receives_every_reply() {
        let streams = Arc::new(SessionStreams::with_grace(Duration::from_secs(5)));
        let mut paused = streams.subscribe("s");
        let total = CONNECTION_QUEUE_CAPACITY * 4;
        (0..total).for_each(|n| streams.reply("s", &[frame(n)]));
        for n in 0..total {
            assert_eq!(paused.recv().await, Ok(frame(n)), "reply {n}");
        }
    }
```

### 9. Const REPLY_BACKLOG_BYTES
`rust:apps/causeway-server/src/composition/state/session_streams.rs::REPLY_BACKLOG_BYTES`  apps/causeway-server/src/composition/state/session_streams.rs:126  pkg:cargo/causeway-server  score 2.05  
`pub(crate) const REPLY_BACKLOG_BYTES: usize = 64 * 1024 * 1024;`  
Bytes of replies one connection may have queued; a reply past it evicts (doc continues, 132 bytes)  
importers 1: `tests` (apps/causeway-server/src/composition/state/session_streams.rs:620-623)  
apps/causeway-server/src/composition/state/session_streams.rs:126-126
```rust
pub(crate) const REPLY_BACKLOG_BYTES: usize = 64 * 1024 * 1024;
```

### 10. Method SessionFeed::recv
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionFeed::recv`  apps/causeway-server/src/composition/state/session_streams.rs:573-579  pkg:cargo/causeway-server  score 1.84  via `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` <- TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:938, 1.00)  
`pub(crate) async fn recv(&mut self) -> Result<Vec<u8>, FellBehind>`  
The next frame for this connection. (doc continues, 264 bytes)  
callers 21: `tests::sessions_are_isolated` (apps/causeway-server/src/composition/state/session_streams.rs:890), `tests::a_tab_that_stops_draining_is_evicted_and_the_others_carry_on` (apps/causeway-server/src/composition/state/session_streams.rs:842), `tests::a_slow_wide_connection_never_holds_more_than_its_budget` (apps/causeway-server/src/composition/state/session_streams.rs:739) … 18 more  
calls 3: `SessionFeed::took` (apps/causeway-server/src/composition/state/session_streams.rs:577), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:577), `Eviction::wait` (apps/causeway-server/src/composition/state/session_streams.rs:576)  
tests 12: `tests::a_full_queue_makes_the_producer_wait` (apps/causeway-server/src/composition/state/session_streams.rs:819), `tests::replies_past_the_backlog_bound_evict_the_tab` (apps/causeway-server/src/composition/state/session_streams.rs:953), `tests::data_frames_are_bounded_in_bytes_not_only_in_frames` (apps/causeway-server/src/composition/state/session_streams.rs:682) … 9 more  
uses_types 1: `FellBehind` (apps/causeway-server/src/composition/state/session_streams.rs:573)  
apps/causeway-server/src/composition/state/session_streams.rs:573-579
```rust
    pub(crate) async fn recv(&mut self) -> Result<Vec<u8>, FellBehind> {
        tokio::select! {
            biased;
            _ = self.eviction.wait() => Err(FellBehind),
            queued = self.frames.recv() => queued.map(|queued| self.took(queued)).ok_or(FellBehind),
        }
    }
```

### 11. Method SessionStreams::subscribe
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::subscribe`  apps/causeway-server/src/composition/state/session_streams.rs:354-382  pkg:cargo/causeway-server  score 1.55  via `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` <- TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:934, 1.00)  
`pub(super) fn subscribe(self: &Arc<Self>, session_id: &str) -> SessionFeed`  
Registers a new connection of `session_id` and returns its feed. The (doc continues, 162 bytes)  
callers 23: `tests::sessions_are_isolated` (apps/causeway-server/src/composition/state/session_streams.rs:888), `tests::a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace` (apps/causeway-server/src/composition/state/session_streams.rs:756), `tests::eviction_releases_a_producer_waiting_on_the_evicted_tab` (apps/causeway-server/src/composition/state/session_streams.rs:967) … 20 more  
calls 6: `DataBudget::permits` (apps/causeway-server/src/composition/state/session_streams.rs:362), `Backlog::new` (apps/causeway-server/src/composition/state/session_streams.rs:357), `SessionId::from` (apps/causeway-server/src/composition/state/session_streams.rs:367) … 3 more  
references 1: `Standing::Served` (apps/causeway-server/src/composition/state/session_streams.rs:356)  
tests 16: `tests::an_eviction_watcher_sees_the_eviction` (apps/causeway-server/src/composition/state/session_streams.rs:986), `tests::a_tab_paused_for_less_than_the_grace_receives_every_reply` (apps/causeway-server/src/composition/state/session_streams.rs:923), `tests::dropping_the_last_feed_forgets_the_session_and_frees_waiting_producers` (apps/causeway-server/src/composition/state/session_streams.rs:867) … 13 more  
uses_types 3: `ConnectionSink` (apps/causeway-server/src/composition/state/session_streams.rs:358), `SessionFeed` (apps/causeway-server/src/composition/state/session_streams.rs:354), `Registration` (apps/causeway-server/src/composition/state/session_streams.rs:376)  
apps/causeway-server/src/composition/state/session_streams.rs:354-382
```rust
    pub(super) fn subscribe(self: &Arc<Self>, session_id: &str) -> SessionFeed {
        let (sender, receiver) = mpsc::unbounded_channel();
        let (standing, standing_rx) = watch::channel(Standing::Served);
        let backlog = Arc::new(Backlog::new());
        let sink = ConnectionSink {
            id: SinkId(self.next_sink.fetch_add(1, Ordering::Relaxed)),
            frames: sender,
            data_slots: Arc::new(Semaphore::new(CONNECTION_QUEUE_CAPACITY)),
            data_bytes: Arc::new(Semaphore::new(self.budget.permits())),
            budget: self.budget,
            backlog: Arc::clone(&backlog),
            standing: Arc::new(standing),
        };
        let session_id = SessionId::from(session_id);
        self.lock()
            .entry(session_id.clone())
            .or_default()
            .push(sink.clone());
        SessionFeed {
            frames: receiver,
            backlog,
            eviction: Eviction(standing_rx),
            _registration: Registration {
                streams: Arc::clone(self),
                session_id,
                sink,
            },
        }
    }
```

### 12. Test tests::a_reply_never_waits_on_a_full_queue_and_keeps_its_place
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_reply_never_waits_on_a_full_queue_and_keeps_its_place`  apps/causeway-server/src/composition/state/session_streams.rs:899-918  pkg:cargo/causeway-server  score 1.45  via `SessionStreams::reply` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:907, 1.00)  
`async fn a_reply_never_waits_on_a_full_queue_and_keeps_its_place()`  
calls 14: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:907), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:917), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:910) … 11 more  
tests_for 7: `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:907), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:901), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:903) … 4 more  
apps/causeway-server/src/composition/state/session_streams.rs:899-918
```rust
    async fn a_reply_never_waits_on_a_full_queue_and_keeps_its_place() {
        let streams = Arc::new(SessionStreams::default());
        let mut feed = streams.subscribe("s");
        for n in 0..CONNECTION_QUEUE_CAPACITY {
            streams.send("s", frame(n)).await;
        }

        // Synchronous: the apply loop cannot be held up here.
        streams.reply("s", &[frame(9000), frame(9001)]);

        for n in 0..CONNECTION_QUEUE_CAPACITY {
            assert_eq!(feed.recv().await, Ok(frame(n)));
        }
        assert_eq!(
            feed.recv().await,
            Ok(frame(9000)),
            "the reply keeps its place"
        );
        assert_eq!(feed.recv().await, Ok(frame(9001)));
    }
```

### 13. Function reply_to_session
`rust:apps/causeway-server/src/realtime.rs::reply_to_session`  apps/causeway-server/src/realtime.rs:42-44  pkg:cargo/causeway-server  score 1.44  
`pub(crate) fn reply_to_session(state: &AppState, session_id: &str, frames: &[Vec<u8>])`  
Delivers the session apply loop's reply `frames` to every connection of (doc continues, 286 bytes)  
callers 1: `run_session_actor` (apps/causeway-server/src/ws/session.rs:211)  
calls 2: `AppState::reply_to_session` (apps/causeway-server/src/realtime.rs:43), `RealtimeRuntime::reply_to_session` (apps/causeway-server/src/realtime.rs:43)  
uses_types 1: `AppState` (apps/causeway-server/src/realtime.rs:42)  
apps/causeway-server/src/realtime.rs:42-44
```rust
pub(crate) fn reply_to_session(state: &AppState, session_id: &str, frames: &[Vec<u8>]) {
    state.reply_to_session(session_id, frames);
}
```

### 14. Method AppState::reply_to_session
`rust:apps/causeway-server/src/composition/state.rs::AppState::reply_to_session`  apps/causeway-server/src/composition/state.rs:909-911  pkg:cargo/causeway-server  score 1.29  
`pub(crate) fn reply_to_session(&self, session_id: &str, frames: &[Vec<u8>])`  
Delivers the session apply loop's reply `frames` to every connection (doc continues, 183 bytes)  
callers 2: `AppState::reply_to_session` (apps/causeway-server/src/composition/state.rs:910), `reply_to_session` (apps/causeway-server/src/realtime.rs:43)  
calls 1: `AppState::reply_to_session` (apps/causeway-server/src/composition/state.rs:910)  
apps/causeway-server/src/composition/state.rs:909-911
```rust
    pub(crate) fn reply_to_session(&self, session_id: &str, frames: &[Vec<u8>]) {
        self.realtime.reply_to_session(session_id, frames);
    }
```

### 15. Module tests
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests`  apps/causeway-server/src/composition/state/session_streams.rs:613-1016  pkg:cargo/causeway-server  score 1.28  via `tests::stall_and_evict` <- DEFINES (apps/causeway-server/src/composition/state/session_streams.rs:958-962, 1.00)  
`mod tests`  
members 23  
imports 7: `DataBudget` (apps/causeway-server/src/composition/state/session_streams.rs:620-623), `ServiceEnded` (apps/causeway-server/src/composition/state/session_streams.rs:620-623), `cargo/tokio` (apps/causeway-server/src/composition/state/session_streams.rs:618) … 4 more  

### 16. Struct SessionStreams
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams`  apps/causeway-server/src/composition/state/session_streams.rs:316-325  pkg:cargo/causeway-server  score 1.15  via `SessionStreams#impl` -> USES_TYPE (apps/causeway-server/src/composition/state/session_streams.rs:333-506, 1.00)  
`pub(crate) struct SessionStreams`  
Per-session connection queues keyed by session id.  
members 14  
importers 3: `apps/causeway-server/src/composition/state/realtime_runtime.rs` (apps/causeway-server/src/composition/state/realtime_runtime.rs:11), `apps/causeway-server/src/composition/state.rs` (apps/causeway-server/src/composition/state.rs:41), `tests` (apps/causeway-server/src/composition/state/session_streams.rs:620-623)  
used_by 7: `SessionStreams#impl(Default)` (apps/causeway-server/src/composition/state/session_streams.rs:327-331), `SessionStreams#impl` (apps/causeway-server/src/composition/state/session_streams.rs:333-506), `RealtimeRuntime` (apps/causeway-server/src/composition/state/realtime_runtime.rs:60) … 4 more  
uses_types 3: `DataBudget` (apps/causeway-server/src/composition/state/session_streams.rs:322), `ConnectionSink` (apps/causeway-server/src/composition/state/session_streams.rs:319), `SessionId` (apps/causeway-server/src/composition/state/session_streams.rs:319)  
apps/causeway-server/src/composition/state/session_streams.rs:316-325
```rust
pub(crate) struct SessionStreams {
    // A std mutex, never held across an await: the feed's `Drop`
    // unregisters through it.
    sessions: Mutex<HashMap<SessionId, Vec<ConnectionSink>>>,
    next_sink: AtomicU64,
    /// The data bytes each connection may hold queued.
    budget: DataBudget,
    /// How long a data producer waits on one full queue before evicting it.
    grace: Duration,
}
```

### 17. Variant Delivery::TooSlow
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Delivery::TooSlow`  apps/causeway-server/src/composition/state/session_streams.rs:312  pkg:cargo/causeway-server  score 1.07  via `SessionStreams::reply` -> REFERENCES (apps/causeway-server/src/composition/state/session_streams.rs:436, 1.00)  
`TooSlow`  
The connection could not take the frame in time: evict it.  
referenced_by 4: `SessionStreams::send` (apps/causeway-server/src/composition/state/session_streams.rs:411), `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:436), `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:281) … 1 more  

### 18. File apps/causeway-server/src/composition/state/session_streams.rs
`file:apps/causeway-server/src/composition/state/session_streams.rs`  apps/causeway-server/src/composition/state/session_streams.rs  pkg:cargo/causeway-server  score 1.05  via `REPLY_BACKLOG_BYTES` <- CONTAINS (apps/causeway-server/src/composition/state/session_streams.rs:126, 1.00)  
members 30  
imports 2: `SessionId` (apps/causeway-server/src/composition/state/session_streams.rs:62), `cargo/tokio` (apps/causeway-server/src/composition/state/session_streams.rs:60)  

### 19. Method SessionStreams::send
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::send`  apps/causeway-server/src/composition/state/session_streams.rs:399-423  pkg:cargo/causeway-server  score 1.01  via `Delivery::TooSlow` <- REFERENCES (apps/causeway-server/src/composition/state/session_streams.rs:419, 1.00)  
`pub(super) async fn send(&self, session_id: &str, frame: Vec<u8>)`  
Delivers a data `frame` to every connection of `session_id`, waiting (doc continues, 323 bytes)  
callers 21: `tests::a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace` (apps/causeway-server/src/composition/state/session_streams.rs:763), `tests::a_frame_larger_than_the_budget_passes_alone` (apps/causeway-server/src/composition/state/session_streams.rs:709), `tests::a_frame_larger_than_the_budget_passes_alone` (apps/causeway-server/src/composition/state/session_streams.rs:698) … 18 more  
calls 5: `SessionStreams::evict` (apps/causeway-server/src/composition/state/session_streams.rs:421), `ConnectionSink::enqueue` (apps/causeway-server/src/composition/state/session_streams.rs:407), `SessionStreams::sinks` (apps/causeway-server/src/composition/state/session_streams.rs:402) … 2 more  
references 4: `Delivery::TooSlow` (apps/causeway-server/src/composition/state/session_streams.rs:411), `Delivery::Queued` (apps/causeway-server/src/composition/state/session_streams.rs:408), `Delivery::TooSlow` (apps/causeway-server/src/composition/state/session_streams.rs:419) … 1 more  
tests 11: `tests::a_frame_larger_than_the_budget_passes_alone` (apps/causeway-server/src/composition/state/session_streams.rs:698), `tests::dropping_the_last_feed_forgets_the_session_and_frees_waiting_producers` (apps/causeway-server/src/composition/state/session_streams.rs:869), `tests::eviction_releases_a_producer_waiting_on_the_evicted_tab` (apps/causeway-server/src/composition/state/session_streams.rs:969) … 8 more  
uses_types 2: `Data` (apps/causeway-server/src/composition/state/session_streams.rs:407), `AuditedBody::Data` (apps/causeway-server/src/composition/state/session_streams.rs:407)  
apps/causeway-server/src/composition/state/session_streams.rs:399-423
```rust
    pub(super) async fn send(&self, session_id: &str, frame: Vec<u8>) {
        let session_id = SessionId::from(session_id);
        let grace = self.grace;
        let deliveries = self.sinks(&session_id).into_iter().map(|sink| {
            let frame = frame.clone();
            async move {
                let delivery = match tokio::time::timeout(grace, sink.room_for(frame.len())).await {
                    Ok(Ok(room)) => {
                        sink.enqueue(frame, Lane::Data { _room: room });
                        Delivery::Queued
                    }
                    Ok(Err(_closed)) => Delivery::Gone,
                    Err(_elapsed) => Delivery::TooSlow,
                };
                (sink, delivery)
            }
        });
        futures::future::join_all(deliveries)
            .await
            .into_iter()
            .filter(|(_, delivery)| matches!(delivery, Delivery::TooSlow))
            .for_each(|(slow, _)| {
                self.evict(&session_id, &slow, "a data producer waited out the grace")
            });
    }
```

### 20. Test tests::eviction_releases_a_producer_waiting_on_the_evicted_tab
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::eviction_releases_a_producer_waiting_on_the_evicted_tab`  apps/causeway-server/src/composition/state/session_streams.rs:965-981  pkg:cargo/causeway-server  score 0.99  via `tests::stall_and_evict` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:976, 1.00)  
`async fn eviction_releases_a_producer_waiting_on_the_evicted_tab()`  
calls 9: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:973), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:967), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:969) … 6 more  
tests_for 6: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:969), `tests::stall_and_evict` (apps/causeway-server/src/composition/state/session_streams.rs:976), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:967) … 3 more  
apps/causeway-server/src/composition/state/session_streams.rs:965-981
```rust
    async fn eviction_releases_a_producer_waiting_on_the_evicted_tab() {
        let streams = Arc::new(SessionStreams::with_grace(Duration::from_millis(100)));
        let _stalled = streams.subscribe("s");
        for n in 0..CONNECTION_QUEUE_CAPACITY {
            streams.send("s", frame(n)).await;
        }
        let waiting = {
            let streams = Arc::clone(&streams);
            tokio::spawn(async move { streams.send("s", frame(0)).await })
        };
        tokio::task::yield_now().await;
        stall_and_evict(&streams).await;
        timeout(Duration::from_millis(200), waiting)
            .await
            .expect("the waiting producer is released")
            .expect("it does not panic");
    }
```

### 21. Method SessionStreams::with_grace
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::with_grace`  apps/causeway-server/src/composition/state/session_streams.rs:347-349  pkg:cargo/causeway-server  score 0.96  via `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` <- TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:933, 1.00)  
`pub(crate) fn with_grace(grace: Duration) -> Self`  
Queues with the default budget whose data producers wait `grace` on (doc continues, 99 bytes)  
callers 6: `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` (apps/causeway-server/src/composition/state/session_streams.rs:933), `tests::a_tab_that_stops_draining_is_evicted_and_the_others_carry_on` (apps/causeway-server/src/composition/state/session_streams.rs:827), `RealtimeRuntime::with_session_grace` (apps/causeway-server/src/composition/state/realtime_runtime.rs:108) … 3 more  
calls 2: `SessionStreams::new` (apps/causeway-server/src/composition/state/session_streams.rs:348), `Backlog::new` (apps/causeway-server/src/composition/state/session_streams.rs:348)  
references 1: `DataBudget::DEFAULT` (apps/causeway-server/src/composition/state/session_streams.rs:348)  
tests 5: `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` (apps/causeway-server/src/composition/state/session_streams.rs:933), `tests::an_eviction_watcher_sees_the_eviction` (apps/causeway-server/src/composition/state/session_streams.rs:985), `tests::a_tab_paused_for_less_than_the_grace_receives_every_reply` (apps/causeway-server/src/composition/state/session_streams.rs:922) … 2 more  
apps/causeway-server/src/composition/state/session_streams.rs:347-349
```rust
    pub(crate) fn with_grace(grace: Duration) -> Self {
        Self::new(DataBudget::DEFAULT, grace)
    }
```

### 22. Impl SessionStreams#impl
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams#impl`  apps/causeway-server/src/composition/state/session_streams.rs:333-506  pkg:cargo/causeway-server  score 0.92  via `SessionStreams::reply` <- DEFINES (apps/causeway-server/src/composition/state/session_streams.rs:430-444, 1.00)  
`impl SessionStreams`  
members 13  
uses_types 1: `SessionStreams` (apps/causeway-server/src/composition/state/session_streams.rs:333-506)  

### 23. Struct ConnectionSink
`rust:apps/causeway-server/src/composition/state/session_streams.rs::ConnectionSink`  apps/causeway-server/src/composition/state/session_streams.rs:226-236  pkg:cargo/causeway-server  score 0.90  via `ConnectionSink#impl` -> USES_TYPE (apps/causeway-server/src/composition/state/session_streams.rs:238-304, 1.00)  
`struct ConnectionSink`  
The producer side of one connection's queue.  
members 6  
used_by 7: `ConnectionSink#impl` (apps/causeway-server/src/composition/state/session_streams.rs:238-304), `SessionStreams` (apps/causeway-server/src/composition/state/session_streams.rs:319), `SessionStreams::evict` (apps/causeway-server/src/composition/state/session_streams.rs:452) … 4 more  
uses_types 5: `Queued` (apps/causeway-server/src/composition/state/session_streams.rs:228), `DataBudget` (apps/causeway-server/src/composition/state/session_streams.rs:233), `SinkId` (apps/causeway-server/src/composition/state/session_streams.rs:227) … 2 more  
apps/causeway-server/src/composition/state/session_streams.rs:226-236
```rust
struct ConnectionSink {
    id: SinkId,
    frames: mpsc::UnboundedSender<Queued>,
    /// Frame slots: [`CONNECTION_QUEUE_CAPACITY`] of them.
    data_slots: Arc<Semaphore>,
    /// Bytes of the connection's [`DataBudget`].
    data_bytes: Arc<Semaphore>,
    budget: DataBudget,
    backlog: Arc<Backlog>,
    standing: Arc<watch::Sender<Standing>>,
}
```

### 24. Method RealtimeRuntime::reply_to_session
`rust:apps/causeway-server/src/composition/state/realtime_runtime.rs::RealtimeRuntime::reply_to_session`  apps/causeway-server/src/composition/state/realtime_runtime.rs:305-307  pkg:cargo/causeway-server  score 0.78  via `SessionStreams::reply` <- CALLS (apps/causeway-server/src/composition/state/realtime_runtime.rs:306, 0.75)  
`pub(super) fn reply_to_session(&self, session_id: &str, frames: &[Vec<u8>])`  
Delivers the apply loop's reply `frames` to every connection of (doc continues, 126 bytes)  
callers 1: `reply_to_session` (apps/causeway-server/src/realtime.rs:43)  
calls 1: `SessionStreams::reply` (apps/causeway-server/src/composition/state/realtime_runtime.rs:306)  
apps/causeway-server/src/composition/state/realtime_runtime.rs:305-307
```rust
    pub(super) fn reply_to_session(&self, session_id: &str, frames: &[Vec<u8>]) {
        self.session_streams.reply(session_id, frames);
    }
```

### 25. Method SessionStreams::evict
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::evict`  apps/causeway-server/src/composition/state/session_streams.rs:452-460  pkg:cargo/causeway-server  score 0.74  via `SessionStreams::reply` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:437, 1.00)  
`fn evict(&self, session_id: &SessionId, sink: &ConnectionSink, why: &str)`  
Removes a connection that could not keep up and tells its feed.  
callers 2: `SessionStreams::send` (apps/causeway-server/src/composition/state/session_streams.rs:421), `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:437)  
calls 2: `SessionStreams::unregister` (apps/causeway-server/src/composition/state/session_streams.rs:458), `ConnectionSink::evict` (apps/causeway-server/src/composition/state/session_streams.rs:459)  
uses_types 2: `SessionId` (apps/causeway-server/src/composition/state/session_streams.rs:452), `ConnectionSink` (apps/causeway-server/src/composition/state/session_streams.rs:452)  
apps/causeway-server/src/composition/state/session_streams.rs:452-460
```rust
    fn evict(&self, session_id: &SessionId, sink: &ConnectionSink, why: &str) {
        tracing::warn!(
            reason = why,
            grace_ms = u64::try_from(self.grace.as_millis()).unwrap_or(u64::MAX),
            "session connection fell behind; closing it so it resyncs",
        );
        self.unregister(session_id, sink.id);
        sink.evict();
    }
```

### 26. Test tests::an_eviction_watcher_sees_the_eviction
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::an_eviction_watcher_sees_the_eviction`  apps/causeway-server/src/composition/state/session_streams.rs:984-1001  pkg:cargo/causeway-server  score 0.70  via `tests::stall_and_evict` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:994, 1.00)  
`async fn an_eviction_watcher_sees_the_eviction()`  
calls 6: `tests::stall_and_evict` (apps/causeway-server/src/composition/state/session_streams.rs:994), `SessionFeed::eviction` (apps/causeway-server/src/composition/state/session_streams.rs:987), `Eviction::wait` (apps/causeway-server/src/composition/state/session_streams.rs:989) … 3 more  
references 1: `ServiceEnded::Evicted` (apps/causeway-server/src/composition/state/session_streams.rs:999)  
tests_for 5: `SessionStreams::with_grace` (apps/causeway-server/src/composition/state/session_streams.rs:985), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:986), `Eviction::wait` (apps/causeway-server/src/composition/state/session_streams.rs:989) … 2 more  
apps/causeway-server/src/composition/state/session_streams.rs:984-1001
```rust
    async fn an_eviction_watcher_sees_the_eviction() {
        let streams = Arc::new(SessionStreams::with_grace(Duration::from_millis(100)));
        let feed = streams.subscribe("s");
        let mut eviction = feed.eviction();
        assert!(
            timeout(Duration::from_millis(20), eviction.wait())
                .await
                .is_err(),
            "a served tab is not evicted"
        );
        stall_and_evict(&streams).await;
        assert_eq!(
            timeout(Duration::from_millis(200), eviction.wait())
                .await
                .expect("the watcher wakes"),
            ServiceEnded::Evicted
        );
    }
```

### 27. Enum Delivery
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Delivery`  apps/causeway-server/src/composition/state/session_streams.rs:307-313  pkg:cargo/causeway-server  score 0.70  via `ConnectionSink::offer_replies` -> USES_TYPE (apps/causeway-server/src/composition/state/session_streams.rs:275, 1.00)  
`enum Delivery`  
What a delivery to one connection came to.  
members 3  
used_by 1: `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:275)  
uses_types 2: `Delivery` (apps/causeway-server/src/composition/state/session_streams.rs:307), `Delivery` (apps/causeway-server/src/composition/state/session_streams.rs:307)  
apps/causeway-server/src/composition/state/session_streams.rs:307-313
```rust
enum Delivery {
    Queued,
    /// The connection is gone, or was evicted by another producer.
    Gone,
    /// The connection could not take the frame in time: evict it.
    TooSlow,
}
```

### 28. Variant Lane::Reply
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Lane::Reply`  apps/causeway-server/src/composition/state/session_streams.rs:167  pkg:cargo/causeway-server  score 0.68  via `ConnectionSink::offer_replies` -> REFERENCES (apps/causeway-server/src/composition/state/session_streams.rs:283, 1.00)  
`Reply`  
An apply-loop reply, counted against the reply backlog.  
referenced_by 3: `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:283), `ConnectionSink::enqueue` (apps/causeway-server/src/composition/state/session_streams.rs:295), `SessionFeed::took` (apps/causeway-server/src/composition/state/session_streams.rs:585)  

### 29. Method SessionStreams::sinks
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::sinks`  apps/causeway-server/src/composition/state/session_streams.rs:447-449  pkg:cargo/causeway-server  score 0.67  via `SessionStreams::reply` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:435, 1.00)  
`fn sinks(&self, session_id: &SessionId) -> Vec<ConnectionSink>`  
The connections of `session_id` right now.  
callers 3: `SessionStreams::held_data_bytes` (apps/causeway-server/src/composition/state/session_streams.rs:484), `SessionStreams::send` (apps/causeway-server/src/composition/state/session_streams.rs:402), `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:435)  
calls 1: `SessionStreams::lock` (apps/causeway-server/src/composition/state/session_streams.rs:448)  
uses_types 2: `SessionId` (apps/causeway-server/src/composition/state/session_streams.rs:447), `ConnectionSink` (apps/causeway-server/src/composition/state/session_streams.rs:447)  
apps/causeway-server/src/composition/state/session_streams.rs:447-449
```rust
    fn sinks(&self, session_id: &SessionId) -> Vec<ConnectionSink> {
        self.lock().get(session_id).cloned().unwrap_or_default()
    }
```

### 30. File apps/causeway-server/src/realtime.rs
`file:apps/causeway-server/src/realtime.rs`  apps/causeway-server/src/realtime.rs  pkg:cargo/causeway-server  score 0.65  via `reply_to_session` <- CONTAINS (apps/causeway-server/src/realtime.rs:42-44, 1.00)  
members 10  
imports 6: `SessionFeed` (apps/causeway-server/src/realtime.rs:24), `AppState` (apps/causeway-server/src/realtime.rs:24), `FellBehind` (apps/causeway-server/src/realtime.rs:24) … 3 more  

### 31. Method SessionId::from
`rust:apps/causeway-server/src/composition/state/session_id.rs::SessionId::from`  apps/causeway-server/src/composition/state/session_id.rs:19-21  pkg:cargo/causeway-server  score 0.65  via `SessionStreams::reply` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:434, 1.00)  
`fn from(value: &str) -> Self`  
callers 13: `QueryRuntime::status` (apps/causeway-server/src/composition/state/query.rs:92), `SessionStreams::reply` (apps/causeway-server/src/composition/state/session_streams.rs:434), `SessionStreams::connections` (apps/causeway-server/src/composition/state/session_streams.rs:494) … 10 more  
calls 1: `SessionId::new` (apps/causeway-server/src/composition/state/session_id.rs:20)  
apps/causeway-server/src/composition/state/session_id.rs:19-21
```rust
    fn from(value: &str) -> Self {
        Self::new(value)
    }
```

### 32. Impl ConnectionSink#impl
`rust:apps/causeway-server/src/composition/state/session_streams.rs::ConnectionSink#impl`  apps/causeway-server/src/composition/state/session_streams.rs:238-304  pkg:cargo/causeway-server  score 0.64  via `ConnectionSink::offer_replies` <- DEFINES (apps/causeway-server/src/composition/state/session_streams.rs:275-291, 1.00)  
`impl ConnectionSink`  
members 6  
uses_types 1: `ConnectionSink` (apps/causeway-server/src/composition/state/session_streams.rs:238-304)  

### 33. Method SessionStreams::connections
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::connections`  apps/causeway-server/src/composition/state/session_streams.rs:492-496  pkg:cargo/causeway-server  score 0.64  via `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` <- TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:944, 1.00)  
`pub(super) fn connections(&self, session_id: &str) -> usize`  
Number of live connections of `session_id` (test observability).  
callers 3: `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` (apps/causeway-server/src/composition/state/session_streams.rs:944), `tests::a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace` (apps/causeway-server/src/composition/state/session_streams.rs:778), `RealtimeRuntime::session_connections` (apps/causeway-server/src/composition/state/realtime_runtime.rs:312)  
calls 2: `SessionId::from` (apps/causeway-server/src/composition/state/session_streams.rs:494), `SessionStreams::lock` (apps/causeway-server/src/composition/state/session_streams.rs:493)  
tests 2: `tests::a_stalled_tab_is_evicted_by_a_reply_while_a_tab_that_keeps_up_gets_them_all` (apps/causeway-server/src/composition/state/session_streams.rs:944), `tests::a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace` (apps/causeway-server/src/composition/state/session_streams.rs:778)  
apps/causeway-server/src/composition/state/session_streams.rs:492-496
```rust
    pub(super) fn connections(&self, session_id: &str) -> usize {
        self.lock()
            .get(&SessionId::from(session_id))
            .map_or(0, Vec::len)
    }
```

### 34. Const CONNECTION_QUEUE_CAPACITY
`rust:apps/causeway-server/src/composition/state/session_streams.rs::CONNECTION_QUEUE_CAPACITY`  apps/causeway-server/src/composition/state/session_streams.rs:66  pkg:cargo/causeway-server  score 0.63  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> REFERENCES (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:375, 1.00)  
`pub(crate) const CONNECTION_QUEUE_CAPACITY: usize = 256;`  
Data frames one connection may have queued before data producers wait, (doc continues, 134 bytes)  
importers 3: `apps/causeway-server/src/ws/tests/delivery.rs` (apps/causeway-server/src/ws/tests/delivery.rs:13), `apps/causeway-server/src/composition/state.rs` (apps/causeway-server/src/composition/state.rs:42-45), `tests` (apps/causeway-server/src/composition/state/session_streams.rs:620-623)  
referenced_by 2: `a_slow_subscriber_receives_every_session_frame_in_order` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:332), `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:375)  
apps/causeway-server/src/composition/state/session_streams.rs:66-66
```rust
pub(crate) const CONNECTION_QUEUE_CAPACITY: usize = 256;
```

### 35. Method SessionStreams::default
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::default`  apps/causeway-server/src/composition/state/session_streams.rs:328-330  pkg:cargo/causeway-server  score 0.62  via `tests::replies_past_the_backlog_bound_evict_the_tab` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:949, 1.00)  
`fn default() -> Self`  
callers 8: `RealtimeRuntime::default` (apps/causeway-server/src/composition/state/realtime_runtime.rs:84), `tests::a_slow_consumer_receives_every_frame_in_order` (apps/causeway-server/src/composition/state/session_streams.rs:783), `tests::an_eviction_watcher_ends_when_the_connection_closes` (apps/causeway-server/src/composition/state/session_streams.rs:1005) … 5 more  
calls 2: `SessionStreams::new` (apps/causeway-server/src/composition/state/session_streams.rs:329), `Backlog::new` (apps/causeway-server/src/composition/state/session_streams.rs:329)  
references 1: `DataBudget::DEFAULT` (apps/causeway-server/src/composition/state/session_streams.rs:329)  
tests 7: `tests::replies_past_the_backlog_bound_evict_the_tab` (apps/causeway-server/src/composition/state/session_streams.rs:949), `tests::dropping_the_last_feed_forgets_the_session_and_frees_waiting_producers` (apps/causeway-server/src/composition/state/session_streams.rs:866), `tests::a_full_queue_makes_the_producer_wait` (apps/causeway-server/src/composition/state/session_streams.rs:810) … 4 more  
apps/causeway-server/src/composition/state/session_streams.rs:328-330
```rust
    fn default() -> Self {
        Self::new(DataBudget::DEFAULT, SLOW_CONNECTION_GRACE)
    }
```

### 36. Struct AppState
`rust:apps/causeway-server/src/composition/state.rs::AppState`  apps/causeway-server/src/composition/state.rs:206-259  pkg:cargo/causeway-server  score 0.61  via `AppState#impl` -> USES_TYPE (apps/causeway-server/src/composition/state.rs:261-1021, 1.00)  
`pub(crate) struct AppState`  
members 153  
importers 138: `apps/causeway-server/src/agent/ws.rs` (apps/causeway-server/src/agent/ws.rs:11), `apps/causeway-server/src/routes/pages/query.rs` (apps/causeway-server/src/routes/pages/query.rs:8), `apps/causeway-server/src/routes/pages/semantic.rs` (apps/causeway-server/src/routes/pages/semantic.rs:10) … 135 more  
references 6: `access` (apps/causeway-server/src/composition/state.rs:236), `AppState::browser_auth` (apps/causeway-server/src/composition/state.rs:258), `services` (apps/causeway-server/src/composition/state.rs:252) … 3 more  
used_by 322: `metadata_quarantine_dismiss_batch` (apps/causeway-server/src/routes/pages/metadata_ops/quarantine.rs:177), `execute_catalog_query` (apps/causeway-server/src/catalog/ws.rs:77), `contract_review_ensure` (apps/causeway-server/src/routes/pages/contracts/review.rs:37) … 319 more  
uses_types 21: `MetadataAdminAuth` (apps/causeway-server/src/composition/state.rs:236), `DriveRuntime` (apps/causeway-server/src/composition/state.rs:218), `RpcAuthority` (apps/causeway-server/src/composition/state.rs:241) … 18 more  
apps/causeway-server/src/composition/state.rs:206-259
```rust
pub(crate) struct AppState {
    session: Arc<SessionRuntime>,
    content: Arc<ContentRuntime>,
    realtime: Arc<RealtimeRuntime>,
    /// Command settlements when no Valkey is attached (`settlement_store`).
    local_settlements: Arc<settlement_store::LocalSettlements>,
    services: Arc<ServiceRuntime>,
    search: Arc<SearchRuntime>,
    catalog_serving: causeway_catalog_host::CatalogServingState,
    cache: Arc<CacheRuntime>,
    platform: Arc<PlatformRuntime>,
    object_store: Arc<ObjectStoreRuntime>,
    drive: Arc<DriveRuntime>,
    catalog: Arc<CatalogRuntime>,
    query: Arc<QueryRuntime>,
    semantic: Arc<SemanticRuntime>,
    /// How this process answers the public catalog RPC surface
    /// (RFD-q276nkj9): in process from `catalog_serving`, or
    /// forwarded to the catalog read host. Startup sets it from the typed
    /// topology; construction defaults to in process.
    catalog_rpc: RpcServing,
    /// How this process answers the catalog publication commands
    /// (RFD-q276nkj9): the command host's service in process, or
    /// forwarded to the metadata admin command host. Never the shell's own
    /// code path: the shell records no decision.
    catalog_commands: CatalogCommandServing,
    /// The metadata-admin plane's authentication, built once at startup
    /// from the environment and the topology and handed to every surface
    /// that checks it: the metadata-ops routes, the review pages and the
    /// in-process commands. No surface builds its own (RFD-j1jabh49).
    metadata_admin: Arc<causeway_metadata_admin_host::access::MetadataAdminAuth>,
    /// The Cedar half of the RPC enforcement point, built once at startup
    /// with it. The session socket asks it for each command that reaches a
    /// method the Connect-RPC surface also serves, so the command is decided
    /// as its Connect twin is (ADR-bn90xr5m, decision 5).
    rpc_authority: connect_rpc_access::RpcAuthority,
    #[cfg(test)]
    ontology_access_override: Option<bool>,
    /// Digest of the catalog-publication policy bundle in force, loaded
    /// once at startup. `None` when the bundle cannot be read — surfaces
    /// then render binding provenance without a freshness claim rather
    /// than implying currency (RFD-0115: staleness is a rendered
    /// comparison, and no comparison is not the same as fresh).
    policy_bundle_digest: Option<Arc<str>>,
    /// Display projection of the ported usage-defaults policies, parsed
    /// from the same bundle load as the digest so the two cannot skew.
    usage_defaults: Arc<causeway_server_web::services::UsageDefaults>,
    /// Display projection of the compiled contract rights (RFD-2x49z609),
    /// from the same bundle load so it cannot skew from the digest.
    contract_rights: Arc<causeway_server_web::services::ContractRights>,
    /// Browser sign-in (RFD-q276nkj9), when the topology names a
    /// browser client. `None` leaves pages anonymous.
    browser_auth: Option<Arc<crate::browser_auth::BrowserAuth>>,
}
```

### 37. Method Backlog::stalled
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Backlog::stalled`  apps/causeway-server/src/composition/state/session_streams.rs:196-201  pkg:cargo/causeway-server  score 0.60  via `ConnectionSink::offer_replies` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:279, 1.00)  
`fn stalled(&self, grace: Duration) -> bool`  
Whether the connection holds frames and has taken none for `grace`.  
callers 1: `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:279)  
calls 11: `Backlog::now_ms` (apps/causeway-server/src/composition/state/session_streams.rs:198), `load` (apps/causeway-server/src/composition/state/session_streams.rs:199), `Shell::load` (apps/causeway-server/src/composition/state/session_streams.rs:200) … 8 more  
apps/causeway-server/src/composition/state/session_streams.rs:196-201
```rust
    fn stalled(&self, grace: Duration) -> bool {
        let idle_ms = self
            .now_ms()
            .saturating_sub(self.moved_at_ms.load(Ordering::Acquire));
        self.queued.load(Ordering::Acquire) > 0 && Duration::from_millis(idle_ms) >= grace
    }
```

### 38. File apps/causeway-server/src/transport/session_rpc/binding_tests.rs
`file:apps/causeway-server/src/transport/session_rpc/binding_tests.rs`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs  pkg:cargo/causeway-server  score 0.60  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` <- CONTAINS (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:368-404, 1.00)  
members 23  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:524)  
imports 20: `cargo/futures` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:18), `cargo/session-domain` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:20), `cargo/buffa` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:15) … 17 more  

### 39. Method Backlog::would_exceed_replies
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Backlog::would_exceed_replies`  apps/causeway-server/src/composition/state/session_streams.rs:203-208  pkg:cargo/causeway-server  score 0.60  via `ConnectionSink::offer_replies` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:279, 1.00)  
`fn would_exceed_replies(&self, bytes: usize) -> bool`  
callers 1: `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:279)  
calls 5: `load` (apps/causeway-server/src/composition/state/session_streams.rs:205), `load` (apps/causeway-server/src/composition/state/session_streams.rs:205), `Shell::load` (apps/causeway-server/src/composition/state/session_streams.rs:205) … 2 more  
apps/causeway-server/src/composition/state/session_streams.rs:203-208
```rust
    fn would_exceed_replies(&self, bytes: usize) -> bool {
        self.reply_bytes
            .load(Ordering::Acquire)
            .saturating_add(bytes)
            > REPLY_BACKLOG_BYTES
    }
```

### 40. Test tests::a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace`  apps/causeway-server/src/composition/state/session_streams.rs:754-779  pkg:cargo/causeway-server  score 0.57  via `SessionFeed::recv` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:769, 1.00)  
`async fn a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace()`  
calls 11: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:770), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:756), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:769) … 8 more  
tests_for 9: `tests::number` (apps/causeway-server/src/composition/state/session_streams.rs:769), `SessionStreams::connections` (apps/causeway-server/src/composition/state/session_streams.rs:778), `tests::wide_frame` (apps/causeway-server/src/composition/state/session_streams.rs:763) … 6 more  
apps/causeway-server/src/composition/state/session_streams.rs:754-779
```rust
    async fn a_tab_stalled_on_its_byte_budget_is_evicted_after_the_grace() {
        let streams = budgeted(128 * KIB, Duration::from_millis(100));
        let mut stalled = streams.subscribe("s");
        let mut live = streams.subscribe("s");
        let total = 8;
        let producer = {
            let streams = Arc::clone(&streams);
            tokio::spawn(async move {
                for n in 0..total {
                    streams.send("s", wide_frame(n, 64 * KIB)).await;
                }
            })
        };
        for n in 0..total {
            assert_eq!(
                live.recv().await.map(|f| number(&f)),
                Ok(frame(n)),
                "the live tab gets frame {n}"
            );
        }
        producer
            .await
            .expect("the producer finishes past the grace");
        assert_eq!(stalled.recv().await, Err(FellBehind));
        assert_eq!(streams.connections("s"), 1);
    }
```

### 41. Method Shell::subscribe
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::Shell::subscribe`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:89-122  pkg:cargo/causeway-server  score 0.56  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> CALLS (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:370, 1.00)  
`async fn subscribe( &self, token: &str, session_id: &str, ) -> ( ConnectionId, futures::stream::BoxStream<'static, Result<Bytes, ConnectError>>, )`  
Opens `token`'s `Subscribe` stream on `session_id` and returns the (doc continues, 158 bytes)  
callers 15: `another_persons_send_at_a_connection_is_refused_and_runs_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:235), `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:370), `a_send_the_enforcement_point_never_decided_is_refused` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:271) … 12 more  
calls 3: `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:106), `announced_connection_id` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:121), `PolicyDispatcher::call_server_streaming` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:104)  
tests 11: `a_counter_id_names_no_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:300), `disagreeing_keys_are_refused_and_run_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:493), `a_retry_on_a_new_subscribe_replays_the_settlement` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:515) … 8 more  
uses_types 3: `ConnectionId` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:94), `BoxStream` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:95), `BoxStream` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:95)  
apps/causeway-server/src/transport/session_rpc/binding_tests.rs:89-122
```rust
    async fn subscribe(
        &self,
        token: &str,
        session_id: &str,
    ) -> (
        ConnectionId,
        futures::stream::BoxStream<'static, Result<Bytes, ConnectError>>,
    ) {
        let request = pb::SubscribeRequest {
            session_id: session_id.to_owned(),
            ..Default::default()
        };
        let (mut stream, _ctx) = self
            .service
            .dispatcher()
            .call_server_streaming(
                SUBSCRIBE,
                Context::new(bearer(Some(token))),
                Bytes::from(request.encode_to_vec()),
                CodecFormat::Proto,
            )
            .await
            .expect("the subscriber is admitted");
        let hello = stream
            .next()
            .await
            .expect("a hello frame")
            .expect("the hello frame is not an error");
        let hello = pb::ServerFrame::decode_from_slice(&hello).expect("a server frame");
        let Some(pb::server_frame::Body::Control(control)) = hello.body else {
            panic!("the hello is a control frame");
        };
        (announced_connection_id(&control.envelope), stream)
    }
```

### 42. Variant Delivery::Gone
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Delivery::Gone`  apps/causeway-server/src/composition/state/session_streams.rs:310  pkg:cargo/causeway-server  score 0.56  via `ConnectionSink::offer_replies` -> REFERENCES (apps/causeway-server/src/composition/state/session_streams.rs:278, 1.00)  
`Gone`  
The connection is gone, or was evicted by another producer.  
referenced_by 2: `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:278), `SessionStreams::send` (apps/causeway-server/src/composition/state/session_streams.rs:410)  

### 43. Method Shell::ran_on
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::Shell::ran_on`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:159-165  pkg:cargo/causeway-server  score 0.54  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> CALLS (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:400, 1.00)  
`async fn ran_on(receiver: &mut crate::composition::state::SessionFeed) -> bool`  
Whether a command ran on `session_id` within a short wait: its (doc continues, 104 bytes)  
callers 6: `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:400), `a_send_the_enforcement_point_never_decided_is_refused` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:294), `another_persons_send_at_a_connection_is_refused_and_runs_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:247) … 3 more  
references 1: `state` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:159)  
tests 6: `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:400), `a_send_the_enforcement_point_never_decided_is_refused` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:294), `another_persons_send_at_a_connection_is_refused_and_runs_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:247) … 3 more  
uses_types 1: `SessionFeed` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:159)  
apps/causeway-server/src/transport/session_rpc/binding_tests.rs:159-165
```rust
    async fn ran_on(receiver: &mut crate::composition::state::SessionFeed) -> bool {
        tokio::time::timeout(Duration::from_millis(300), receiver.recv())
            .await
            .is_ok_and(|frame| {
                frame.is_ok_and(|frame| String::from_utf8_lossy(&frame).contains("probe"))
            })
    }
```

### 44. Function subscribe_session
`rust:apps/causeway-server/src/realtime.rs::subscribe_session`  apps/causeway-server/src/realtime.rs:30-32  pkg:cargo/causeway-server  score 0.53  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> CALLS (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:373, 1.00)  
`pub(crate) fn subscribe_session(state: &AppState, session_id: &str) -> SessionFeed`  
callers 13: `the_same_key_in_the_frame_and_the_command_is_one_key` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:464), `ConnectionOutbox::open` (apps/causeway-server/src/realtime.rs:102), `disagreeing_keys_are_refused_and_run_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:494) … 10 more  
calls 2: `AppState::session_subscribe` (apps/causeway-server/src/realtime.rs:31), `RealtimeRuntime::session_subscribe` (apps/causeway-server/src/realtime.rs:31)  
tests 10: `the_subscriber_sends_on_its_own_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:222), `the_client_frames_own_idempotency_key_replays_a_resend` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:420), `the_same_key_in_the_frame_and_the_command_is_one_key` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:464) … 7 more  
uses_types 2: `AppState` (apps/causeway-server/src/realtime.rs:30), `SessionFeed` (apps/causeway-server/src/realtime.rs:30)  
apps/causeway-server/src/realtime.rs:30-32
```rust
pub(crate) fn subscribe_session(state: &AppState, session_id: &str) -> SessionFeed {
    state.session_subscribe(session_id)
}
```

### 45. Method ConnectionSink::enqueue
`rust:apps/causeway-server/src/composition/state/session_streams.rs::ConnectionSink::enqueue`  apps/causeway-server/src/composition/state/session_streams.rs:293-303  pkg:cargo/causeway-server  score 0.53  via `ConnectionSink::offer_replies` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:283, 1.00)  
`fn enqueue(&self, frame: Vec<u8>, lane: Lane)`  
callers 2: `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:283), `SessionStreams::send` (apps/causeway-server/src/composition/state/session_streams.rs:407)  
calls 3: `Backlog::pushed` (apps/causeway-server/src/composition/state/session_streams.rs:301), `SessionSender::send` (apps/causeway-server/src/composition/state/session_streams.rs:300), `SessionStreams::send` (apps/causeway-server/src/composition/state/session_streams.rs:300)  
references 1: `Lane::Reply` (apps/causeway-server/src/composition/state/session_streams.rs:295)  
uses_types 4: `Lane` (apps/causeway-server/src/composition/state/session_streams.rs:293), `Queued` (apps/causeway-server/src/composition/state/session_streams.rs:300), `Data` (apps/causeway-server/src/composition/state/session_streams.rs:296) … 1 more  
apps/causeway-server/src/composition/state/session_streams.rs:293-303
```rust
    fn enqueue(&self, frame: Vec<u8>, lane: Lane) {
        let reply_bytes = match lane {
            Lane::Reply => frame.len(),
            Lane::Data { .. } => 0,
        };
        // A closed channel means the connection is gone; its feed's drop
        // unregisters it, and the frame's room goes with it.
        if self.frames.send(Queued { frame, lane }).is_ok() {
            self.backlog.pushed(reply_bytes);
        }
    }
```

### 46. Test another_persons_send_at_a_connection_is_refused_and_runs_nothing
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::another_persons_send_at_a_connection_is_refused_and_runs_nothing`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:233-250  pkg:cargo/causeway-server  score 0.52  via `subscribe_session` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:236, 1.00)  
`async fn another_persons_send_at_a_connection_is_refused_and_runs_nothing()`  
calls 6: `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:235), `Shell::ran_on` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:247), `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:241) … 3 more  
tests_for 6: `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:241), `Shell::send` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:241), `Shell::ran_on` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:247) … 3 more  
apps/causeway-server/src/transport/session_rpc/binding_tests.rs:233-250
```rust
async fn another_persons_send_at_a_connection_is_refused_and_runs_nothing() {
    let shell = Shell::start().await;
    let (connection, _stream) = shell.subscribe(&shell.alice, "alice-session").await;
    let mut session = crate::realtime::subscribe_session(&shell.state, "alice-session");

    // Bob carries no session cookie: the check that once looked only at a
    // cookie, and skipped itself without one, is gone.
    let refused = shell
        .send(bearer(Some(&shell.bob)), &connection.to_string())
        .await
        .expect_err("Bob's Send at Alice's connection");

    assert_eq!(refused.code, ErrorCode::PermissionDenied, "{refused:?}");
    assert!(
        !Shell::ran_on(&mut session).await,
        "nothing ran on Alice's connection"
    );
}
```

### 47. Test tests::a_tab_that_stops_draining_is_evicted_and_the_others_carry_on
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_tab_that_stops_draining_is_evicted_and_the_others_carry_on`  apps/causeway-server/src/composition/state/session_streams.rs:826-862  pkg:cargo/causeway-server  score 0.51  via `SessionFeed::recv` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:842, 1.00)  
`async fn a_tab_that_stops_draining_is_evicted_and_the_others_carry_on()`  
calls 14: `SessionStreams::with_grace` (apps/causeway-server/src/composition/state/session_streams.rs:827), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:842), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:828) … 11 more  
tests_for 6: `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:828), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:842), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:836) … 3 more  

### 48. File apps/causeway-server/src/composition/state.rs
`file:apps/causeway-server/src/composition/state.rs`  apps/causeway-server/src/composition/state.rs  pkg:cargo/causeway-server  score 0.50  via `AppState#impl` <- CONTAINS (apps/causeway-server/src/composition/state.rs:261-1021, 1.00)  
members 30  
designs 1: `docs/rfd/RFD-2cqt6wf1-web-session-state-and-recovery.md` (docs/rfd/RFD-2cqt6wf1-web-session-state-and-recovery.md:83)  
documents 1: `docs/DESIGN-LOG.md` (docs/DESIGN-LOG.md:476)  
imports 32: `cargo/drive-runtime` (apps/causeway-server/src/composition/state.rs:30), `SessionRuntime` (apps/causeway-server/src/composition/state.rs:40), `PlatformRuntime` (apps/causeway-server/src/composition/state.rs:31) … 29 more  

### 49. Function run_session_actor
`rust:apps/causeway-server/src/ws/session.rs::run_session_actor`  apps/causeway-server/src/ws/session.rs:172-215  pkg:cargo/causeway-server  score 0.50  via `reply_to_session` <- CALLS (apps/causeway-server/src/ws/session.rs:211, 1.00)  
`pub(crate) async fn run_session_actor( session_id: String, state: AppState, mut inputs: mpsc::Receiver<SessionInput>, )`  
Runs one session's apply loop. It drains client frames from every (doc continues, 388 bytes)  
callers 2: `serve` (apps/causeway-server/src/ws/socket.rs:92), `Session::subscribe` (apps/causeway-server/src/transport/session_rpc.rs:187)  
calls 5: `handle_frame` (apps/causeway-server/src/ws/session.rs:200), `as_browser_caller` (apps/causeway-server/src/ws/session.rs:198), `SessionConnection::id` (apps/causeway-server/src/ws/session.rs:186) … 2 more  
importers 2: `apps/causeway-server/src/transport/session_rpc.rs` (apps/causeway-server/src/transport/session_rpc.rs:50), `apps/causeway-server/src/ws.rs` (apps/causeway-server/src/ws.rs:28-31)  
uses_types 3: `AppState` (apps/causeway-server/src/ws/session.rs:174), `SessionInput` (apps/causeway-server/src/ws/session.rs:175), `ClientFrame` (apps/causeway-server/src/ws/session.rs:179)  

### 50. Struct DataBudget
`rust:apps/causeway-server/src/composition/state/session_streams.rs::DataBudget`  apps/causeway-server/src/composition/state/session_streams.rs:74  pkg:cargo/causeway-server  score 0.48  via `tests` -> IMPORTS (apps/causeway-server/src/composition/state/session_streams.rs:620-623, 1.00)  
`pub(crate) struct DataBudget`  
The bytes of data frames one connection may hold queued before data (doc continues, 211 bytes)  
members 7  
importers 4: `tests` (apps/causeway-server/src/composition/state/session_streams.rs:620-623), `apps/causeway-server/src/startup/config.rs` (apps/causeway-server/src/startup/config.rs:17), `apps/causeway-server/src/composition/state.rs` (apps/causeway-server/src/composition/state.rs:42-45) … 1 more  
used_by 9: `SessionStreams` (apps/causeway-server/src/composition/state/session_streams.rs:322), `RealtimeRuntime::with_session_data_budget` (apps/causeway-server/src/composition/state/realtime_runtime.rs:96), `ServerConfig` (apps/causeway-server/src/startup/config.rs:75) … 6 more  
apps/causeway-server/src/composition/state/session_streams.rs:74-74
```rust
pub(crate) struct DataBudget(NonZeroU32);
```

### 51. Function bearer
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::bearer`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:169-178  pkg:cargo/causeway-server  score 0.48  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> CALLS (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:396, 1.00)  
`fn bearer(token: Option<&str>) -> HeaderMap`  
`Authorization: Bearer <token>`, or no credential at all.  
callers 12: `a_send_without_a_credential_is_refused_and_runs_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:260), `the_client_frames_own_idempotency_key_replays_a_resend` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:421), `the_subscriber_sends_on_its_own_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:225) … 9 more  
tests 10: `the_client_frames_own_idempotency_key_replays_a_resend` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:421), `the_subscriber_sends_on_its_own_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:225), `a_counter_id_names_no_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:304) … 7 more  
apps/causeway-server/src/transport/session_rpc/binding_tests.rs:169-178
```rust
fn bearer(token: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    if let Some(token) = token {
        headers.insert(
            header::AUTHORIZATION,
            format!("Bearer {token}").parse().expect("a header value"),
        );
    }
    headers
}
```

### 52. Impl AppState#impl
`rust:apps/causeway-server/src/composition/state.rs::AppState#impl`  apps/causeway-server/src/composition/state.rs:261-1021  pkg:cargo/causeway-server  score 0.48  via `AppState::reply_to_session` <- DEFINES (apps/causeway-server/src/composition/state.rs:909-911, 1.00)  
`impl AppState`  
members 86  
uses_types 1: `AppState` (apps/causeway-server/src/composition/state.rs:261-1021)  

### 53. Test a_send_without_a_credential_is_refused_and_runs_nothing
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::a_send_without_a_credential_is_refused_and_runs_nothing`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:253-266  pkg:cargo/causeway-server  score 0.46  via `subscribe_session` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:256, 1.00)  
`async fn a_send_without_a_credential_is_refused_and_runs_nothing()`  
calls 6: `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:260), `Shell::start` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:254), `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:255) … 3 more  
tests_for 6: `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:255), `Shell::ran_on` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:265), `Shell::send` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:260) … 3 more  

### 54. Test the_subscriber_sends_on_its_own_connection
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::the_subscriber_sends_on_its_own_connection`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:219-230  pkg:cargo/causeway-server  score 0.46  via `subscribe_session` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:222, 1.00)  
`async fn the_subscriber_sends_on_its_own_connection()`  
calls 6: `Shell::send` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:225), `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:221), `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:225) … 3 more  
tests_for 6: `subscribe_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:222), `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:225), `Shell::start` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:220) … 3 more  

### 55. Test a_send_the_enforcement_point_never_decided_is_refused
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::a_send_the_enforcement_point_never_decided_is_refused`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:269-295  pkg:cargo/causeway-server  score 0.45  via `subscribe_session` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:272, 1.00)  
`async fn a_send_the_enforcement_point_never_decided_is_refused()`  
calls 12: `Session::new` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:276), `Shell::ran_on` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:294), `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:271) … 9 more  
tests_for 12: `Shell::ran_on` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:294), `Session::new` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:276), `subscribe_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:272) … 9 more  
uses_types 1: `ClientFrame` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:277)  

### 56. Variant Delivery::Queued
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Delivery::Queued`  apps/causeway-server/src/composition/state/session_streams.rs:308  pkg:cargo/causeway-server  score 0.45  via `ConnectionSink::offer_replies` -> REFERENCES (apps/causeway-server/src/composition/state/session_streams.rs:288, 1.00)  
`Queued`  
referenced_by 2: `ConnectionSink::offer_replies` (apps/causeway-server/src/composition/state/session_streams.rs:288), `SessionStreams::send` (apps/causeway-server/src/composition/state/session_streams.rs:408)  

### 57. Test a_slow_subscriber_receives_every_session_frame_in_order
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::a_slow_subscriber_receives_every_session_frame_in_order`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:327-362  pkg:cargo/causeway-server  score 0.44  via `CONNECTION_QUEUE_CAPACITY` <- REFERENCES (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:332, 1.00)  
`async fn a_slow_subscriber_receives_every_session_frame_in_order()`  
A `Subscribe` stream read slowly still receives every session frame, in (doc continues, 149 bytes)  
calls 3: `send_to_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:338), `Shell::start` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:328), `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:329)  
references 2: `CONNECTION_QUEUE_CAPACITY` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:332), `TAG_PROTO` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:337)  
tests_for 3: `Shell::start` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:328), `send_to_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:338), `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:329)  

### 58. Function send_to_session
`rust:apps/causeway-server/src/realtime.rs::send_to_session`  apps/causeway-server/src/realtime.rs:34-36  pkg:cargo/causeway-server  score 0.44  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> CALLS (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:381, 1.00)  
`pub(crate) async fn send_to_session(state: &AppState, session_id: &str, frame: Vec<u8>)`  
callers 15: `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:381), `QueryAppendStream::send` (apps/causeway-server/src/query_session/batch.rs:46), `a_slow_subscriber_receives_every_session_frame_in_order` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:338) … 12 more  
calls 2: `AppState::send_to_session` (apps/causeway-server/src/realtime.rs:35), `RealtimeRuntime::send_to_session` (apps/causeway-server/src/realtime.rs:35)  
tests 5: `a_slow_subscriber_receives_every_session_frame_in_order` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:338), `a_slow_socket_receives_every_session_frame_in_order` (apps/causeway-server/src/ws/tests/delivery.rs:82), `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:381) … 2 more  
uses_types 1: `AppState` (apps/causeway-server/src/realtime.rs:34)  

### 59. Method Shell::send
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::Shell::send`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:125-128  pkg:cargo/causeway-server  score 0.44  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> CALLS (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:396, 1.00)  
`async fn send(&self, headers: HeaderMap, connection: &str) -> Result<(), ConnectError>`  
Sends a `query.status` command at `connection` with `headers`.  
callers 6: `the_subscriber_sends_on_its_own_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:225), `a_counter_id_names_no_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:314), `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:396) … 3 more  
calls 2: `Shell::send_frame` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:126), `query_status_command` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:126)  
tests 5: `a_counter_id_names_no_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:304), `another_persons_send_at_a_connection_is_refused_and_runs_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:241), `the_subscriber_sends_on_its_own_connection` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:225) … 2 more  

### 60. Method Shell::start
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::Shell::start`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:67-84  pkg:cargo/causeway-server  score 0.44  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> CALLS (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:369, 1.00)  
`async fn start() -> Self`  
callers 46: `the_client_frames_own_idempotency_key_replays_a_resend` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:418), `a_send_without_a_credential_is_refused_and_runs_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:254), `the_same_key_in_the_frame_and_the_command_is_one_key` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:462) … 43 more  
calls 10: `AppState::mock_for_tests` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:70), `RpcAccessControl::oidc` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:69), `Session::new` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:72) … 7 more  
tests 45: `a_slow_subscriber_receives_every_session_frame_in_order` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:328), `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:369), `disagreeing_keys_are_refused_and_run_nothing` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:492) … 42 more  

### 61. Struct FellBehind
`rust:apps/causeway-server/src/composition/state/session_streams.rs::FellBehind`  apps/causeway-server/src/composition/state/session_streams.rs:136  pkg:cargo/causeway-server  score 0.41  via `tests` -> IMPORTS (apps/causeway-server/src/composition/state/session_streams.rs:620-623, 1.00)  
`pub(crate) struct FellBehind;`  
A connection fell too far behind its session's frames and was closed; (doc continues, 108 bytes)  
importers 3: `apps/causeway-server/src/composition/state.rs` (apps/causeway-server/src/composition/state.rs:42-45), `apps/causeway-server/src/realtime.rs` (apps/causeway-server/src/realtime.rs:24), `tests` (apps/causeway-server/src/composition/state/session_streams.rs:620-623)  
used_by 1: `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:573)  

### 62. Struct SessionId
`rust:apps/causeway-server/src/composition/state/session_id.rs::SessionId`  apps/causeway-server/src/composition/state/session_id.rs:6  pkg:cargo/causeway-server  score 0.41  via `SessionStreams::evict` -> USES_TYPE (apps/causeway-server/src/composition/state/session_streams.rs:452, 1.00)  
`pub(crate) struct SessionId`  
Session identity for server-local UI state. (doc continues, 145 bytes)  
members 3  
importers 4: `apps/causeway-server/src/composition/state/catalog_runtime.rs` (apps/causeway-server/src/composition/state/catalog_runtime.rs:13), `apps/causeway-server/src/composition/state.rs` (apps/causeway-server/src/composition/state.rs:39), `apps/causeway-server/src/composition/state/query.rs` (apps/causeway-server/src/composition/state/query.rs:9) … 1 more  
used_by 12: `SessionStreams::unregister` (apps/causeway-server/src/composition/state/session_streams.rs:462), `CatalogRuntime` (apps/causeway-server/src/composition/state/catalog_runtime.rs:30), `SessionStreams::sinks` (apps/causeway-server/src/composition/state/session_streams.rs:447) … 9 more  
uses_types 1: `SessionId` (apps/causeway-server/src/composition/state/session_id.rs:6)  

### 63. Test a_counter_id_names_no_connection
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::a_counter_id_names_no_connection`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:298-321  pkg:cargo/causeway-server  score 0.40  via `Shell::send` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:304, 1.00)  
`async fn a_counter_id_names_no_connection()`  
calls 7: `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:315), `ConnectionId::mint` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:316), `Shell::send` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:314) … 4 more  
tests_for 5: `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:300), `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:304), `Shell::start` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:299) … 2 more  

### 64. Test disagreeing_keys_are_refused_and_run_nothing
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::disagreeing_keys_are_refused_and_run_nothing`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:491-507  pkg:cargo/causeway-server  score 0.39  via `subscribe_session` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:494, 1.00)  
`async fn disagreeing_keys_are_refused_and_run_nothing()`  
calls 7: `Shell::send_frame` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:497), `subscribe_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:494), `keyed_query_status_command` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:500) … 4 more  
tests_for 7: `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:493), `Shell::start` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:492), `subscribe_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:494) … 4 more  

### 65. Struct SessionFeed
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionFeed`  apps/causeway-server/src/composition/state/session_streams.rs:556-561  pkg:cargo/causeway-server  score 0.38  via `subscribe_session` -> USES_TYPE (apps/causeway-server/src/realtime.rs:30, 1.00)  
`pub(crate) struct SessionFeed`  
One connection's view of its session's frames, in send order. (doc continues, 155 bytes)  
members 3  
importers 3: `apps/causeway-server/src/composition/state/realtime_runtime.rs` (apps/causeway-server/src/composition/state/realtime_runtime.rs:12), `apps/causeway-server/src/realtime.rs` (apps/causeway-server/src/realtime.rs:24), `apps/causeway-server/src/composition/state.rs` (apps/causeway-server/src/composition/state.rs:42-45)  
used_by 8: `SessionFeed#impl` (apps/causeway-server/src/composition/state/session_streams.rs:563-596), `AppState::session_subscribe` (apps/causeway-server/src/composition/state.rs:902), `Shell::ran_on` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:159) … 5 more  
uses_types 4: `Backlog` (apps/causeway-server/src/composition/state/session_streams.rs:558), `Registration` (apps/causeway-server/src/composition/state/session_streams.rs:560), `Queued` (apps/causeway-server/src/composition/state/session_streams.rs:557) … 1 more  

### 66. Test tests::a_full_queue_makes_the_producer_wait
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_full_queue_makes_the_producer_wait`  apps/causeway-server/src/composition/state/session_streams.rs:809-823  pkg:cargo/causeway-server  score 0.38  via `SessionFeed::recv` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:819, 1.00)  
`async fn a_full_queue_makes_the_producer_wait()`  
calls 13: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:813), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:816), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:819) … 10 more  
tests_for 6: `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:819), `SessionStreams::default` (apps/causeway-server/src/composition/state/session_streams.rs:810), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:813) … 3 more  

### 67. Test tests::a_slow_consumer_receives_every_frame_in_order
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_slow_consumer_receives_every_frame_in_order`  apps/causeway-server/src/composition/state/session_streams.rs:782-806  pkg:cargo/causeway-server  score 0.38  via `SessionFeed::recv` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:799, 1.00)  
`async fn a_slow_consumer_receives_every_frame_in_order()`  
calls 7: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:791), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:784), `SessionStreams::default` (apps/causeway-server/src/composition/state/session_streams.rs:783) … 4 more  
tests_for 6: `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:791), `SessionStreams::default` (apps/causeway-server/src/composition/state/session_streams.rs:783), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:784) … 3 more  

### 68. Test tests::a_slow_wide_connection_never_holds_more_than_its_budget
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_slow_wide_connection_never_holds_more_than_its_budget`  apps/causeway-server/src/composition/state/session_streams.rs:715-751  pkg:cargo/causeway-server  score 0.37  via `SessionFeed::recv` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:739, 1.00)  
`async fn a_slow_wide_connection_never_holds_more_than_its_budget()`  
calls 10: `tests::number` (apps/causeway-server/src/composition/state/session_streams.rs:740), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:739), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:740) … 7 more  
tests_for 10: `tests::number` (apps/causeway-server/src/composition/state/session_streams.rs:740), `tests::wide_frame` (apps/causeway-server/src/composition/state/session_streams.rs:726), `tests::budgeted` (apps/causeway-server/src/composition/state/session_streams.rs:717) … 7 more  

### 69. Enum Lane
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Lane`  apps/causeway-server/src/composition/state/session_streams.rs:163-168  pkg:cargo/causeway-server  score 0.37  via `Lane::Reply` <- DEFINES (apps/causeway-server/src/composition/state/session_streams.rs:167, 1.00)  
`enum Lane`  
Which kind of frame is queued, and the room it holds until taken.  
members 2  
used_by 2: `ConnectionSink::enqueue` (apps/causeway-server/src/composition/state/session_streams.rs:293), `Queued` (apps/causeway-server/src/composition/state/session_streams.rs:152)  

### 70. Impl Shell#impl
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::Shell#impl`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:66-166  pkg:cargo/causeway-server  score 0.36  via `Shell::ran_on` <- DEFINES (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:159-165, 1.00)  
`impl Shell`  
members 5  
uses_types 1: `Shell` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:66-166)  

### 71. Impl Backlog#impl
`rust:apps/causeway-server/src/composition/state/session_streams.rs::Backlog#impl`  apps/causeway-server/src/composition/state/session_streams.rs:181-222  pkg:cargo/causeway-server  score 0.36  via `Backlog::stalled` <- DEFINES (apps/causeway-server/src/composition/state/session_streams.rs:196-201, 1.00)  
`impl Backlog`  
members 6  
uses_types 1: `Backlog` (apps/causeway-server/src/composition/state/session_streams.rs:181-222)  

### 72. Test a_retry_on_a_new_subscribe_replays_the_settlement
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::a_retry_on_a_new_subscribe_replays_the_settlement`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:510-553  pkg:cargo/causeway-server  score 0.35  via `subscribe_session` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:513, 1.00)  
`async fn a_retry_on_a_new_subscribe_replays_the_settlement()`  
calls 13: `Shell::send_frame` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:540), `Shell::send_frame` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:517), `subscribe_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:513) … 10 more  
tests_for 9: `Shell::send_frame` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:517), `Shell::subscribe` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:515), `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:512) … 6 more  

### 73. Const TAG_PROTO
`rust:torrentfx/crates/protocol-wire/src/lib.rs::TAG_PROTO`  torrentfx/crates/protocol-wire/src/lib.rs:12  pkg:cargo/protocol-wire  score 0.35  via `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` -> REFERENCES (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:380, 1.00)  
`pub const TAG_PROTO: u8 = 0x01;`  
Proto envelope tag byte.  
importers 4: `tests` (apps/causeway-server/src/ws/session/idempotency.rs:107), `apps/causeway-server/src/ws/tests/live_socket.rs` (apps/causeway-server/src/ws/tests/live_socket.rs:9), `tests` (apps/causeway-server/src/transport/session_rpc.rs:418) … 1 more  
referenced_by 8: `action_frame` (apps/causeway-server/src/ws/tests/explorer.rs:304), `a_stalled_tab_does_not_hold_up_replies_to_the_sessions_other_tabs` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:380), `an_evicted_tab_blocked_in_a_socket_write_is_torn_down_at_once` (apps/causeway-server/src/ws/tests/delivery.rs:149) … 5 more  

### 74. Impl SessionId#impl(From)
`rust:apps/causeway-server/src/composition/state/session_id.rs::SessionId#impl(From)`  apps/causeway-server/src/composition/state/session_id.rs:18-22  pkg:cargo/causeway-server  score 0.35  via `SessionId::from` <- DEFINES (apps/causeway-server/src/composition/state/session_id.rs:19-21, 1.00)  
`impl From<&str> for SessionId`  
members 1  
uses_types 1: `SessionId` (apps/causeway-server/src/composition/state/session_id.rs:18-22)  

### 75. Method SessionStreams::lock
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::lock`  apps/causeway-server/src/composition/state/session_streams.rs:498-505  pkg:cargo/causeway-server  score 0.34  via `SessionStreams::sinks` -> CALLS (apps/causeway-server/src/composition/state/session_streams.rs:448, 1.00)  
`fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<SessionId, Vec<ConnectionSink>>>`  
callers 6: `SessionStreams::unregister` (apps/causeway-server/src/composition/state/session_streams.rs:463), `SessionStreams::lock` (apps/causeway-server/src/composition/state/session_streams.rs:503), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:368) … 3 more  
calls 1: `SessionStreams::lock` (apps/causeway-server/src/composition/state/session_streams.rs:503)  
uses_types 2: `ConnectionSink` (apps/causeway-server/src/composition/state/session_streams.rs:498), `SessionId` (apps/causeway-server/src/composition/state/session_streams.rs:498)  

### 76. Enum ServiceEnded
`rust:apps/causeway-server/src/composition/state/session_streams.rs::ServiceEnded`  apps/causeway-server/src/composition/state/session_streams.rs:530-535  pkg:cargo/causeway-server  score 0.33  via `tests` -> IMPORTS (apps/causeway-server/src/composition/state/session_streams.rs:620-623, 1.00)  
`pub(crate) enum ServiceEnded`  
How a watched connection's service ended.  
members 2  
importers 3: `apps/causeway-server/src/transport/session_rpc.rs` (apps/causeway-server/src/transport/session_rpc.rs:48), `tests` (apps/causeway-server/src/composition/state/session_streams.rs:620-623), `apps/causeway-server/src/composition/state.rs` (apps/causeway-server/src/composition/state.rs:42-45)  
used_by 1: `Eviction::wait` (apps/causeway-server/src/composition/state/session_streams.rs:540)  

### 77. Test tests::a_frame_larger_than_the_budget_passes_alone
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::a_frame_larger_than_the_budget_passes_alone`  apps/causeway-server/src/composition/state/session_streams.rs:693-712  pkg:cargo/causeway-server  score 0.33  via `SessionFeed::recv` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:708, 1.00)  
`async fn a_frame_larger_than_the_budget_passes_alone()`  
calls 12: `tests::budgeted` (apps/causeway-server/src/composition/state/session_streams.rs:694), `tests::wide_frame` (apps/causeway-server/src/composition/state/session_streams.rs:698), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:695) … 9 more  
tests_for 7: `tests::wide_frame` (apps/causeway-server/src/composition/state/session_streams.rs:698), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:703), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:695) … 4 more  

### 78. Test tests::sessions_are_isolated
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::sessions_are_isolated`  apps/causeway-server/src/composition/state/session_streams.rs:885-896  pkg:cargo/causeway-server  score 0.33  via `SessionFeed::recv` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:890, 1.00)  
`async fn sessions_are_isolated()`  
calls 9: `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:890), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:888), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:892) … 6 more  
tests_for 6: `SessionStreams::default` (apps/causeway-server/src/composition/state/session_streams.rs:886), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:889), `SessionFeed::recv` (apps/causeway-server/src/composition/state/session_streams.rs:890) … 3 more  

### 79. Test the_client_frames_own_idempotency_key_replays_a_resend
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::the_client_frames_own_idempotency_key_replays_a_resend`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:417-458  pkg:cargo/causeway-server  score 0.32  via `subscribe_session` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:420, 1.00)  
`async fn the_client_frames_own_idempotency_key_replays_a_resend()`  
calls 11: `query_status_command` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:442), `query_status_command` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:428), `Shell::start` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:418) … 8 more  
tests_for 7: `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:421), `subscribe_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:420), `Shell::send_frame` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:425) … 4 more  

### 80. Test the_same_key_in_the_frame_and_the_command_is_one_key
`rust:apps/causeway-server/src/transport/session_rpc/binding_tests.rs::the_same_key_in_the_frame_and_the_command_is_one_key`  apps/causeway-server/src/transport/session_rpc/binding_tests.rs:461-488  pkg:cargo/causeway-server  score 0.32  via `subscribe_session` -> TESTED_BY (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:464, 1.00)  
`async fn the_same_key_in_the_frame_and_the_command_is_one_key()`  
calls 10: `subscribe_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:464), `next_settlement` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:487), `Shell::send_frame` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:479) … 7 more  
tests_for 7: `subscribe_session` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:464), `bearer` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:465), `next_settlement` (apps/causeway-server/src/transport/session_rpc/binding_tests.rs:476) … 4 more  

### 81. Method SessionStreams::held_data_bytes
`rust:apps/causeway-server/src/composition/state/session_streams.rs::SessionStreams::held_data_bytes`  apps/causeway-server/src/composition/state/session_streams.rs:483-488  pkg:cargo/causeway-server  score 0.32  via `SessionStreams::sinks` <- CALLS (apps/causeway-server/src/composition/state/session_streams.rs:484, 1.00)  
`pub(super) fn held_data_bytes(&self, session_id: &str) -> Vec<usize>`  
Bytes of data frames queued for each live connection of (doc continues, 91 bytes)  
callers 3: `tests::a_slow_wide_connection_never_holds_more_than_its_budget` (apps/causeway-server/src/composition/state/session_streams.rs:736), `tests::data_frames_are_bounded_in_bytes_not_only_in_frames` (apps/causeway-server/src/composition/state/session_streams.rs:689), `tests::data_frames_are_bounded_in_bytes_not_only_in_frames` (apps/causeway-server/src/composition/state/session_streams.rs:670)  
calls 2: `SessionStreams::sinks` (apps/causeway-server/src/composition/state/session_streams.rs:484), `SessionId::from` (apps/causeway-server/src/composition/state/session_streams.rs:484)  
references 1: `ConnectionSink::held_data_bytes` (apps/causeway-server/src/composition/state/session_streams.rs:486)  
tests 2: `tests::data_frames_are_bounded_in_bytes_not_only_in_frames` (apps/causeway-server/src/composition/state/session_streams.rs:670), `tests::a_slow_wide_connection_never_holds_more_than_its_budget` (apps/causeway-server/src/composition/state/session_streams.rs:736)  

### 82. Package cargo/tokio
`pkg:cargo/tokio`  pkg:cargo/tokio  score 0.32  via `tests` -> IMPORTS (apps/causeway-server/src/composition/state/session_streams.rs:618, 1.00)  
dependents 111: `cargo/cw-host` (bootstrap/cw-host/Cargo.toml:28), `cargo/causeway-capability` (crates/causeway-capability/Cargo.toml:16), `cargo/catalog-query-understanding` (component/discovery/search/query-understanding/Cargo.toml:20) … 108 more  
importers 91: `dev_dependencies` (crates/ingest-corpus/src/lib.rs:41), `crates/causeway-metadata-ops/src/materializer.rs` (crates/causeway-metadata-ops/src/materializer.rs:16), `tests` (component/discovery/search-feedback/src/lib.rs:877) … 88 more  

### 83. Test tests::data_frames_are_bounded_in_bytes_not_only_in_frames
`rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::data_frames_are_bounded_in_bytes_not_only_in_frames`  apps/causeway-server/src/composition/state/session_streams.rs:664-690  pkg:cargo/causeway-server  score 0.32  via `SessionFeed::recv` -> TESTED_BY (apps/causeway-server/src/composition/state/session_streams.rs:682, 1.00)  
`async fn data_frames_are_bounded_in_bytes_not_only_in_frames()`  
calls 18: `tests::wide_frame` (apps/causeway-server/src/composition/state/session_streams.rs:674), `tests::wide_frame` (apps/causeway-server/src/composition/state/session_streams.rs:668), `SessionStreams::subscribe` (apps/causeway-server/src/composition/state/session_streams.rs:666) … 15 more  
tests_for 10: `tests::wide_frame` (apps/causeway-server/src/composition/state/session_streams.rs:668), `tests::frame` (apps/causeway-server/src/composition/state/session_streams.rs:682), `tests::number` (apps/causeway-server/src/composition/state/session_streams.rs:682) … 7 more  

## Left out

28 included nodes have no source window for want of budget.  
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::tests::dropping_the_last_feed_forgets_the_session_and_frees_waiting_producers` Test 0.31 (over budget, 287 tokens)
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::ConnectionSink::evict` Method 0.30 (over budget, 211 tokens)
- `rust:apps/causeway-server/src/composition/state/session_streams.rs::Registration` Struct 0.30 (over budget, 249 tokens)
- and 599 more

## Unresolved references in the seeds' files

- `plaintext` at apps/causeway-server/src/composition/state.rs:100 (no-declaration)
- `Forwarded` at apps/causeway-server/src/composition/state.rs:103 (no-declaration)
- `review_catalog_publication_with_options` at apps/causeway-server/src/composition/state.rs:129 (no-declaration)
- `propose_taxonomy_placement_with_options` at apps/causeway-server/src/composition/state.rs:153 (no-declaration)
- `decide_taxonomy_placement_with_options` at apps/causeway-server/src/composition/state.rs:177 (no-declaration)
- `InProcess` at apps/causeway-server/src/composition/state.rs:330 (no-declaration)
- `debug_struct` at apps/causeway-server/src/composition/state.rs:80 (no-declaration)
- `finish_non_exhaustive` at apps/causeway-server/src/composition/state.rs:82 (no-declaration)
- `now` at apps/causeway-server/src/composition/state.rs:825 (qualifier-mismatch)
- `now` at apps/causeway-server/src/composition/state.rs:864 (qualifier-mismatch)

