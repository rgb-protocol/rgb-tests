pub mod chain;
pub mod wallet;

pub const TEST_DATA_DIR: &str = "test-data";
pub const INTEGRATION_DATA_DIR: &str = "integration";
pub const STRESS_DATA_DIR: &str = "stress";
pub const SAVE_DIR: &str = "saves";

pub const ELECTRUM_MAINNET_URL: &str = "ssl://electrum.iriswallet.com:50003";
pub const ESPLORA_MAINNET_URL: &str = "https://blockstream.info/api";
pub const ELECTRUM_TESTNET3_URL: &str = "ssl://electrum.iriswallet.com:50013";
pub const ESPLORA_TESTNET3_URL: &str = "https://blockstream.info/testnet/api";
pub const ELECTRUM_TESTNET4_URL: &str = "ssl://electrum.iriswallet.com:50053";
pub const ESPLORA_SIGNET_URL: &str = "https://blockstream.info/signet/api/";
pub const ELECTRUM_SIGNET_URL: &str = "ssl://mempool.space:60602"; // no verbose support
pub const FAKE_TXID: &str = "e5a3e577309df31bd606f48049049d2e1e02b048206ba232944fcc053a176ccb:0";
pub const UDA_FIXED_INDEX: u32 = 0;
pub const DEFAULT_FEE_ABS: u64 = 400;
pub const MEDIA_FPATH: &str = "tests/fixtures/rgb_logo.jpeg";
pub const REJECT_LIST_URL: &str = "example.xyz/rejectList";
pub const PURPOSE_BIP84: u32 = 84;
pub const PURPOSE_BIP86: u32 = 86;
pub const COIN_RGB_TESTNET: u32 = 827167;
pub const ACCOUNT: u32 = 0;

pub const INSTANCE_1: u8 = 1;
pub const INSTANCE_2: u8 = 2;
pub const INSTANCE_3: u8 = 3;

pub type TT = TransferType;
pub type DT = DescriptorType;
pub type AS = AssetSchema;

#[cfg(not(target_os = "windows"))]
pub use std::os::unix::process::CommandExt;
pub use std::{
    borrow::Borrow,
    cell::{OnceCell, RefCell},
    cmp::max,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env::VarError,
    ffi::OsString,
    fmt::{self, Display},
    fs::OpenOptions,
    io::Write,
    num::NonZeroU32,
    path::{MAIN_SEPARATOR, PathBuf},
    process::{Command, Stdio},
    str::FromStr,
    sync::Arc,
    sync::atomic::{AtomicBool, AtomicU32, Ordering},
    sync::{Mutex, Once, OnceLock, RwLock},
    time::{Duration, Instant},
};

