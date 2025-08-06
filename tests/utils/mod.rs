pub mod chain;
pub mod helpers;

pub const TEST_DATA_DIR: &str = "test-data";
pub const INTEGRATION_DATA_DIR: &str = "integration";
pub const STRESS_DATA_DIR: &str = "stress";

pub const ELECTRUM_1_REGTEST_URL: &str = "127.0.0.1:50001";
pub const ELECTRUM_2_REGTEST_URL: &str = "127.0.0.1:50002";
pub const ELECTRUM_3_REGTEST_URL: &str = "127.0.0.1:50003";
pub const ELECTRUM_MAINNET_URL: &str = "ssl://electrum.iriswallet.com:50003";
pub const ESPLORA_1_REGTEST_URL: &str = "http://127.0.0.1:8094/regtest/api";
pub const ESPLORA_2_REGTEST_URL: &str = "http://127.0.0.1:8095/regtest/api";
pub const ESPLORA_3_REGTEST_URL: &str = "http://127.0.0.1:8096/regtest/api";
pub const ESPLORA_MAINNET_URL: &str = "https://blockstream.info/api";
pub const FAKE_TXID: &str = "e5a3e577309df31bd606f48049049d2e1e02b048206ba232944fcc053a176ccb:0";
pub const UDA_FIXED_INDEX: u32 = 0;
pub const DEFAULT_FEE_ABS: u64 = 400;
pub const MEDIA_FPATH: &str = "tests/fixtures/rgb_logo.jpeg";
pub const REJECT_LIST_URL: &str = "example.xyz/rejectList";

pub const INSTANCE_1: u8 = 1;
pub const INSTANCE_2: u8 = 2;
pub const INSTANCE_3: u8 = 3;

pub type TT = TransferType;
pub type DT = DescriptorType;
pub type AS = AssetSchema;

pub use std::{
    cell::OnceCell,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    env::VarError,
    ffi::OsString,
    fmt::{self, Display},
    fs::OpenOptions,
    io::Write,
    num::NonZeroU32,
    path::{PathBuf, MAIN_SEPARATOR},
    process::{Command, Stdio},
    str::FromStr,
    sync::{Mutex, Once, OnceLock, RwLock},
    time::{Duration, Instant},
};

