use outbox::relay::Drain;
use outbox::Relay;

fn main() {
    let mut relay = Relay::new(Default::default());
    let sent = relay.drain(100);
    println!("{sent}");
    report_users();
}

fn report_users() {
    let _ = sqlx::query_as::<_, (i64,)>("SELECT count(*) FROM cw_users.users WHERE active");
}