pub use aluvm::{
    isa::Instr,
    library::{Lib, LibSeg, LibSite},
};
pub use amplify::{
    ByteArray, Bytes64, From, Wrapper, bmap, bset, confinement,
    confinement::{
        Collection, Confined, LargeVec, NonEmptyOrdMap, NonEmptyOrdSet, NonEmptyVec, SmallBlob,
        SmallOrdMap, TinyOrdMap, TinyOrdSet, U16, U32,
    },
    hex::FromHex,
    map, none,
    num::u24,
    s, set, tiny_bmap, zero,
};
pub use bdk_electrum::{
    BdkElectrumClient,
    electrum_client::{Client as ElectrumClient, ElectrumApi as _},
};
pub use bdk_esplora::{
    EsploraExt,
    esplora_client::{BlockingClient as EsploraClient, Builder as EsploraBuilder},
};
pub use bdk_wallet::{
    ChangeSet, KeychainKind, PersistedWallet, SignOptions, Update, Wallet as BdkWallet,
    bitcoin::{
        bip32::Xpriv,
        psbt::{Output as BdkOutput, raw::ProprietaryKey},
    },
    chain::{
        ChainPosition,
        spk_client::{FullScanRequest, FullScanResponse},
    },
    file_store::Store,
};
pub use bitcoin_hashes::sha256;
pub use bpwallet::{
    Address as BpAddress, AnyIndexer, ConsensusDecode, DerivationPath, DerivationSeg, Derive,
    DerivedAddr, Descriptor, HardenedIndex, Idx, IdxBase, Indexer as BpIndexer, InternalPk,
    Keychain, LockTime as BpLockTime, Network as BpNetwork, NormalIndex, Outpoint as BpOutpoint,
    Sats, ScriptPubkey, SeqNo, Terminal as BpTerminal, Tx, TxStatus, TxVer, Txid as BpTxid,
    Vout as BpVout, Wallet as BpWallet, WalletUtxo, Wpkh, XkeyOrigin, XprivAccount, Xpub,
    XpubAccount, XpubDerivable, XpubFp,
    fs::FsTextStore,
    h,
    indexers::esplora::Client as BpEsploraClient,
    psbt::{
        Beneficiary as PsbtBeneficiary, KeyMap, Output, Payment, Prevout, PropKey, Psbt as BpPsbt,
        PsbtConstructor, PsbtMeta as BpPsbtMeta, PsbtVer, Utxo,
    },
    signers::TestnetSigner,
};
pub use chrono::Utc;
pub use electrum::{Client as BpElectrumClient, ElectrumApi as _, Param};
pub use file_format::FileFormat;
pub use lazy_static::lazy_static;
#[cfg(not(target_os = "windows"))]
pub use nix::unistd::{self, Pid};
pub use once_cell::sync::Lazy;
pub use psrgbt::{
    OpoutAndOpids, RgbOutExt, RgbPropKeyExt, RgbPsbtExt, Terminal,
    bp_conversion_utils::{
        address_bitcoin_to_bp, address_bp_to_bitcoin, address_payload_bitcoin_from_script_pubkey,
        address_payload_bp_from_script_pubkey, internal_pk_to_untweakedpublickey,
        network_bp_to_bitcoin, outpoint_bitcoin_to_bp, outpoint_bp_to_bitcoin,
        script_buf_to_script_pubkey, tx_bitcoin_to_bp, tx_bp_to_bitcoin, txid_bitcoin_to_bp,
        txid_bp_to_bitcoin, untweakedpublickey_to_internal_pk,
    },
};
pub use rand::{Rng, RngCore, SeedableRng, rngs::StdRng, seq::SliceRandom};
pub use rgb::{
    Anchor, Assign, AssignFungible, AssignmentDetails, AssignmentType, BundleId, DescriptorRgb,
    FungibleState, FungibleType, GenesisSchema, GenesisSeal, GlobalDetails, GlobalStateSchema,
    GraphSeal, Identity, KnownTransition, MetaDetails, MetaType, MetaValue, Occurrences,
    OccurrencesMismatch, OpFullType, OpId, Opout, Outpoint, OwnedStateSchema, RevealedData,
    RevealedValue, RgbDescr, RgbWallet, SqliteRgbWallet, StateType, TapretKey, TransferParams,
    Transition, TransitionBundle, TransitionDetails, TransitionSchema, TransitionType,
    TypedAssigns, Vin, VoidState, WalletProvider, WpkhDescr,
    assignments::AssignVec,
    bitcoin::{self, Sequence, TxIn, TxOut, Witness, absolute::LockTime},
    bitcoin::{
        Address, BlockHash, CompressedPublicKey, Network, Psbt, ScriptBuf, TapLeafHash,
        TapNodeHash, Transaction, block::Header as BlockHeader, constants::ChainHash,
        hashes::sha256d, key::Secp256k1 as BitcoinSecp256k1, taproot::LeafScript,
        taproot::LeafVersion, transaction::Version,
    },
    containers::{ConsignmentVer, TerminalSeals, ValidContract, WitnessBundle, legacy::TransferV0},
    contract::{
        AllocatedState, AssignmentsFilter, ContractOp, FilterIncludeAll, FilteredContractState,
        OpDirection, SchemaWrapper,
    },
    info::ContractInfo,
    invoice::{AddressPayload, Pay2Vout},
    opret::OpretProof,
    pay::{PsbtMeta, TxParams},
    persistence::{ContractAssignments, Stock, StockError},
    rgbasm,
    stl::{ContractTerms, RejectListUrl, StandardTypes, rgb_contract_stl},
    tapret::{TapretNodePartner, TapretRightBranch},
    validation::{
        DbcProof, Failure, OpoutsDagData, ResolveWitness, SchemaDefError, SchemaDefinition,
        SchemaRules, Scripts, SpvProof, Status, TypeLibs, ValidationConfig, ValidationError,
        Validator, Validity, Warning, WitnessOrdProvider, WitnessResolverError, WitnessStatus,
    },
    vm::{
        ContractStateAccess, ContractStateEvolve, GlobalStateEntry, GlobalsIter, RgbIsa,
        UnknownGlobalStateType, WitnessOrd, WitnessPos,
    },
};
pub use rgbcore::{
    Txid, Vout,
    commit_verify::mpc,
    dbc::tapret::{TapretCommitment, TapretProof},
    seals::txout::TxPtr,
    seals::txout::{BlindSeal, CloseMethod, ExplicitSeal},
    secp256k1::{Message, SecretKey, generate_keypair},
};
pub use rgbstd::{
    Allocation,
    Amount,
    ChainNet,
    ContractId,
    GlobalStateType,
    KnownState,
    Layer1,
    Operation,
    OutputAssignment,
    OutputSeal,
    OwnedFraction,
    Precision,
    Schema,
    SecretSeal,
    TokenIndex,
    TxoSeal,
    containers::{
        BuilderSeal, Consignment, ConsignmentConstraintError, ConsignmentExt, Fascia, FileContent,
        Transfer, UncheckedTransfer,
    },
    contract::{
        ContractBuilder, ContractData, ContractStateRead, DataAllocation, FilterExclude,
        FungibleAllocation, IssuerWrapper, LinkableSchemaWrapper, TransitionBuilder,
    },
    daggy::Walker,
    indexers::bitcoind_blocking::BitcoindClient,
    indexers::bitcoind_blocking::bitcoincore_rpc::{Auth as BitcoindAuth, Client as BitcoindRpc},
    // `Indexer` is aliased since `utils::chain` has an enum by that name
    indexers::{AnyResolver, ResolveSpvProof},
    invoice::{Beneficiary, RgbInvoice, RgbInvoiceBuilder, XChainNet},
    persistence::{
        ContractStateSnapshot, RgbStore,
        sqlite::{SqliteStock, SqliteStore},
    },
    schema::SchemaId,
    stl::{
        AssetSpec, Attachment, Details, EmbeddedMedia, MediaType, Name, ProofOfReserves,
        RicardianContract, Ticker, TokenData,
    },
};
pub use rstest::rstest;
pub use schemata::{
    CFA_SCHEMA_ID, CollectibleFungibleAsset, ERRNO_BURN_MISMATCH, ERRNO_BURN_ZERO,
    ERRNO_INFLATION_MISMATCH, ERRNO_ISSUED_MISMATCH, ERRNO_NON_EQUAL_IN_OUT, GS_ISSUED_SUPPLY,
    IFA_SCHEMA_ID, IfaWrapper, InflatableFungibleAsset, MS_ALLOWED_INFLATION, NIA_SCHEMA_ID,
    NonInflatableAsset, OS_ASSET, OS_INFLATION, OS_LINK, PFA_SCHEMA_ID, PermissionedFungibleAsset,
    TS_BURN, TS_INFLATION, TS_TRANSFER, UDA_SCHEMA_ID, UniqueDigitalAsset, burn_meta_by_assignment,
};
pub use serde::{Deserialize, Serialize};
pub use serde_json::{Value, json};
pub use serial_test::serial;
pub use signal_hook::consts::{SIGINT, SIGTERM};
pub use signal_hook::flag::register;
pub use strict_encoding::{FieldName, StrictSerialize, TypeName, fname, strict_dumb, tn};
pub use strict_types::ast::{Field, NamedFields};
pub use strict_types::{
    LibRef, SemId, StrictDecode, StrictDeserialize, StrictDumb, StrictEncode, StrictType,
    StrictVal, Ty, TypeLibId, TypeSystem,
};
pub use strum::{EnumIter, IntoEnumIterator};
pub use time::OffsetDateTime;

