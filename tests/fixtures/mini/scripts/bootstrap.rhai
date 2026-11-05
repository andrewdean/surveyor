// Bring a development database up: migrate, then seed through the api binary.
proc::shell(["mise", "run", "db:migrate"], #{});

fn seed(ctx) {
    cargo_run(ctx.repo, ctx.vars, "Cargo.toml", "api", ["seed"], #{});
}

seed(#{ repo: ".", vars: #{} });
