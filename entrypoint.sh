#!/usr/bin/env bash

# shellcheck disable=SC2329
_term() {
    echo "termination request received..."
    if [ -n "$CHILD_PID" ]; then
        STRESS_PID=$(pgrep stress)
        if [ -z "$STRESS_PID" ]; then
            echo "failed to get test PID, aborting"
            exit 1
        fi
        kill -TERM "$CHILD_PID"
        wait "$CHILD_PID"
        unset CHILD_PID
        while ps "$STRESS_PID" >/dev/null; do
            echo -n "."
            sleep .2
        done
    else
        echo "cannot terminate, CHILD_PID not set"
    fi
}
trap _term SIGTERM SIGINT

# match internal with external user/group IDs
if [ "$MY_UID" != 1000 ]; then
    usermod -u "$MY_UID" "$USR"
fi
if [ "$MY_GID" != 1000 ]; then
    usermod -g "$MY_GID" "$USR"
fi
chown -Rf "$USR" "$WORKDIR"
chgrp -Rf "$(id -g "$USR")" /usr/local/cargo/registry
chmod g+w /usr/local/cargo/registry

# match internal with external "docker" group ID
DOCKER_GID=$(stat -c '%g' /var/run/docker.sock)
groupmod -g "$DOCKER_GID" docker

# start the test
CMD_BEGIN=(cargo test --profile reldebug --test stress random_transfers)
CMD_END=(--nocapture --ignored)

if [ -n "$MEMPROF" ]; then
    gosu "$USR" "${CMD_BEGIN[@]}" --features memprof -- "${CMD_END[@]}" &
    CHILD_PID=$!
else
    gosu "$USR" "${CMD_BEGIN[@]}" -- "${CMD_END[@]}" &
    CHILD_PID=$!
fi

wait $CHILD_PID
exit $?