pub use crate::utils::{chain::*, wallet::*};

fn running_in_docker() -> bool {
    std::path::Path::new("/.dockerenv").exists()
}

lazy_static! {
    pub static ref ELECTRUM_1_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "electrum_1:50001"
        } else {
            "127.0.0.1:50001"
        }
    };
    pub static ref ELECTRUM_2_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "electrum_2:50001"
        } else {
            "127.0.0.1:50002"
        }
    };
    pub static ref ELECTRUM_3_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "electrum_3:50001"
        } else {
            "127.0.0.1:50003"
        }
    };
    pub static ref ESPLORA_1_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "http://esplora_1:80/regtest/api"
        } else {
            "http://127.0.0.1:8094/regtest/api"
        }
    };
    pub static ref ESPLORA_2_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "http://esplora_2:80/regtest/api"
        } else {
            "http://127.0.0.1:8095/regtest/api"
        }
    };
    pub static ref ESPLORA_3_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "http://esplora_3:80/regtest/api"
        } else {
            "http://127.0.0.1:8096/regtest/api"
        }
    };
    pub static ref BITCOIND_1_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "http://bitcoind_1:18443"
        } else {
            "http://127.0.0.1:18443"
        }
    };
    pub static ref BITCOIND_2_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "http://bitcoind_2:18443"
        } else {
            "http://127.0.0.1:18453"
        }
    };
    pub static ref BITCOIND_3_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "http://bitcoind_3:18443"
        } else {
            "http://127.0.0.1:18463"
        }
    };
    pub static ref BITCOIND_NO_TXINDEX_REGTEST_URL: &'static str = {
        if running_in_docker() {
            "http://bitcoind_no_txindex:18443"
        } else {
            "http://127.0.0.1:18473"
        }
    };
    pub static ref ELECTRUM_SIGNET_CUSTOM_URL: &'static str = {
        if running_in_docker() {
            "electrs_signet_custom:50001"
        } else {
            "127.0.0.1:50007"
        }
    };
    pub static ref ESPLORA_SIGNET_CUSTOM_URL: &'static str = {
        if running_in_docker() {
            "http://esplora_signet_custom:80/api"
        } else {
            "http://127.0.0.1:8097/signet/api"
        }
    };
}
