from svc.db import Reports


def run():
    Reports().active_users(None)


def test_run():
    run()
