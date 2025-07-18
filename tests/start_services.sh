#!/bin/bash
set -u

_die () {
    echo "ERR: $*" >&2
    exit 1
}

_prepare_bitcoin_nodes() {
    $BCLI_1 createwallet miner
    $BCLI_2 createwallet miner
    $BCLI_3 createwallet miner
    $BCLI_1 -rpcwallet=miner -generate 103
    $BCLI_2 -rpcwallet=miner -generate 103
    # connect the 2 bitcoin services for the reorg
    if [ "$PROFILE" == "esplora" ]; then
        $BCLI_2 addnode "esplora_3:18444" "onetry"
        $BCLI_3 addnode "esplora_2:18444" "onetry"
    elif [ "$PROFILE" == "electrum" ]; then
        $BCLI_2 addnode "bitcoind_3:18444" "onetry"
        $BCLI_3 addnode "bitcoind_2:18444" "onetry"
    fi
}

_wait_for_bitcoind() {
    # wait for bitcoind to be up
    bitcoind_service_name="$1"
    start_time=$(date +%s)
    until $COMPOSE logs $bitcoind_service_name |grep -q 'Bound to'; do
        current_time=$(date +%s)
        if [ $((current_time - start_time)) -gt $TIMEOUT ]; then
            echo "Timeout waiting for $bitcoind_service_name to start"
            $COMPOSE logs $bitcoind_service_name
            exit 1
        fi
        sleep 1
    done
}

_wait_for_electrs() {
    # wait for electrs to have completed startup
    electrs_service_name="$1"
    start_time=$(date +%s)
    until $COMPOSE logs $electrs_service_name |grep -q 'finished full compaction'; do
        current_time=$(date +%s)
        if [ $((current_time - start_time)) -gt $TIMEOUT ]; then
            echo "Timeout waiting for $electrs_service_name to start"
            $COMPOSE logs $electrs_service_name
            exit 1
        fi
        sleep 1
    done
}

_wait_for_esplora() {
    # wait for esplora to have completed startup
    esplora_service_name="$1"
    start_time=$(date +%s)
    until $COMPOSE logs $esplora_service_name |grep -q 'run: nginx:'; do
        current_time=$(date +%s)
        if [ $((current_time - start_time)) -gt $TIMEOUT ]; then
            echo "Timeout waiting for $esplora_service_name to start"
            $COMPOSE logs $esplora_service_name
            exit 1
        fi
        sleep 1
    done
}

_stop_esplora_tor() {
    esplora_service_name="$1"
    if $COMPOSE ps |grep -q $esplora_service_name; then
        for SRV in socat tor; do
            $COMPOSE exec $esplora_service_name bash -c "sv -w 60 force-stop /etc/service/$SRV"
        done
    fi
}

_stop_services() {
    # bring all services down
    $COMPOSE --profile '*' down -v --remove-orphans
}

_start_services() {
    _stop_services
    mkdir -p $TEST_DATA_DIR
    for port in "${EXPOSED_PORTS[@]}"; do
        if [ -n "$(ss -HOlnt "sport = :$port")" ];then
            _die "port $port is already bound, services can't be started"
        fi
    done
    $COMPOSE --profile "$PROFILE" up -d
}

COMPOSE="docker compose"
if ! $COMPOSE >/dev/null; then
    _die "could not call docker compose (hint: install docker compose plugin)"
fi
COMPOSE="$COMPOSE -f tests/compose.yaml"
PROFILE=${PROFILE:-"esplora"}
TEST_DATA_DIR="./test-data"
TIMEOUT=100

# see compose.yaml for the exposed ports
if [ "$PROFILE" == "esplora" ]; then
    BCLI_1="$COMPOSE exec -T esplora_1 cli"
    BCLI_2="$COMPOSE exec -T esplora_2 cli"
    BCLI_3="$COMPOSE exec -T esplora_3 cli"
    EXPOSED_PORTS=(8094 8095 8096 50004 50005 50006)
elif [ "$PROFILE" == "electrum" ]; then
    BCLI_1="$COMPOSE exec -T -u blits bitcoind_1 bitcoin-cli -regtest"
    BCLI_2="$COMPOSE exec -T -u blits bitcoind_2 bitcoin-cli -regtest"
    BCLI_3="$COMPOSE exec -T -u blits bitcoind_3 bitcoin-cli -regtest"
    EXPOSED_PORTS=(50001 50002 50003)
else
    _die "invalid profile"
fi

# restart services (down + up) checking for ports availability
_start_services

# wait for services (pre-mining)
if [ "$PROFILE" == "esplora" ]; then
    _wait_for_esplora esplora_1
    _wait_for_esplora esplora_2
    _wait_for_esplora esplora_3
    _stop_esplora_tor esplora_1
    _stop_esplora_tor esplora_2
    _stop_esplora_tor esplora_3
elif [ "$PROFILE" == "electrum" ]; then
    _wait_for_bitcoind bitcoind_1
    _wait_for_bitcoind bitcoind_2
    _wait_for_bitcoind bitcoind_3
fi

_prepare_bitcoin_nodes

# wait for services (post-mining)
if [ "$PROFILE" == "electrum" ]; then
    _wait_for_electrs electrs_1
    _wait_for_electrs electrs_2
    _wait_for_electrs electrs_3
fi
