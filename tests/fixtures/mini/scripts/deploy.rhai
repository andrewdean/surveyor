import "host/k8s" as k8s;

fn deploy(env) {
    k8s::apply(env, "deploy/outbox.yaml");
    notify(env);
}

deploy("prod");
