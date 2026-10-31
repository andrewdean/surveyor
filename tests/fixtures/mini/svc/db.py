"""Database access for the reporting service."""


class Reports:
    """Runs reporting queries."""

    def active_users(self, cur):
        cur.execute("SELECT count(*) FROM cw_users.users WHERE active")
        return cur.fetchone()