pub use amplify::{
    bmap, bset,
    confinement::{
        Collection, Confined, LargeVec, NonEmptyOrdMap, NonEmptyOrdSet, NonEmptyVec, SmallOrdMap,
        TinyOrdMap, TinyOrdSet, U16,
    },
    hex::FromHex,
    map, none,
    num::u24,
    s, set, ByteArray, Bytes64, From, Wrapper,
};
use bitcoin_hashes::{sha256, Hash};
pub use bp::{
    dbc::tapret::{TapretCommitment, TapretProof},
    seals::txout::TxPtr,
    seals::txout::{BlindSeal, CloseMethod, ExplicitSeal},
    secp256k1::{Message, Secp256k1, SecretKey},
    CompressedPk, ConsensusDecode, InternalPk, LockTime, Outpoint, Sats, ScriptPubkey, SeqNo, Tx,
    TxVer, Txid, Vout,
};
pub use bpstd::{
    h, signers::TestnetSigner, Address, DerivationPath, DerivationSeg, Derive, DerivedAddr,
    Descriptor, HardenedIndex, IdxBase, Keychain, Network, NormalIndex, Terminal, XkeyOrigin,
    Xpriv, XprivAccount, Xpub, XpubAccount, XpubDerivable, XpubFp,
};
pub use bpwallet::{
    fs::FsTextStore, indexers::esplora::Client as EsploraClient, AnyIndexer, Indexer as BpIndexer,
    Wallet, WalletUtxo,
};
pub use chrono::Utc;
pub use commit_verify::mpc;
pub use descriptors::Wpkh;
pub use electrum::{Client as ElectrumClient, ElectrumApi, Param};
pub use file_format::FileFormat;
pub use once_cell::sync::Lazy;
pub use psbt::{
    Beneficiary as PsbtBeneficiary, KeyMap, Payment, Prevout, PropKey, Psbt, PsbtConstructor,
    PsbtMeta, PsbtVer, Utxo,
};
#[cfg(not(feature = "altered"))]
pub use psrgbt::{OpoutAndOpids, ProprietaryKeyRgb, RgbExt, RgbPsbt, TxParams};
#[cfg(feature = "altered")]
pub use psrgbt::{OpoutAndOpids, ProprietaryKeyRgb, RgbExt, RgbPsbt, TxParams};
pub use rand::RngCore;
#[cfg(not(feature = "altered"))]
pub use rgb::{
    assignments::AssignVec,
    containers::{PubWitness, ValidContract, WitnessBundle},
    contract::{AllocatedState, AssignmentsFilter, ContractOp, FilterIncludeAll, OpDirection},
    info::ContractInfo,
    invoice::Pay2Vout,
    persistence::{MemContract, MemContractState, Stock},
    stl::{rgb_contract_stl, ContractTerms, RejectListUrl, StandardTypes},
    validation::{
        DbcProof, Failure, ResolveWitness, Scripts, Validator, Validity, Warning,
        WitnessOrdProvider, WitnessResolverError, WitnessStatus,
    },
    vm::{WitnessOrd, WitnessPos},
    Assign, AssignmentDetails, AssignmentType, BundleId, DescriptorRgb, FungibleState, GenesisSeal,
    GlobalDetails, GlobalStateSchema, GraphSeal, Identity, KnownTransition, MetaDetails, MetaType,
    MetaValue, Occurrences, OccurrencesMismatch, OpId, Opout, OwnedStateSchema, RevealedData,
    RevealedValue, RgbDescr, RgbKeychain, RgbWallet, StateType, TapretKey, TransferParams,
    Transition, TransitionBundle, TransitionType, TypedAssigns, Vin, VoidState, WalletProvider,
};
#[cfg(feature = "altered")]
pub use rgb_altered::{
    assignments::AssignVec,
    containers::{PubWitness, ValidContract, WitnessBundle},
    contract::{AllocatedState, AssignmentsFilter, ContractOp, FilterIncludeAll, OpDirection},
    info::ContractInfo,
    invoice::Pay2Vout,
    persistence::{MemContract, MemContractState, Stock},
    stl::{rgb_contract_stl, ContractTerms, RejectListUrl, StandardTypes},
    validation::{
        DbcProof, Failure, ResolveWitness, Scripts, Validator, Validity, Warning,
        WitnessResolverError, WitnessStatus,
    },
    vm::{WitnessOrd, WitnessPos},
    Assign, AssignmentDetails, AssignmentType, BundleId, DescriptorRgb, FungibleState, GenesisSeal,
    GlobalDetails, GlobalStateSchema, GraphSeal, Identity, KnownTransition, MetaDetails, MetaType,
    MetaValue, Occurrences, OccurrencesMismatch, OpId, Opout, OwnedStateSchema, RevealedData,
    RevealedValue, RgbDescr, RgbKeychain, RgbWallet, StateType, TapretKey, TransferParams,
    Transition, TransitionBundle, TransitionType, TypedAssigns, VoidState, WalletProvider,
};
pub use rgbstd::{
    containers::{
        BuilderSeal, ConsignmentExt, Fascia, FileContent, IndexedConsignment, Kit, Transfer,
        ValidKit,
    },
    contract::{
        ContractBuilder, ContractData, DataAllocation, FilterExclude, FungibleAllocation,
        IssuerWrapper, TransitionBuilder,
    },
    indexers::AnyResolver,
    invoice::{Beneficiary, RgbInvoice, RgbInvoiceBuilder, XChainNet},
    persistence::{fs::FsBinStore, StashReadProvider},
    schema::SchemaId,
    stl::{
        AssetSpec, Attachment, Details, EmbeddedMedia, MediaType, Name, ProofOfReserves,
        RicardianContract, Ticker, TokenData,
    },
    Allocation, Amount, ChainNet, ContractId, GlobalStateType, KnownState, Layer1, Operation,
    OutputAssignment, OutputSeal, OwnedFraction, Precision, Schema, SecretSeal, TokenIndex,
    TxoSeal,
};
pub use rstest::rstest;
pub use schemata::{
    CollectibleFungibleAsset, InflatableFungibleAsset, NonInflatableAsset,
    PermissionedFungibleAsset, UniqueDigitalAsset, CFA_SCHEMA_ID, ERRNO_INFLATION_MISMATCH,
    ERRNO_ISSUED_MISMATCH, ERRNO_NON_EQUAL_IN_OUT, ERRNO_REPLACE_HIDDEN_BURN,
    ERRNO_REPLACE_NO_INPUT, GS_ISSUED_SUPPLY, IFA_SCHEMA_ID, MS_ALLOWED_INFLATION, NIA_SCHEMA_ID,
    OS_ASSET, OS_INFLATION, OS_REPLACE, PFA_SCHEMA_ID, TS_BURN, TS_INFLATION, TS_REPLACE,
    TS_TRANSFER, UDA_SCHEMA_ID,
};
pub use serde_json::Value;
pub use serial_test::serial;
pub use strict_encoding::{fname, tn, FieldName, StrictSerialize, TypeName};
pub use strict_types::{SemId, StrictDumb, StrictVal, TypeSystem};
pub use strum::IntoEnumIterator;
pub use strum_macros::EnumIter;
pub use time::OffsetDateTime;

pub use crate::utils::{chain::*, helpers::*};
