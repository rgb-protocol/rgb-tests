pub mod utils;

use rstest_reuse::{self, *};
use utils::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ChainNetScenario {
    Regtest,
    Mainnet,
    Signet,
    SignetCustom,
    Testnet3,
    Testnet4,
}

fn get_chain_net_config(scenario: &ChainNetScenario) -> (String, ChainNet) {
    let url = match (INDEXER.get().unwrap(), scenario) {
        (Indexer::Electrum, ChainNetScenario::Regtest) => ELECTRUM_1_REGTEST_URL.to_string(),
        (Indexer::Esplora, ChainNetScenario::Regtest) => ESPLORA_1_REGTEST_URL.to_string(),
        (Indexer::Electrum, ChainNetScenario::Mainnet) => ELECTRUM_MAINNET_URL.to_string(),
        (Indexer::Esplora, ChainNetScenario::Mainnet) => ESPLORA_MAINNET_URL.to_string(),
        (Indexer::Electrum, ChainNetScenario::Signet) => ELECTRUM_SIGNET_URL.to_string(),
        (Indexer::Esplora, ChainNetScenario::Signet) => ESPLORA_SIGNET_URL.to_string(),
        (Indexer::Electrum, ChainNetScenario::SignetCustom) => {
            ELECTRUM_SIGNET_CUSTOM_URL.to_string()
        }
        (Indexer::Esplora, ChainNetScenario::SignetCustom) => ESPLORA_SIGNET_CUSTOM_URL.to_string(),
        (Indexer::Electrum, ChainNetScenario::Testnet3) => ELECTRUM_TESTNET3_URL.to_string(),
        (Indexer::Esplora, ChainNetScenario::Testnet3) => ESPLORA_TESTNET3_URL.to_string(),
        (Indexer::Electrum, ChainNetScenario::Testnet4) => ELECTRUM_TESTNET4_URL.to_string(),
        (Indexer::Esplora, ChainNetScenario::Testnet4) => unreachable!(),
    };

    let chain_net = match scenario {
        ChainNetScenario::Regtest => ChainNet::BitcoinRegtest,
        ChainNetScenario::Mainnet => ChainNet::BitcoinMainnet,
        ChainNetScenario::Testnet3 => ChainNet::BitcoinTestnet3,
        ChainNetScenario::Testnet4 => ChainNet::BitcoinTestnet4,
        ChainNetScenario::Signet => ChainNet::BitcoinSignet,
        ChainNetScenario::SignetCustom => ChainNet::BitcoinSignetCustom,
    };

    (url, chain_net)
}

#[template]
#[rstest]
#[case(ChainNetScenario::Regtest)]
#[case(ChainNetScenario::Mainnet)]
#[case(ChainNetScenario::Testnet3)]
#[case(ChainNetScenario::Testnet4)]
#[case(ChainNetScenario::Signet)]
#[case(ChainNetScenario::SignetCustom)]
fn chain_net_scenario(#[case] scenario: ChainNetScenario) {}

#[apply(chain_net_scenario)]
fn check_chain_net(scenario: ChainNetScenario) {
    println!("scenario {scenario:?}");

    initialize();

    match (&scenario, INDEXER.get().unwrap()) {
        // no public testnet4 esplora indexer yet
        (ChainNetScenario::Testnet4, Indexer::Esplora) => return,
        // no public signet electrum indexer with verbose support yet
        (ChainNetScenario::Signet, Indexer::Electrum) => return,
        _ => {}
    }

    let (url, chain_net) = get_chain_net_config(&scenario);

    let resolver = get_resolver(&url);
    resolver
        .check_chain_net(chain_net)
        .expect("chain net match");
}

#[rstest]
#[case(*ELECTRUM_SIGNET_CUSTOM_URL, WitnessResolverError::WrongChainNet)]
#[case(ELECTRUM_SIGNET_URL, WitnessResolverError::ResolverIssue(None, "verbose transactions are unsupported by the provided electrum service".to_string()))]
fn check_chain_net_failures(#[case] url: &str, #[case] expected_err: WitnessResolverError) {
    initialize();

    if *INDEXER.get().unwrap() != Indexer::Electrum {
        return;
    }

    let resolver = get_resolver(url);
    let result = resolver.check_chain_net(ChainNet::BitcoinSignet);
    assert!(matches!(result, Err(err) if err == expected_err));
}

/// A Bitcoin Core node with no transaction index is on the right chain like any other, so
/// `check_chain_net` accepts it: it can still serve block headers, and the operations it
/// cannot serve reject it on their own, see `bitcoind_no_txindex_is_spv_only`.
#[test]
fn check_chain_net_bitcoind_no_txindex() {
    initialize();

    // the bitcoind RPC is only exposed under the electrum profile
    if *INDEXER.get().unwrap() != Indexer::Electrum {
        return;
    }

    // the node with a transaction index is accepted
    get_bitcoind_resolver(INSTANCE_1)
        .check_chain_net(ChainNet::BitcoinRegtest)
        .unwrap();

    // and so is the one without it
    get_bitcoind_no_txindex_resolver()
        .check_chain_net(ChainNet::BitcoinRegtest)
        .unwrap();

    // telling the two apart is what `has_synced_txindex` is for
    assert!(
        get_bitcoind_client(&bitcoind_url(INSTANCE_1))
            .has_synced_txindex()
            .unwrap()
    );
    assert!(
        !get_bitcoind_client(&BITCOIND_NO_TXINDEX_REGTEST_URL)
            .has_synced_txindex()
            .unwrap()
    );
}
