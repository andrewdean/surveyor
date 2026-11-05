"""Database access for the reporting service."""


class Reports:
    """Runs reporting queries."""

    def active_users(self, cur):
        cur.execute("SELECT count(*) FROM cw_users.users WHERE active")
        return cur.fetchone()

    def backlog(self, cur):
        cur.execute("SELECT waiting FROM outbox_backlog_v1")
        return cur.fetchone()

