pub mod utils;

use rstest_reuse::{self, *};
use utils::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ChainNetScenario {
    Regtest,
    Mainnet,
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
        ChainNetScenario::SignetCustom => {
            let genesis_hash = match get_indexer_client(&url) {
                IndexerClient::Electrum(client) => client.block_header(0).unwrap().block_hash(),
                IndexerClient::Esplora(client) => {
                    client.get_block_hash(0).expect("genesis block hash")
                }
            };
            ChainNet::BitcoinSignetCustom(ChainHash::from_genesis_block_hash(genesis_hash))
        }
    };

    (url, chain_net)
}

#[template]
#[rstest]
#[case(ChainNetScenario::Regtest)]
#[case(ChainNetScenario::Mainnet)]
#[case(ChainNetScenario::Testnet3)]
#[case(ChainNetScenario::Testnet4)]
#[case(ChainNetScenario::SignetCustom)]
fn chain_net_scenario(#[case] scenario: ChainNetScenario) {}

#[cfg(not(feature = "altered"))]
#[apply(chain_net_scenario)]
fn check_chain_net(scenario: ChainNetScenario) {
    println!("scenario {scenario:?}");

    initialize();

    if scenario == ChainNetScenario::Testnet4 && *INDEXER.get().unwrap() == Indexer::Esplora {
        // no public testnet4 esplora indexer yet
        return;
    }

    let (url, chain_net) = get_chain_net_config(&scenario);

    let resolver = get_resolver(&url);
    resolver
        .check_chain_net(chain_net)
        .expect("chain net match");
}
