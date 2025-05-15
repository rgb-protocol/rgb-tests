use super::*;

pub struct TestWallet {
    wallet: RgbWallet<Wallet<XpubDerivable, RgbDescr>>,
    signer: Option<TestnetSigner>,
    wallet_dir: PathBuf,
    instance: u8,
}

enum WalletAccount {
    Private(XprivAccount),
    Public(XpubAccount),
}

pub enum AllocationFilter {
    Stock,
    Wallet,
    WalletAll,
    WalletTentative,
}

impl AllocationFilter {
    pub fn filter_for(self, wlt: &TestWallet) -> Filter {
        match self {
            Self::WalletAll => Filter::WalletAll(&wlt.wallet),
            Self::WalletTentative => Filter::WalletTentative(&wlt.wallet),
            Self::Wallet => Filter::Wallet(&wlt.wallet),
            Self::Stock => Filter::NoWallet,
        }
    }
}

pub enum Filter<'w> {
    NoWallet,
    Wallet(&'w RgbWallet<Wallet<XpubDerivable, RgbDescr>>),
    WalletAll(&'w RgbWallet<Wallet<XpubDerivable, RgbDescr>>),
    WalletTentative(&'w RgbWallet<Wallet<XpubDerivable, RgbDescr>>),
}

impl AssignmentsFilter for Filter<'_> {
    fn should_include(&self, outpoint: impl Into<Outpoint>, id: Option<Txid>) -> bool {
        match self {
            Filter::Wallet(wallet) => wallet
                .wallet()
                .filter_unspent()
                .should_include(outpoint, id),
            Filter::WalletTentative(wallet) => wallet
                .wallet()
                .filter_outpoints()
                .should_include(outpoint, id),
            _ => true,
        }
    }
}
impl Filter<'_> {
    fn comment(&self, outpoint: Outpoint) -> &'static str {
        match self {
            Filter::Wallet(rgb) if rgb.wallet().is_unspent(outpoint) => "",
            Filter::WalletAll(rgb) | Filter::WalletTentative(rgb)
                if rgb.wallet().is_unspent(outpoint) =>
            {
                "-- unspent"
            }
            Filter::WalletAll(rgb) | Filter::WalletTentative(rgb)
                if rgb.wallet().has_outpoint(outpoint) =>
            {
                "-- spent"
            }
            _ => "-- third-party",
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DescriptorType {
    Wpkh,
    Tr,
}

impl fmt::Display for DescriptorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Debug, Copy, Clone)]
pub enum HistoryType {
    Linear,
    Branching,
    Merging,
}

#[derive(Debug, Copy, Clone)]
pub enum ReorgType {
    ChangeOrder,
    Revert,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransferType {
    Blinded,
    Witness,
}

impl fmt::Display for TransferType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

pub enum InvoiceType {
    Blinded(Option<Outpoint>),
    Witness,
    WitnessTapret,
}

impl From<TransferType> for InvoiceType {
    fn from(transfer_type: TransferType) -> Self {
        match transfer_type {
            TransferType::Blinded => InvoiceType::Blinded(None),
            TransferType::Witness => InvoiceType::Witness,
        }
    }
}

/// RGB asset-specific information to color a transaction
#[derive(Clone, Debug)]
pub struct AssetColoringInfo {
    /// Input outpoints of the assets being spent
    pub input_outpoints: Vec<Outpoint>,
    /// Map of vouts and asset amounts to color the transaction outputs
    pub output_map: HashMap<u32, u64>,
    /// Static blinding to keep the transaction construction deterministic
    pub static_blinding: Option<u64>,
}

/// RGB information to color a transaction
#[derive(Clone, Debug)]
pub struct ColoringInfo {
    /// Asset-specific information
    pub asset_info_map: HashMap<ContractId, AssetColoringInfo>,
    /// Static blinding to keep the transaction construction deterministic
    pub static_blinding: Option<u64>,
    /// Nonce for offchain TXs ordering
    pub nonce: Option<u64>,
}

/// Map of contract ID and list of its beneficiaries
pub type AssetBeneficiariesMap = BTreeMap<ContractId, Vec<BuilderSeal<GraphSeal>>>;

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
pub enum AssetSchema {
    Nia,
    Uda,
    Cfa,
    Pfa,
    Ifa,
}

impl fmt::Display for AssetSchema {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

impl AssetSchema {
    fn schema(&self) -> Schema {
        match self {
            Self::Nia => NonInflatableAsset::schema(),
            Self::Uda => UniqueDigitalAsset::schema(),
            Self::Cfa => CollectibleFungibleAsset::schema(),
            Self::Pfa => PermissionedFungibleAsset::schema(),
            Self::Ifa => InflatableFungibleAsset::schema(),
        }
    }

    fn scripts(&self) -> Scripts {
        match self {
            Self::Nia => NonInflatableAsset::scripts(),
            Self::Uda => UniqueDigitalAsset::scripts(),
            Self::Cfa => CollectibleFungibleAsset::scripts(),
            Self::Pfa => PermissionedFungibleAsset::scripts(),
            Self::Ifa => InflatableFungibleAsset::scripts(),
        }
    }

    fn types(&self) -> TypeSystem {
        match self {
            Self::Nia => NonInflatableAsset::types(),
            Self::Uda => UniqueDigitalAsset::types(),
            Self::Cfa => CollectibleFungibleAsset::types(),
            Self::Pfa => PermissionedFungibleAsset::types(),
            Self::Ifa => InflatableFungibleAsset::types(),
        }
    }

    fn get_valid_kit(&self) -> ValidKit {
        let mut kit = Kit::default();
        kit.schemata.push(self.schema()).unwrap();
        kit.scripts.extend(self.scripts().into_values()).unwrap();
        kit.types = self.types();
        kit.validate().unwrap()
    }

    fn default_state_type(&self) -> StateType {
        match self {
            Self::Cfa | Self::Nia | Self::Pfa | Self::Ifa => StateType::Fungible,
            Self::Uda => StateType::Structured,
        }
    }

    fn allocated_state(&self, value: u64) -> AllocatedState {
        match self {
            Self::Cfa | Self::Nia | Self::Pfa | Self::Ifa => AllocatedState::Amount(value.into()),
            Self::Uda => AllocatedState::Data(
                Allocation::with(UDA_FIXED_INDEX, OwnedFraction::from(1)).into(),
            ),
        }
    }
}

impl From<SchemaId> for AssetSchema {
    fn from(schema_id: SchemaId) -> Self {
        match schema_id {
            CFA_SCHEMA_ID => AssetSchema::Cfa,
            NIA_SCHEMA_ID => AssetSchema::Nia,
            UDA_SCHEMA_ID => AssetSchema::Uda,
            PFA_SCHEMA_ID => AssetSchema::Pfa,
            IFA_SCHEMA_ID => AssetSchema::Ifa,
            _ => panic!("unknown schema ID"),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum AssetInfo {
    Nia {
        spec: AssetSpec,
        terms: ContractTerms,
        issue_amounts: Vec<u64>,
    },
    Uda {
        spec: AssetSpec,
        terms: ContractTerms,
        token_data: TokenData,
    },
    Cfa {
        name: Name,
        precision: Precision,
        details: Option<Details>,
        terms: ContractTerms,
        issue_amounts: Vec<u64>,
    },
    Pfa {
        spec: AssetSpec,
        terms: ContractTerms,
        issue_amounts: Vec<u64>,
        pubkey: CompressedPk,
    },
    Ifa {
        spec: AssetSpec,
        terms: ContractTerms,
        opid_reject_url: Option<OpidRejectUrl>,
        issue_amounts: Vec<u64>,
        replace_outpoints: Vec<Outpoint>,
        inflation_info: Vec<(Outpoint, u64)>,
    },
}

impl AssetInfo {
    pub fn asset_schema(&self) -> AssetSchema {
        match self {
            Self::Nia { .. } => AssetSchema::Nia,
            Self::Uda { .. } => AssetSchema::Uda,
            Self::Cfa { .. } => AssetSchema::Cfa,
            Self::Pfa { .. } => AssetSchema::Pfa,
            Self::Ifa { .. } => AssetSchema::Ifa,
        }
    }

    pub fn schema(&self) -> Schema {
        self.asset_schema().schema()
    }

    pub fn scripts(&self) -> Scripts {
        self.asset_schema().scripts()
    }

    pub fn types(&self) -> TypeSystem {
        self.asset_schema().types()
    }

    pub fn default_cfa(issue_amounts: Vec<u64>) -> Self {
        AssetInfo::cfa("CFA asset name", 0, None, "CFA terms", None, issue_amounts)
    }

    pub fn default_nia(issue_amounts: Vec<u64>) -> Self {
        AssetInfo::nia(
            "NIATCKR",
            "NIA asset name",
            2,
            None,
            "NIA terms",
            None,
            issue_amounts,
        )
    }

    pub fn default_pfa(issue_amounts: Vec<u64>, pubkey: CompressedPk) -> Self {
        AssetInfo::pfa(
            "PFATCKR",
            "PFA asset name",
            2,
            None,
            "PFA terms",
            None,
            issue_amounts,
            pubkey,
        )
    }

    pub fn default_ifa(
        issue_amounts: Vec<u64>,
        replace_outpoints: Vec<Outpoint>,
        inflation_info: Vec<(Outpoint, u64)>,
    ) -> Self {
        AssetInfo::ifa(
            "IFATCKR",
            "IFA asset name",
            0,
            None,
            "IFA terms",
            None,
            Some(OPID_REJECT_URL),
            issue_amounts,
            replace_outpoints,
            inflation_info,
        )
    }

    pub fn default_uda() -> Self {
        AssetInfo::uda(
            "UDATCKR",
            "UDA asset name",
            None,
            "NIA terms",
            None,
            uda_token_data_minimal(),
        )
    }

    pub fn nia(
        ticker: &str,
        name: &str,
        precision: u8,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        issue_amounts: Vec<u64>,
    ) -> Self {
        let spec = AssetSpec::with(
            ticker,
            name,
            Precision::try_from(precision).unwrap(),
            details,
        )
        .unwrap();
        let text = RicardianContract::from_str(terms_text).unwrap();
        let attachment = terms_media_fpath.map(attachment_from_fpath);
        let terms = ContractTerms {
            text,
            media: attachment,
        };
        Self::Nia {
            spec,
            terms,
            issue_amounts,
        }
    }

    pub fn uda(
        ticker: &str,
        name: &str,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        token_data: TokenData,
    ) -> AssetInfo {
        let spec = AssetSpec::with(ticker, name, Precision::try_from(0).unwrap(), details).unwrap();
        let text = RicardianContract::from_str(terms_text).unwrap();
        let attachment = terms_media_fpath.map(attachment_from_fpath);
        let terms = ContractTerms {
            text,
            media: attachment.clone(),
        };
        Self::Uda {
            spec,
            terms,
            token_data,
        }
    }

    pub fn cfa(
        name: &str,
        precision: u8,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        issue_amounts: Vec<u64>,
    ) -> AssetInfo {
        let text = RicardianContract::from_str(terms_text).unwrap();
        let attachment = terms_media_fpath.map(attachment_from_fpath);
        let terms = ContractTerms {
            text,
            media: attachment,
        };
        Self::Cfa {
            name: Name::try_from(name.to_owned()).unwrap(),
            precision: Precision::try_from(precision).unwrap(),
            details: details.map(|d| Details::try_from(d.to_owned()).unwrap()),
            terms,
            issue_amounts,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn pfa(
        ticker: &str,
        name: &str,
        precision: u8,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        issue_amounts: Vec<u64>,
        pubkey: CompressedPk,
    ) -> Self {
        let spec = AssetSpec::with(
            ticker,
            name,
            Precision::try_from(precision).unwrap(),
            details,
        )
        .unwrap();
        let text = RicardianContract::from_str(terms_text).unwrap();
        let attachment = terms_media_fpath.map(attachment_from_fpath);
        let terms = ContractTerms {
            text,
            media: attachment,
        };
        Self::Pfa {
            spec,
            terms,
            issue_amounts,
            pubkey,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ifa(
        ticker: &str,
        name: &str,
        precision: u8,
        details: Option<&str>,
        terms_text: &str,
        terms_media_fpath: Option<&str>,
        opid_reject_url: Option<&str>,
        issue_amounts: Vec<u64>,
        replace_outpoints: Vec<Outpoint>,
        inflation_info: Vec<(Outpoint, u64)>,
    ) -> Self {
        let spec = AssetSpec::with(
            ticker,
            name,
            Precision::try_from(precision).unwrap(),
            details,
        )
        .unwrap();
        let text = RicardianContract::from_str(terms_text).unwrap();
        let attachment = terms_media_fpath.map(attachment_from_fpath);
        let terms = ContractTerms {
            text,
            media: attachment,
        };
        Self::Ifa {
            spec,
            terms,
            opid_reject_url: opid_reject_url
                .map(|u| OpidRejectUrl::try_from(u.to_owned()).unwrap()),
            issue_amounts,
            replace_outpoints,
            inflation_info,
        }
    }
    pub fn add_global_state(&self, mut builder: ContractBuilder) -> ContractBuilder {
        match self {
            Self::Nia {
                spec,
                terms,
                issue_amounts,
            } => builder
                .add_global_state("spec", spec.clone())
                .unwrap()
                .add_global_state("terms", terms.clone())
                .unwrap()
                .add_global_state(
                    "issuedSupply",
                    Amount::from(issue_amounts.iter().sum::<u64>()),
                )
                .unwrap(),
            Self::Uda {
                spec,
                terms,
                token_data,
            } => builder
                .add_global_state("spec", spec.clone())
                .unwrap()
                .add_global_state("terms", terms.clone())
                .unwrap()
                .add_global_state("tokens", token_data.clone())
                .unwrap(),
            Self::Cfa {
                name,
                precision,
                details,
                terms,
                issue_amounts: issued_supply,
            } => {
                builder = builder
                    .add_global_state("name", name.clone())
                    .unwrap()
                    .add_global_state("precision", *precision)
                    .unwrap()
                    .add_global_state("terms", terms.clone())
                    .unwrap()
                    .add_global_state(
                        "issuedSupply",
                        Amount::from(issued_supply.iter().sum::<u64>()),
                    )
                    .unwrap();
                if let Some(details) = details {
                    builder = builder
                        .add_global_state("details", details.clone())
                        .unwrap()
                }
                builder
            }
            Self::Pfa {
                spec,
                terms,
                issue_amounts,
                pubkey,
            } => builder
                .add_global_state("pubkey", *pubkey)
                .unwrap()
                .add_global_state("spec", spec.clone())
                .unwrap()
                .add_global_state("terms", terms.clone())
                .unwrap()
                .add_global_state(
                    "issuedSupply",
                    Amount::from(issue_amounts.iter().sum::<u64>()),
                )
                .unwrap(),
            Self::Ifa {
                spec,
                terms,
                issue_amounts,
                opid_reject_url,
                inflation_info,
                ..
            } => {
                let issue_amount = Amount::from(issue_amounts.iter().sum::<u64>());
                let inflation_amount =
                    Amount::from(inflation_info.iter().map(|(_, amt)| amt).sum::<u64>());
                builder = builder
                    .add_global_state("spec", spec.clone())
                    .unwrap()
                    .add_global_state("terms", terms.clone())
                    .unwrap()
                    .add_global_state("issuedSupply", issue_amount)
                    .unwrap()
                    .add_global_state("maxSupply", issue_amount + inflation_amount)
                    .unwrap();
                if let Some(opid_reject_url) = opid_reject_url {
                    builder = builder
                        .add_global_state("opidRejectUrl", opid_reject_url.clone())
                        .unwrap()
                }
                builder
            }
        }
    }

    pub fn add_asset_owner(
        &self,
        mut builder: ContractBuilder,
        outpoints: Vec<Outpoint>,
        blinding: Option<u64>,
    ) -> ContractBuilder {
        match self {
            Self::Nia { issue_amounts, .. }
            | Self::Cfa { issue_amounts, .. }
            | Self::Pfa { issue_amounts, .. }
            | Self::Ifa { issue_amounts, .. } => {
                for (amt, outpoint) in issue_amounts.iter().zip(outpoints.iter().cycle()) {
                    builder = builder
                        .add_fungible_state(
                            "assetOwner",
                            get_builder_seal(*outpoint, blinding),
                            *amt,
                        )
                        .unwrap();
                }
                builder
            }
            Self::Uda { token_data, .. } => {
                let fraction = OwnedFraction::from(1);
                let allocation = Allocation::with(token_data.index, fraction);
                builder
                    .add_data(
                        "assetOwner",
                        get_builder_seal(outpoints[0], blinding),
                        allocation,
                    )
                    .unwrap()
            }
        }
    }

    pub fn add_inflation_allowance(
        &self,
        mut builder: ContractBuilder,
        blinding: Option<u64>,
    ) -> ContractBuilder {
        if let Self::Ifa { inflation_info, .. } = self {
            for (outpoint, amt) in inflation_info {
                builder = builder
                    .add_fungible_state(
                        "inflationAllowance",
                        get_builder_seal(*outpoint, blinding),
                        *amt,
                    )
                    .unwrap();
            }
        }
        builder
    }

    pub fn add_replace_right(
        &self,
        mut builder: ContractBuilder,
        blinding: Option<u64>,
    ) -> ContractBuilder {
        if let Self::Ifa {
            replace_outpoints, ..
        } = self
        {
            for outpoint in replace_outpoints {
                builder = builder
                    .add_rights("replaceRight", get_builder_seal(*outpoint, blinding))
                    .unwrap();
            }
        }
        builder
    }
}

pub struct Report {
    pub report_path: PathBuf,
}

impl Report {
    pub fn write_header(&self, fields: &[&str]) {
        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&self.report_path)
            .unwrap();
        file.write_all(format!("{}\n", fields.join(";")).as_bytes())
            .unwrap();
    }

    pub fn write_duration(&self, duration: Duration) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.report_path)
            .unwrap();
        file.write_all(format!("{};", duration.as_millis()).as_bytes())
            .unwrap();
    }

    pub fn end_line(&self) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.report_path)
            .unwrap();
        file.write_all("\n".as_bytes()).unwrap();
    }
}

pub fn get_builder_seal(outpoint: Outpoint, blinding: Option<u64>) -> BuilderSeal<BlindSeal<Txid>> {
    let blind_seal = if let Some(blinding) = blinding {
        BlindSeal::with_blinding(outpoint.txid, outpoint.vout, blinding)
    } else {
        BlindSeal::new_random(outpoint.txid, outpoint.vout)
    };
    BuilderSeal::from(blind_seal)
}

fn _get_wallet(
    descriptor_type: &DescriptorType,
    network: Network,
    wallet_dir: PathBuf,
    wallet_account: WalletAccount,
    instance: u8,
    import_kits: bool,
) -> TestWallet {
    std::fs::create_dir_all(&wallet_dir).unwrap();
    println!("wallet dir: {wallet_dir:?}");

    let xpub_account = match wallet_account {
        WalletAccount::Private(ref xpriv_account) => xpriv_account.to_xpub_account(),
        WalletAccount::Public(ref xpub_account) => xpub_account.clone(),
    };
    const OPRET_KEYCHAINS: [Keychain; 3] = [
        Keychain::INNER,
        Keychain::OUTER,
        Keychain::with(RgbKeychain::Rgb as u8),
    ];
    const TAPRET_KEYCHAINS: [Keychain; 4] = [
        Keychain::INNER,
        Keychain::OUTER,
        Keychain::with(RgbKeychain::Rgb as u8),
        Keychain::with(RgbKeychain::Tapret as u8),
    ];
    let keychains: &[Keychain] = match *descriptor_type {
        DescriptorType::Tr => &TAPRET_KEYCHAINS[..],
        DescriptorType::Wpkh => &OPRET_KEYCHAINS[..],
    };
    let xpub_derivable = XpubDerivable::with(xpub_account.clone(), keychains);

    let descriptor = match descriptor_type {
        DescriptorType::Wpkh => RgbDescr::Wpkh(Wpkh::from(xpub_derivable)),
        DescriptorType::Tr => RgbDescr::TapretKey(TapretKey::from(xpub_derivable)),
    };

    let name = "bp_wallet_name";
    let mut bp_wallet = Wallet::new_layer1(descriptor.clone(), network);
    bp_wallet.set_name(name.to_string());
    let bp_dir = wallet_dir.join(name);
    let bp_wallet_provider = FsTextStore::new(bp_dir).unwrap();
    bp_wallet.make_persistent(bp_wallet_provider, true).unwrap();

    let stock_provider = FsBinStore::new(wallet_dir.clone()).unwrap();
    let mut stock = Stock::in_memory();
    stock.make_persistent(stock_provider, true).unwrap();
    let mut wallet = RgbWallet::new(stock, bp_wallet);

    if import_kits {
        for asset_schema in AssetSchema::iter() {
            let valid_kit = asset_schema.get_valid_kit();
            wallet.stock_mut().import_kit(valid_kit).unwrap();
        }
    }

    let signer = match wallet_account {
        WalletAccount::Private(xpriv_account) => Some(TestnetSigner::new(xpriv_account)),
        WalletAccount::Public(_) => None,
    };

    let mut wallet = TestWallet {
        wallet,
        signer,
        wallet_dir,
        instance,
    };

    wallet.sync();

    wallet
}

pub fn get_wallet(descriptor_type: &DescriptorType) -> TestWallet {
    get_wallet_custom(descriptor_type, None, true)
}

pub fn get_wallet_custom(
    descriptor_type: &DescriptorType,
    instance: Option<u8>,
    import_kits: bool,
) -> TestWallet {
    let mut seed = vec![0u8; 128];
    rand::thread_rng().fill_bytes(&mut seed);

    let xpriv_account = XprivAccount::with_seed(true, &seed).derive(h![86, 1, 0]);

    let fingerprint = xpriv_account.account_fp().to_string();
    let wallet_dir = PathBuf::from(TEST_DATA_DIR)
        .join(INTEGRATION_DATA_DIR)
        .join(fingerprint);

    _get_wallet(
        descriptor_type,
        Network::Regtest,
        wallet_dir,
        WalletAccount::Private(xpriv_account),
        instance.unwrap_or(INSTANCE_1),
        import_kits,
    )
}

pub fn get_mainnet_wallet() -> TestWallet {
    let xpub_account = XpubAccount::from_str(
        "[c32338a7/86h/0h/0h]xpub6CmiK1xc7YwL472qm4zxeURFX8yMCSasioXujBjVMMzA3AKZr6KLQEmkzDge1Ezn2p43ZUysyx6gfajFVVnhtQ1AwbXEHrioLioXXgj2xW5"
    ).unwrap();

    let wallet_dir = PathBuf::from(TEST_DATA_DIR)
        .join(INTEGRATION_DATA_DIR)
        .join("mainnet");

    _get_wallet(
        &DescriptorType::Wpkh,
        Network::Mainnet,
        wallet_dir,
        WalletAccount::Public(xpub_account),
        INSTANCE_1,
        true,
    )
}

fn get_indexer(indexer_url: &str) -> AnyIndexer {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => {
            AnyIndexer::Electrum(Box::new(ElectrumClient::new(indexer_url).unwrap()))
        }
        Indexer::Esplora => {
            AnyIndexer::Esplora(Box::new(EsploraClient::new_esplora(indexer_url).unwrap()))
        }
    }
}

fn get_resolver(indexer_url: &str) -> AnyResolver {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => AnyResolver::electrum_blocking(indexer_url, None).unwrap(),
        Indexer::Esplora => AnyResolver::esplora_blocking(indexer_url, None).unwrap(),
    }
}

fn broadcast_tx(tx: &Tx, indexer_url: &str) {
    match get_indexer(indexer_url) {
        AnyIndexer::Electrum(inner) => {
            inner.transaction_broadcast(tx).unwrap();
        }
        AnyIndexer::Esplora(inner) => {
            inner.publish(tx).unwrap();
        }
        _ => unreachable!("unsupported indexer"),
    }
}

pub fn broadcast_tx_and_mine(tx: &Tx, instance: u8) {
    broadcast_tx(tx, &indexer_url(instance, Network::Regtest));
    mine_custom(false, instance, 1);
}

pub fn attachment_from_fpath(fpath: &str) -> Attachment {
    let file_bytes = std::fs::read(fpath).unwrap();
    let file_hash: sha256::Hash = Hash::hash(&file_bytes[..]);
    let digest = file_hash.to_byte_array().into();
    let mime = FileFormat::from_file(fpath)
        .unwrap()
        .media_type()
        .to_string();
    let media_ty: &'static str = Box::leak(mime.clone().into_boxed_str());
    let media_type = MediaType::with(media_ty);
    Attachment {
        ty: media_type,
        digest,
    }
}

fn uda_token_data_minimal() -> TokenData {
    TokenData {
        index: TokenIndex::from(UDA_FIXED_INDEX),
        ..Default::default()
    }
}

pub fn uda_token_data(
    ticker: &str,
    name: &str,
    details: &str,
    preview: EmbeddedMedia,
    media: Attachment,
    attachments: BTreeMap<u8, Attachment>,
    reserves: ProofOfReserves,
) -> TokenData {
    let mut token_data = uda_token_data_minimal();
    token_data.preview = Some(preview);
    token_data.media = Some(media);
    token_data.attachments = Confined::try_from(attachments.clone()).unwrap();
    token_data.reserves = Some(reserves);
    token_data.ticker = Some(Ticker::try_from(ticker.to_string()).unwrap());
    token_data.name = Some(Name::try_from(name.to_string()).unwrap());
    token_data.details = Some(Details::try_from(details.to_string()).unwrap());
    token_data
}

impl TestWallet {
    pub fn network(&self) -> Network {
        self.wallet.wallet().network()
    }

    pub fn chain_net(&self) -> ChainNet {
        match self.network() {
            Network::Mainnet => ChainNet::BitcoinMainnet,
            Network::Regtest => ChainNet::BitcoinRegtest,
            Network::Signet => ChainNet::BitcoinSignet,
            Network::Testnet3 => ChainNet::BitcoinTestnet3,
            Network::Testnet4 => ChainNet::BitcoinTestnet4,
        }
    }

    pub fn testnet(&self) -> bool {
        self.network().is_testnet()
    }

    pub fn keychain(&self) -> RgbKeychain {
        RgbKeychain::for_method(self.close_method())
    }

    fn get_next_index(&mut self, keychain: impl Into<Keychain>, shift: bool) -> NormalIndex {
        self.wallet
            .wallet_mut()
            .next_derivation_index(keychain, shift)
    }

    pub fn get_derived_address(&mut self, shift: bool) -> DerivedAddr {
        let keychain = self.keychain();
        let index = self.get_next_index(keychain, shift);
        self.wallet
            .wallet()
            .addresses(keychain)
            .nth(index.index() as usize)
            .expect("address iterator always can produce address")
    }

    pub fn get_address(&mut self) -> Address {
        self.get_derived_address(true).addr
    }

    pub fn get_utxo(&mut self, sats: Option<u64>) -> Outpoint {
        let address = self.get_address();
        let txid = Txid::from_str(&fund_wallet(address.to_string(), sats, self.instance)).unwrap();
        self.sync();
        let mut vout = None;
        let coins = self.wallet.wallet().address_coins();
        assert!(!coins.is_empty());
        for (_derived_addr, utxos) in coins {
            for utxo in utxos {
                if utxo.outpoint.txid == txid {
                    vout = Some(utxo.outpoint.vout_u32());
                }
            }
        }
        Outpoint {
            txid,
            vout: Vout::from_u32(vout.unwrap()),
        }
    }

    pub fn change_instance(&mut self, instance: u8) {
        self.instance = instance;
    }

    pub fn sync_and_update_witnesses(&mut self, after_height: Option<u32>) {
        self.sync();
        self.update_witnesses(after_height.unwrap_or(1), vec![]);
    }

    pub fn switch_to_instance(&mut self, instance: u8) {
        self.change_instance(instance);
        self.sync_and_update_witnesses(None);
    }

    pub fn indexer_url(&self) -> String {
        indexer_url(self.instance, self.network())
    }

    fn get_indexer(&self) -> AnyIndexer {
        get_indexer(&self.indexer_url())
    }

    pub fn get_resolver(&self) -> AnyResolver {
        get_resolver(&self.indexer_url())
    }

    pub fn broadcast_tx(&self, tx: &Tx) {
        broadcast_tx(tx, &self.indexer_url());
    }

    pub fn get_witness_ord(&self, txid: &Txid) -> WitnessOrd {
        self.get_resolver().resolve_pub_witness_ord(*txid).unwrap()
    }

    pub fn get_tx_height(&self, txid: &Txid) -> Option<u32> {
        match self.get_witness_ord(txid) {
            WitnessOrd::Mined(witness_pos) => Some(witness_pos.height().get()),
            _ => None,
        }
    }

    pub fn sync(&mut self) {
        let indexer = self.get_indexer();
        self.wallet
            .wallet_mut()
            .sync_from_scratch(&indexer)
            .into_result()
            .unwrap();
    }

    pub fn utxo(&self, outpoint: &Outpoint) -> (Utxo, ScriptPubkey) {
        self.wallet.wallet().utxo(*outpoint).unwrap()
    }

    pub fn close_method(&self) -> CloseMethod {
        self.wallet.wallet().close_method()
    }

    pub fn descriptor(&self) -> &RgbDescr {
        self.wallet.wallet().descriptor()
    }

    pub fn mine_tx(&self, txid: &Txid, resume: bool) {
        let mut attempts = 10;
        loop {
            mine_custom(resume, self.instance, 1);
            if self.get_tx_height(txid).is_some() {
                break;
            }
            attempts -= 1;
            if attempts == 0 {
                panic!("TX is not getting mined");
            }
        }
    }

    pub fn schema_id(&self, contract_id: ContractId) -> SchemaId {
        self.wallet
            .stock()
            .as_stash_provider()
            .genesis(contract_id)
            .unwrap()
            .schema_id
    }

    pub fn asset_schema(&self, contract_id: ContractId) -> AssetSchema {
        self.schema_id(contract_id).into()
    }

    pub fn import_contract(&mut self, contract: &ValidContract, resolver: impl ResolveWitness) {
        self.wallet
            .stock_mut()
            .import_contract(contract.clone(), resolver)
            .unwrap();
    }

    pub fn issue_with_info(
        &mut self,
        asset_info: AssetInfo,
        outpoints: Vec<Option<Outpoint>>,
        created_at: Option<i64>,
        blinding: Option<u64>,
    ) -> ContractId {
        let outpoints = if outpoints.is_empty() {
            vec![self.get_utxo(None)]
        } else {
            outpoints
                .into_iter()
                .map(|o| o.unwrap_or_else(|| self.get_utxo(None)))
                .collect()
        };

        let mut builder = ContractBuilder::with(
            Identity::default(),
            asset_info.schema(),
            asset_info.types(),
            asset_info.scripts(),
            self.chain_net(),
        );
        builder = asset_info.add_global_state(builder);
        builder = asset_info.add_asset_owner(builder, outpoints, blinding);
        builder = asset_info.add_inflation_allowance(builder, blinding);
        builder = asset_info.add_replace_right(builder, blinding);

        let created_at = created_at.unwrap_or_else(|| Utc::now().timestamp());
        let contract = builder.issue_contract_raw(created_at).unwrap();
        let resolver = self.get_resolver();
        self.import_contract(&contract, resolver);

        contract.contract_id()
    }

    pub fn issue_nia(&mut self, issued_supply: u64, outpoint: Option<&Outpoint>) -> ContractId {
        let asset_info = AssetInfo::default_nia(vec![issued_supply]);
        self.issue_with_info(asset_info, vec![outpoint.copied()], None, None)
    }

    pub fn issue_uda(&mut self, outpoint: Option<&Outpoint>) -> ContractId {
        let asset_info = AssetInfo::default_uda();
        self.issue_with_info(asset_info, vec![outpoint.copied()], None, None)
    }

    pub fn issue_cfa(&mut self, issued_supply: u64, outpoint: Option<&Outpoint>) -> ContractId {
        let asset_info = AssetInfo::default_cfa(vec![issued_supply]);
        self.issue_with_info(asset_info, vec![outpoint.copied()], None, None)
    }

    pub fn issue_pfa(
        &mut self,
        issued_supply: u64,
        outpoint: Option<&Outpoint>,
        pubkey: CompressedPk,
    ) -> ContractId {
        let asset_info = AssetInfo::default_pfa(vec![issued_supply], pubkey);
        self.issue_with_info(asset_info, vec![outpoint.copied()], None, None)
    }

    pub fn issue_ifa(
        &mut self,
        issued_supply: u64,
        outpoint: Option<&Outpoint>,
        replace_outpoints: Vec<Outpoint>,
        inflation_info: Vec<(Outpoint, u64)>,
    ) -> ContractId {
        let asset_info =
            AssetInfo::default_ifa(vec![issued_supply], replace_outpoints, inflation_info);
        self.issue_with_info(asset_info, vec![outpoint.copied()], None, None)
    }

    pub fn get_secret_seal(
        &mut self,
        outpoint: Option<Outpoint>,
        static_blinding: Option<u64>,
    ) -> SecretSeal {
        let outpoint = outpoint.unwrap_or_else(|| self.get_utxo(None));
        let seal = GraphSeal::from(match static_blinding {
            Some(bli) => BlindSeal::with_blinding(outpoint.txid, outpoint.vout, bli),
            None => BlindSeal::new_random(outpoint.txid, outpoint.vout),
        });
        self.wallet.stock_mut().store_secret_seal(seal).unwrap();
        seal.to_secret_seal()
    }

    pub fn invoice(
        &mut self,
        contract_id: ContractId,
        schema_id: SchemaId,
        amount: u64,
        invoice_type: impl Into<InvoiceType>,
    ) -> RgbInvoice {
        let network = self.network();
        let beneficiary = match invoice_type.into() {
            InvoiceType::Blinded(outpoint) => {
                Beneficiary::BlindedSeal(self.get_secret_seal(outpoint, None))
            }
            InvoiceType::Witness => {
                let address = self.get_address();
                Beneficiary::WitnessVout(Pay2Vout::new(address.payload), None)
            }
            InvoiceType::WitnessTapret => {
                let keychain = self.keychain();
                let index = self.get_next_index(keychain, true);
                let descr = self.descriptor();
                let tap_internal_key = descr
                    .derive(keychain, index)
                    .next()
                    .unwrap()
                    .to_internal_pk()
                    .expect("not a taproot wallet");
                let address = Address::with(
                    &ScriptPubkey::p2tr_key_only(tap_internal_key),
                    self.network(),
                )
                .unwrap();
                Beneficiary::WitnessVout(Pay2Vout::new(address.payload), Some(tap_internal_key))
            }
        };

        let mut builder = RgbInvoiceBuilder::new(XChainNet::bitcoin(network, beneficiary))
            .set_contract(contract_id)
            .set_schema(schema_id);

        if matches!(schema_id.into(), AssetSchema::Uda) {
            if amount != 1 {
                panic!("UDA amount must be 1");
            }
            builder = builder
                .clone()
                .set_allocation(UDA_FIXED_INDEX, amount)
                .unwrap();
        } else {
            builder = builder.clone().set_amount_raw(amount);
        }
        builder.finish()
    }

    pub fn sign_finalize(&self, psbt: &mut Psbt) {
        let _sig_count = psbt.sign(self.signer.as_ref().unwrap()).unwrap();
        psbt.finalize(self.descriptor());
    }

    pub fn sign_finalize_extract(&self, psbt: &mut Psbt) -> Tx {
        self.sign_finalize(psbt);
        psbt.extract().unwrap()
    }

    pub fn consign_transfer(
        &self,
        contract_id: ContractId,
        outputs: impl AsRef<[OutputSeal]>,
        secret_seals: impl AsRef<[SecretSeal]>,
        witness_id: Option<Txid>,
    ) -> Transfer {
        self.wallet
            .stock()
            .transfer(contract_id, outputs, secret_seals, witness_id)
            .unwrap()
    }

    pub fn pay(
        &mut self,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
    ) -> (Psbt, PsbtMeta, Transfer) {
        let fee = Sats::from_sats(fee.unwrap_or(DEFAULT_FEE_ABS));
        let sats = Sats::from_sats(sats.unwrap_or(2000));
        let params = TransferParams::with(fee, sats);
        self.wallet.pay(&invoice, params).unwrap()
    }

    pub fn pay_full(
        &mut self,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
        broadcast: bool,
        report: Option<&Report>,
    ) -> (Transfer, Tx, Psbt, PsbtMeta) {
        self.sync();

        let pay_start = Instant::now();
        let (mut psbt, psbt_meta, consignment) = self.pay(invoice, sats, fee);
        let pay_duration = pay_start.elapsed();
        if let Some(report) = report {
            report.write_duration(pay_duration);
        }

        let mut cs_path = self.wallet_dir.join("consignments");
        std::fs::create_dir_all(&cs_path).unwrap();
        let consignment_id = consignment.consignment_id();
        cs_path.push(consignment_id.to_string());
        cs_path.set_extension("json");
        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(cs_path)
            .unwrap();
        serde_json::to_writer(&mut file, &consignment).unwrap();

        let tx = self.sign_finalize_extract(&mut psbt);

        let txid = tx.txid().to_string();
        println!("transfer txid: {txid}, consignment: {consignment_id}");

        let mut tx_path = self.wallet_dir.join("transactions");
        std::fs::create_dir_all(&tx_path).unwrap();
        tx_path.push(&txid);
        tx_path.set_extension("json");
        let mut file = std::fs::File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(tx_path)
            .unwrap();
        serde_json::to_writer(&mut file, &tx).unwrap();
        writeln!(file, "\n---\n").unwrap();
        serde_json::to_writer(&mut file, &psbt).unwrap();

        if broadcast {
            self.broadcast_tx(&tx);
        }

        (consignment, tx, psbt, psbt_meta)
    }

    pub fn accept_transfer(
        &mut self,
        consignment: Transfer,
        report: Option<&Report>,
    ) -> BTreeSet<OpId> {
        let resolver = self.get_resolver();
        self.accept_transfer_custom(consignment, report, &resolver, bset![])
    }

    pub fn accept_transfer_custom(
        &mut self,
        consignment: Transfer,
        report: Option<&Report>,
        resolver: &impl ResolveWitness,
        trusted_op_seals: BTreeSet<OpId>,
    ) -> BTreeSet<OpId> {
        self.sync();
        let validate_start = Instant::now();
        let validated_consignment = consignment
            .clone()
            .validate_with_opids(&resolver, self.chain_net(), None, trusted_op_seals)
            .unwrap();
        let validate_duration = validate_start.elapsed();
        if let Some(report) = report {
            report.write_duration(validate_duration);
        }

        let validation_status = validated_consignment.clone().into_validation_status();
        let validity = validation_status.validity();
        assert_eq!(validity, Validity::Valid);
        let accept_start = Instant::now();
        self.wallet
            .stock_mut()
            .accept_transfer(validated_consignment.clone(), &resolver)
            .unwrap();
        let accept_duration = accept_start.elapsed();
        if let Some(report) = report {
            report.write_duration(accept_duration);
        }
        validated_consignment.validated_opids().clone()
    }

    pub fn try_add_tapret_tweak(&mut self, consignment: Transfer, txid: &Txid) {
        self.wallet
            .wallet_mut()
            .try_add_tapret_tweak(consignment, txid)
            .unwrap();
    }

    pub fn contract_data(
        &self,
        contract_id: ContractId,
    ) -> ContractData<MemContract<&MemContractState>> {
        self.wallet.stock().contract_data(contract_id).unwrap()
    }

    pub fn contract_wrapper<C: IssuerWrapper>(
        &self,
        contract_id: ContractId,
    ) -> C::Wrapper<MemContract<&MemContractState>> {
        self.wallet
            .stock()
            .contract_wrapper::<C>(contract_id)
            .unwrap()
    }

    pub fn contract_fungible_allocations(
        &self,
        contract_id: ContractId,
        show_tentative: bool,
    ) -> Vec<FungibleAllocation> {
        let filter = if show_tentative {
            Filter::WalletTentative(&self.wallet)
        } else {
            Filter::Wallet(&self.wallet)
        };
        self.contract_data(contract_id)
            .fungible("assetOwner", filter)
            .unwrap()
            .collect()
    }

    pub fn contract_data_allocations(&self, contract_id: ContractId) -> Vec<DataAllocation> {
        self.contract_data(contract_id)
            .data("assetOwner", Filter::Wallet(&self.wallet))
            .unwrap()
            .collect()
    }

    pub fn history(&self, contract_id: ContractId) -> Vec<ContractOp> {
        self.wallet.history(contract_id).unwrap()
    }

    pub fn list_contracts(&self) -> Vec<ContractInfo> {
        self.wallet.stock().contracts().unwrap().collect()
    }

    pub fn utxos(&self) -> Vec<WalletUtxo> {
        self.wallet.wallet().utxos().collect()
    }

    pub fn balance(&self) -> u64 {
        self.wallet.wallet().balance().0
    }

    pub fn debug_contracts(&self) {
        println!("Contracts:");
        for info in self.list_contracts() {
            println!("{}", info.to_string().replace("\n", "\t"));
        }
    }

    pub fn debug_logs(&self, contract_id: ContractId, filter: AllocationFilter) {
        let filter = filter.filter_for(self);

        let contract = self.contract_data(contract_id);

        println!("Global:");
        for global_details in contract.schema.global_types.values() {
            let values = contract.global(global_details.name.clone());
            for val in values {
                println!("  {} := {}", global_details.name, val);
            }
        }

        println!("\nOwned:");
        fn witness<S: KnownState>(
            allocation: &OutputAssignment<S>,
            contract: &ContractData<MemContract<&MemContractState>>,
        ) -> String {
            allocation
                .witness
                .and_then(|w| contract.witness_info(w))
                .map(|info| format!("{} ({})", info.id, info.ord))
                .unwrap_or_else(|| s!("~"))
        }
        for details in contract.schema.owned_types.values() {
            println!("  State      \t{:78}\tWitness", "Seal");
            println!("  {}:", details.name);
            if let Ok(allocations) = contract.fungible(details.name.clone(), &filter) {
                for allocation in allocations {
                    println!(
                        "    {: >9}\t{}\t{} {}",
                        allocation.state.value(),
                        allocation.seal,
                        witness(&allocation, &contract),
                        filter.comment(allocation.seal.to_outpoint())
                    );
                }
            }
            if let Ok(allocations) = contract.data(details.name.clone(), &filter) {
                for allocation in allocations {
                    println!(
                        "    {: >9}\t{}\t{} {}",
                        allocation.state,
                        allocation.seal,
                        witness(&allocation, &contract),
                        filter.comment(allocation.seal.to_outpoint())
                    );
                }
            }
            if let Ok(allocations) = contract.rights(details.name.clone(), &filter) {
                for allocation in allocations {
                    println!(
                        "    {: >9}\t{}\t{} {}",
                        "right",
                        allocation.seal,
                        witness(&allocation, &contract),
                        filter.comment(allocation.seal.to_outpoint())
                    );
                }
            }
        }

        println!("\nHeight\t{:>12}\t{:68}", "Amount, ṩ", "Outpoint");
        for (derived_addr, utxos) in self.wallet.wallet().address_coins() {
            println!("{}\t{}", derived_addr.addr, derived_addr.terminal);
            for row in utxos {
                println!("{}\t{: >12}\t{:68}", row.height, row.amount, row.outpoint);
            }
            println!()
        }

        println!("\nWallet total balance: {} ṩ", self.balance());
    }

    pub fn debug_history(&self, contract_id: ContractId, details: bool) {
        let mut history = self.history(contract_id);
        history.sort_by_key(|op| op.witness.map(|w| w.ord).unwrap_or(WitnessOrd::Archived));
        if details {
            println!("Operation\tValue    \tState\t{:78}\tWitness", "Seal");
        } else {
            println!("Operation\tValue    \t{:78}\tWitness", "Seal");
        }
        for ContractOp {
            direction,
            ty,
            opids,
            state,
            to,
            witness,
        } in history
        {
            print!("{:9}\t", direction.to_string());
            if let AllocatedState::Amount(amount) = state {
                print!("{: >9}", amount.as_u64());
            } else {
                print!("{state:>9}");
            }
            if details {
                print!("\t{ty}");
            }
            println!(
                "\t{}\t{}",
                to.first().expect("at least one receiver is always present"),
                witness
                    .map(|info| format!("{} ({})", info.id, info.ord))
                    .unwrap_or_else(|| s!("~"))
            );
            if details {
                println!(
                    "\topid={}",
                    opids
                        .iter()
                        .map(OpId::to_string)
                        .collect::<Vec<_>>()
                        .join("\n\topid=")
                )
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn send(
        &mut self,
        recv_wlt: &mut TestWallet,
        invoice_type: impl Into<InvoiceType>,
        contract_id: ContractId,
        amount: u64,
        sats: u64,
        report: Option<&Report>,
    ) -> (Transfer, Tx) {
        let schema_id = self.schema_id(contract_id);
        let invoice = recv_wlt.invoice(contract_id, schema_id, amount, invoice_type.into());
        self.send_to_invoice(recv_wlt, invoice, Some(sats), None, report)
    }

    pub fn send_to_invoice(
        &mut self,
        recv_wlt: &mut TestWallet,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
        report: Option<&Report>,
    ) -> (Transfer, Tx) {
        let (consignment, tx, _, _) = self.pay_full(invoice, sats, fee, true, report);
        self.mine_tx(&tx.txid(), false);
        recv_wlt.accept_transfer(consignment.clone(), report);
        self.sync();
        (consignment, tx)
    }

    pub fn send_pfa(
        &mut self,
        recv_wlt: &mut TestWallet,
        transfer_type: TransferType,
        contract_id: ContractId,
        amount: u64,
        secret_key: SecretKey,
    ) {
        let transition_signer = |witness_bundle: &mut WitnessBundle| {
            for transition in witness_bundle.bundle_mut().known_transitions.values_mut() {
                let transition_id: [u8; 32] = transition.id().as_ref().into_inner();
                let msg = Message::from_digest(transition_id);
                let signature = secret_key.sign_ecdsa(msg);
                transition.signature =
                    Some(Bytes64::from_array(signature.serialize_compact()).into());
            }
        };

        let schema_id = self.schema_id(contract_id);
        assert_eq!(schema_id, AssetSchema::Pfa.schema().schema_id());
        let invoice = recv_wlt.invoice(contract_id, schema_id, amount, transfer_type);
        let (mut consignment, tx, psbt, psbt_meta) = self.pay_full(invoice, None, None, true, None);
        let txid = tx.txid();
        consignment.modify_bundle(txid, transition_signer);
        self.accept_transfer(consignment.clone(), None);
        let output_seal: OutputSeal =
            ExplicitSeal::new(Outpoint::new(txid, psbt_meta.change_vout.unwrap()));
        for cid in psbt.rgb_contract_ids().unwrap() {
            if cid == contract_id {
                continue;
            }
            let mut extra_cons = self.consign_transfer(cid, vec![output_seal], vec![], Some(txid));
            let changed = extra_cons.modify_bundle(txid, transition_signer);
            assert!(changed);
            self.accept_transfer(extra_cons.clone(), None);
        }
        self.mine_tx(&txid, false);
        recv_wlt.accept_transfer(consignment.clone(), None);
        self.sync();
    }

    pub fn send_ifa(
        &mut self,
        recv_wlt: &mut TestWallet,
        invoice_type: impl Into<InvoiceType>,
        contract_id: ContractId,
        amount: u64,
    ) -> (Transfer, Tx, BTreeSet<OpId>) {
        let schema_id = self.schema_id(contract_id);
        let invoice = recv_wlt.invoice(contract_id, schema_id, amount, invoice_type.into());
        self.send_ifa_to_invoice(recv_wlt, invoice)
    }

    pub fn send_ifa_to_invoice(
        &mut self,
        recv_wlt: &mut TestWallet,
        invoice: RgbInvoice,
    ) -> (Transfer, Tx, BTreeSet<OpId>) {
        let (consignment, tx, _, _) = self.pay_full(invoice, None, None, true, None);
        self.mine_tx(&tx.txid(), false);
        let trusted_op_seals = consignment.replace_transitions_input_ops();
        let validated_opids = recv_wlt.accept_transfer_custom(
            consignment.clone(),
            None,
            &recv_wlt.get_resolver(),
            trusted_op_seals,
        );
        self.sync();
        (consignment, tx, validated_opids)
    }

    pub fn inflate_ifa(
        &mut self,
        contract_id: ContractId,
        inflation_outpoints: Vec<Outpoint>,
        inflation_amounts: Vec<u64>,
    ) {
        let contract = self.contract_wrapper::<InflatableFungibleAsset>(contract_id);
        let inflation_allocations = contract
            .inflation_allocations(Filter::Wallet(&self.wallet))
            .filter(|oa| inflation_outpoints.contains(&oa.seal.outpoint().unwrap()))
            .collect::<Vec<_>>();
        let inflation_supply: u64 = inflation_allocations
            .iter()
            .map(|oa| oa.state.value())
            .sum();

        let total_inflation_amount: u64 = inflation_amounts.iter().sum();
        let inflation_change = inflation_supply - total_inflation_amount;

        let mut psbt_beneficiaries = vec![];
        let mut num_psbt_beneficiaries = inflation_amounts.len();
        if inflation_change > 0 {
            num_psbt_beneficiaries += 1;
        }
        (0..num_psbt_beneficiaries)
            .for_each(|_| psbt_beneficiaries.push((self.get_address(), None)));

        let (mut psbt, _) = self.construct_psbt(inflation_outpoints, psbt_beneficiaries, None);
        let mut asset_transition_builder = self
            .wallet
            .stock()
            .transition_builder(contract_id, "inflate")
            .unwrap();
        let prev_outputs = psbt
            .inputs()
            .map(|txin| txin.previous_outpoint)
            .collect::<HashSet<_>>();
        for (_, opout_state_map) in self
            .wallet
            .stock()
            .contract_assignments_for(contract_id, prev_outputs)
            .unwrap()
        {
            for (opout, state) in opout_state_map {
                asset_transition_builder =
                    asset_transition_builder.add_input(opout, state).unwrap();
            }
        }
        let mut beneficiaries = vec![];
        for (vout, inflation_amount) in inflation_amounts.into_iter().enumerate() {
            let seal = BuilderSeal::Revealed(GraphSeal::new_random_vout(vout as u32));
            beneficiaries.push(seal);
            asset_transition_builder = asset_transition_builder
                .add_fungible_state("assetOwner", seal, inflation_amount)
                .unwrap();
        }

        if inflation_change > 0 {
            let change_vout = num_psbt_beneficiaries as u32 - 1;
            let seal = BuilderSeal::Revealed(GraphSeal::new_random_vout(change_vout));
            beneficiaries.push(seal);
            asset_transition_builder = asset_transition_builder
                .add_fungible_state(fname!("inflationAllowance"), seal, inflation_change)
                .unwrap();
        }
        asset_transition_builder = asset_transition_builder
            .add_global_state("issuedSupply", Amount::from(total_inflation_amount))
            .unwrap()
            .add_metadata("allowedInflation", Amount::from(inflation_change))
            .unwrap();
        let transition = asset_transition_builder.complete_transition().unwrap();
        for input in psbt.inputs_mut() {
            input
                .set_rgb_consumer(contract_id, transition.id())
                .unwrap();
        }
        psbt.push_rgb_transition(transition).unwrap();
        psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO)
            .set_opret_host()
            .unwrap();
        psbt.set_rgb_close_method(CloseMethod::OpretFirst);
        psbt.complete_construction();
        let fascia = psbt.rgb_commit().unwrap();
        self.consume_fascia(fascia, psbt.txid());
        let tx = self.sign_finalize_extract(&mut psbt);
        self.broadcast_tx(&tx);
        self.mine_tx(&tx.txid(), false);
        println!("inflation txid: {}", tx.txid());
        self.sync();
        let consignments = self.create_consignments(bmap![contract_id => beneficiaries], tx.txid());
        for consignment in consignments {
            let all_opids = consignment
                .bundles
                .iter()
                .flat_map(|b| b.bundle().known_transitions.keys().copied())
                .collect::<BTreeSet<_>>();
            let validated_consignment = consignment
                .clone()
                .validate_with_opids(&self.get_resolver(), self.chain_net(), None, bset![])
                .unwrap();
            assert_eq!(*validated_consignment.validated_opids(), all_opids);
        }
    }

    pub fn replace_ifa(
        &mut self,
        right_owner: &mut TestWallet,
        right_utxo: Outpoint,
        contract_id: ContractId,
    ) {
        let address = self.get_address();
        let allocations = self.contract_fungible_allocations(contract_id, false);
        let replaced_amount: u64 = allocations.iter().map(|a| a.state.value()).sum();
        let utxos = allocations.iter().map(|a| a.seal.into()).collect();
        let (mut psbt, _) = self.construct_psbt(utxos, vec![(address, None)], None);
        let (input, _) = right_owner.utxo(&right_utxo);
        right_owner.psbt_add_input(&mut psbt, right_utxo); // include replace right
        psbt.construct_output_expect(
            right_owner.get_address().script_pubkey(),
            Sats::from_sats(input.value.sats()),
        );
        let mut asset_transition_builder = right_owner
            .wallet
            .stock()
            .transition_builder(contract_id, "replace")
            .unwrap();
        let prev_outputs = psbt
            .inputs()
            .map(|txin| txin.previous_outpoint)
            .collect::<HashSet<_>>();
        for wlt in [&right_owner, &self] {
            for (_, opout_state_map) in wlt
                .wallet
                .stock()
                .contract_assignments_for(contract_id, prev_outputs.clone())
                .unwrap()
            {
                for (opout, state) in opout_state_map {
                    asset_transition_builder =
                        asset_transition_builder.add_input(opout, state).unwrap();
                }
            }
        }
        let mut beneficiaries = vec![];
        let seal = BuilderSeal::Revealed(GraphSeal::new_random_vout(0));
        beneficiaries.push(seal);
        asset_transition_builder = asset_transition_builder
            .add_fungible_state("assetOwner", seal, replaced_amount)
            .unwrap(); // add replaced allocation
        let seal = BuilderSeal::Revealed(GraphSeal::new_random_vout(1));
        beneficiaries.push(seal);
        asset_transition_builder = asset_transition_builder
            .add_rights("replaceRight", seal)
            .unwrap(); // add replace right
        let transition = asset_transition_builder.complete_transition().unwrap();
        for input in psbt.inputs_mut() {
            input
                .set_rgb_consumer(contract_id, transition.id())
                .unwrap();
        }
        psbt.push_rgb_transition(transition).unwrap();
        psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO)
            .set_opret_host()
            .unwrap();
        psbt.set_rgb_close_method(CloseMethod::OpretFirst);
        psbt.complete_construction();
        let fascia = psbt.rgb_commit().unwrap();
        self.consume_fascia(fascia.clone(), psbt.txid());
        right_owner.consume_fascia(fascia, psbt.txid());
        right_owner.sign_finalize(&mut psbt);
        let tx = self.sign_finalize_extract(&mut psbt);
        self.broadcast_tx(&tx);
        self.mine_tx(&tx.txid(), false);
        println!("replace txid: {}", tx.txid());
        self.sync();
        right_owner.sync();

        let consignments = self.create_consignments(bmap![contract_id => beneficiaries], tx.txid());
        for consignment in consignments {
            let trusted_op_seals = consignment.replace_transitions_input_ops();
            let validated_consignment = consignment
                .clone()
                .validate_with_opids(
                    &self.get_resolver(),
                    self.chain_net(),
                    None,
                    trusted_op_seals,
                )
                .unwrap();
            let resolver = right_owner.get_resolver();
            right_owner
                .wallet
                .stock_mut()
                .accept_transfer(validated_consignment.clone(), &resolver)
                .unwrap();
        }
    }

    pub fn burn_ifa(&mut self, contract_id: ContractId, utxo: Outpoint) {
        let address = self.get_address();
        let (mut psbt, _) = self.construct_psbt(vec![utxo], vec![(address, None)], None);
        let mut asset_transition_builder = self
            .wallet
            .stock()
            .transition_builder(contract_id, "burn")
            .unwrap();
        let prev_outputs = psbt
            .inputs()
            .map(|txin| txin.previous_outpoint)
            .collect::<HashSet<_>>();
        for (_, opout_state_map) in self
            .wallet
            .stock()
            .contract_assignments_for(contract_id, prev_outputs)
            .unwrap()
        {
            for (opout, state) in opout_state_map {
                asset_transition_builder =
                    asset_transition_builder.add_input(opout, state).unwrap();
            }
        }
        let transition = asset_transition_builder.complete_transition().unwrap();
        for input in psbt.inputs_mut() {
            input
                .set_rgb_consumer(contract_id, transition.id())
                .unwrap();
        }
        psbt.push_rgb_transition(transition).unwrap();
        psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO)
            .set_opret_host()
            .unwrap();
        psbt.set_rgb_close_method(CloseMethod::OpretFirst);
        psbt.complete_construction();
        let fascia = psbt.rgb_commit().unwrap();
        self.consume_fascia(fascia, psbt.txid());
        let tx = self.sign_finalize_extract(&mut psbt);
        self.broadcast_tx(&tx);
        self.mine_tx(&tx.txid(), false);
        println!("burn txid: {}", tx.txid());
        self.sync();
    }

    pub fn check_allocations(
        &self,
        contract_id: ContractId,
        asset_schema: impl Into<AssetSchema>,
        expected_fungible_allocations: Vec<u64>,
        nonfungible_allocation: bool,
    ) {
        match asset_schema.into() {
            AssetSchema::Nia | AssetSchema::Cfa | AssetSchema::Pfa | AssetSchema::Ifa => {
                let allocations = self.contract_fungible_allocations(contract_id, false);
                let mut actual_fungible_allocations = allocations
                    .iter()
                    .map(|a| a.state.value())
                    .collect::<Vec<_>>();
                let mut expected_fungible_allocations = expected_fungible_allocations.clone();
                actual_fungible_allocations.sort();
                expected_fungible_allocations.sort();
                assert_eq!(actual_fungible_allocations, expected_fungible_allocations);
            }
            AssetSchema::Uda => {
                let allocations = self.contract_data_allocations(contract_id);
                let expected_allocations = if nonfungible_allocation {
                    assert_eq!(
                        allocations
                            .iter()
                            .filter(|a| a.state.to_string() == "000000000100000000000000")
                            .count(),
                        1
                    );
                    1
                } else {
                    0
                };
                assert_eq!(allocations.len(), expected_allocations);
            }
        }
    }

    pub fn check_history_operation(
        &self,
        contract_id: &ContractId,
        txid: Option<&Txid>,
        direction: OpDirection,
        amount: u64,
    ) {
        let operation = self
            .history(*contract_id)
            .into_iter()
            .find(|co| {
                co.direction == direction
                    && co.witness.map_or(true, |w| Some(w.id) == txid.copied())
            })
            .unwrap();
        assert!(matches!(operation.state, AllocatedState::Amount(amt) if amt.as_u64() == amount));
    }

    fn _construct_psbt_offchain(
        &mut self,
        input_outpoints: Vec<(Outpoint, u64, Terminal, ScriptPubkey)>,
        beneficiaries: Vec<&PsbtBeneficiary>,
        tx_params: TxParams,
    ) -> (Psbt, PsbtMeta) {
        let mut psbt = Psbt::create(PsbtVer::V2);

        for (outpoint, value, terminal, spk) in input_outpoints {
            psbt.construct_input_expect(
                Prevout::new(outpoint, Sats::from(value)),
                self.descriptor(),
                terminal,
                spk,
                tx_params.seq_no,
            );
        }
        if psbt.inputs().count() == 0 {
            panic!("no inputs");
        }

        let input_value = psbt.input_sum();
        let mut max = Vec::new();
        let mut output_value = Sats::ZERO;
        for beneficiary in beneficiaries {
            let amount = beneficiary.amount.unwrap_or(Sats::ZERO);
            output_value.checked_add_assign(amount).unwrap();
            let out = psbt.construct_output_expect(beneficiary.script_pubkey(), amount);
            if beneficiary.amount.is_max() {
                max.push(out.index());
            }
        }
        let mut remaining_value = input_value
            .checked_sub(output_value)
            .unwrap()
            .checked_sub(tx_params.fee)
            .unwrap();
        if !max.is_empty() {
            let portion = remaining_value / max.len();
            for out in psbt.outputs_mut() {
                if max.contains(&out.index()) {
                    out.amount = portion;
                }
            }
            remaining_value = Sats::ZERO;
        }

        let (change_vout, change_terminal) = if remaining_value > Sats::from(546u64) {
            let change_index =
                self.get_next_index(tx_params.change_keychain, tx_params.change_shift);
            let change_terminal = Terminal::new(tx_params.change_keychain, change_index);
            let change_vout = psbt
                .construct_change_expect(self.descriptor(), change_terminal, remaining_value)
                .index();
            (
                Some(Vout::from_u32(change_vout as u32)),
                Some(change_terminal),
            )
        } else {
            (None, None)
        };

        (
            psbt,
            PsbtMeta {
                change_vout,
                change_terminal,
            },
        )
    }

    fn _construct_beneficiaries(
        &self,
        beneficiaries: Vec<(Address, Option<u64>)>,
    ) -> Vec<PsbtBeneficiary> {
        beneficiaries
            .into_iter()
            .map(|(addr, amt)| {
                let payment = if let Some(amt) = amt {
                    Payment::Fixed(Sats::from_sats(amt))
                } else {
                    Payment::Max
                };
                PsbtBeneficiary::new(addr, payment)
            })
            .collect()
    }

    pub fn construct_psbt_offchain(
        &mut self,
        input_outpoints: Vec<(Outpoint, u64, Terminal, ScriptPubkey)>,
        beneficiaries: Vec<(Address, Option<u64>)>,
        fee: Option<u64>,
    ) -> (Psbt, PsbtMeta) {
        let tx_params = TxParams::with(Sats::from_sats(fee.unwrap_or(DEFAULT_FEE_ABS)));
        let beneficiaries = self._construct_beneficiaries(beneficiaries);
        let beneficiaries: Vec<&PsbtBeneficiary> = beneficiaries.iter().collect();

        self._construct_psbt_offchain(input_outpoints, beneficiaries, tx_params)
    }

    pub fn construct_psbt(
        &mut self,
        input_outpoints: Vec<Outpoint>,
        beneficiaries: Vec<(Address, Option<u64>)>,
        fee: Option<u64>,
    ) -> (Psbt, PsbtMeta) {
        let tx_params = TxParams::with(Sats::from_sats(fee.unwrap_or(DEFAULT_FEE_ABS)));
        let beneficiaries = self._construct_beneficiaries(beneficiaries);
        let beneficiaries: Vec<&PsbtBeneficiary> = beneficiaries.iter().collect();

        self.wallet
            .wallet_mut()
            .construct_psbt(input_outpoints, beneficiaries, tx_params)
            .unwrap()
    }

    pub fn psbt_add_input(&self, psbt: &mut Psbt, utxo: Outpoint) {
        for account in self.descriptor().xpubs() {
            psbt.xpubs.insert(*account.xpub(), account.origin().clone());
        }
        let (input, spk) = self.utxo(&utxo);
        psbt.construct_input_expect(
            input.to_prevout(),
            self.descriptor(),
            input.terminal,
            spk,
            SeqNo::ZERO,
        );
    }

    pub fn color_psbt(
        &self,
        psbt: &mut Psbt,
        coloring_info: ColoringInfo,
    ) -> (Fascia, AssetBeneficiariesMap) {
        let asset_beneficiaries = self.color_psbt_init(psbt, coloring_info);
        psbt.set_rgb_close_method(CloseMethod::OpretFirst);
        psbt.complete_construction();
        let fascia = psbt.rgb_commit().unwrap();
        (fascia, asset_beneficiaries)
    }

    pub fn color_psbt_init(
        &self,
        psbt: &mut Psbt,
        coloring_info: ColoringInfo,
    ) -> AssetBeneficiariesMap {
        if !psbt.outputs().any(|o| o.script.is_op_return()) {
            let _output = psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO);
        }

        let prev_outputs = psbt
            .to_unsigned_tx()
            .inputs
            .iter()
            .map(|txin| txin.prev_output)
            .collect::<HashSet<Outpoint>>();

        let mut all_transitions: HashMap<ContractId, Transition> = HashMap::new();
        let mut asset_beneficiaries: AssetBeneficiariesMap = bmap![];

        for (contract_id, asset_coloring_info) in coloring_info.asset_info_map.clone() {
            let asset_schema = self.asset_schema(contract_id);
            let contract = self.wallet.stock().contract_data(contract_id).unwrap();
            let assignment_types = contract
                .schema
                .assignment_types_for_state(asset_schema.default_state_type());
            let assignment_type = assignment_types[0];
            let transition_type = contract
                .schema
                .default_transition_for_assignment(assignment_type);
            let mut asset_transition_builder = self
                .wallet
                .stock()
                .transition_builder_raw(contract_id, transition_type)
                .unwrap();

            let mut asset_available_amt = 0;
            for (_, opout_state_map) in self
                .wallet
                .stock()
                .contract_assignments_for(
                    contract_id,
                    prev_outputs
                        .iter()
                        // only retrieve assignments for owned prevouts using coloring_info
                        .filter(|op| {
                            coloring_info.asset_info_map[&contract_id]
                                .input_outpoints
                                .contains(op)
                        })
                        .copied(),
                )
                .unwrap()
            {
                for (opout, state) in opout_state_map {
                    if let AllocatedState::Amount(amt) = &state {
                        asset_available_amt += amt.as_u64();
                    }
                    asset_transition_builder =
                        asset_transition_builder.add_input(opout, state).unwrap();
                }
            }

            let mut beneficiaries = vec![];
            let mut sending_amt = 0;
            for (vout, amount) in asset_coloring_info.output_map {
                if amount == 0 {
                    continue;
                }
                sending_amt += amount;
                if vout as usize > psbt.outputs().count() {
                    panic!("invalid vout in output_map, does not exist in the given PSBT");
                }
                let graph_seal = if let Some(blinding) = asset_coloring_info.static_blinding {
                    GraphSeal::with_blinded_vout(vout, blinding)
                } else {
                    GraphSeal::new_random_vout(vout)
                };
                let seal = BuilderSeal::Revealed(graph_seal);
                beneficiaries.push(seal);

                asset_transition_builder = asset_transition_builder
                    .add_owned_state_raw(
                        *assignment_type,
                        seal,
                        asset_schema.allocated_state(amount),
                    )
                    .unwrap();
            }
            if sending_amt > asset_available_amt {
                panic!("total amount in output_map greater than available ({asset_available_amt})");
            }

            if let Some(nonce) = coloring_info.nonce {
                asset_transition_builder = asset_transition_builder.set_nonce(nonce);
            }

            let transition = asset_transition_builder.complete_transition().unwrap();
            all_transitions.insert(contract_id, transition);
            asset_beneficiaries.insert(contract_id, beneficiaries);
        }

        let (opreturn_index, _) = psbt
            .to_unsigned_tx()
            .outputs
            .iter()
            .enumerate()
            .find(|(_, o)| o.script_pubkey.is_op_return())
            .expect("psbt should have an op_return output");
        let (_, opreturn_output) = psbt
            .outputs_mut()
            .enumerate()
            .find(|(i, _)| i == &opreturn_index)
            .unwrap();
        opreturn_output.set_opret_host().unwrap();
        if let Some(blinding) = coloring_info.static_blinding {
            opreturn_output.set_mpc_entropy(blinding).unwrap();
        }

        let tx_inputs = psbt.clone().to_unsigned_tx().inputs;
        for (contract_id, transition) in all_transitions {
            for (input, txin) in psbt.inputs_mut().zip(&tx_inputs) {
                let prevout = txin.prev_output;
                let outpoint = Outpoint::new(prevout.txid.to_byte_array().into(), prevout.vout);
                if coloring_info
                    .asset_info_map
                    .clone()
                    .get(&contract_id)
                    .unwrap()
                    .input_outpoints
                    .contains(&outpoint)
                {
                    input
                        .set_rgb_consumer(contract_id, transition.id())
                        .unwrap();
                }
            }
            psbt.push_rgb_transition(transition).unwrap();
        }

        asset_beneficiaries
    }

    pub fn consume_fascia(&mut self, fascia: Fascia, witness_id: Txid) {
        struct FasciaResolver {
            witness_id: Txid,
        }
        impl ResolveWitness for FasciaResolver {
            fn resolve_pub_witness(&self, _: Txid) -> Result<Tx, WitnessResolverError> {
                unreachable!()
            }
            fn resolve_pub_witness_ord(
                &self,
                witness_id: Txid,
            ) -> Result<WitnessOrd, WitnessResolverError> {
                assert_eq!(witness_id, self.witness_id);
                Ok(WitnessOrd::Tentative)
            }
            fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
                unreachable!()
            }
        }

        let resolver = FasciaResolver { witness_id };

        self.consume_fascia_custom_resolver(fascia, resolver);
    }

    pub fn consume_fascia_custom_resolver(
        &mut self,
        fascia: Fascia,
        resolver: impl ResolveWitness,
    ) {
        self.wallet
            .stock_mut()
            .consume_fascia(fascia, resolver)
            .unwrap();
    }

    pub fn update_witnesses(&mut self, after_height: u32, force_witnesses: Vec<Txid>) {
        let resolver = self.get_resolver();
        self.wallet
            .stock_mut()
            .update_witnesses(resolver, after_height, force_witnesses)
            .unwrap();
    }

    pub fn get_outpoint_unsafe_history(
        &self,
        outpoint: Outpoint,
        safe_height: NonZeroU32,
    ) -> HashMap<ContractId, HashMap<u32, HashSet<Txid>>> {
        self.wallet
            .stock()
            .get_outpoint_unsafe_history(outpoint, safe_height)
            .unwrap()
    }

    pub fn create_consignments(
        &self,
        asset_beneficiaries: AssetBeneficiariesMap,
        witness_id: Txid,
    ) -> Vec<Transfer> {
        let mut transfers = vec![];
        let stock = self.wallet.stock();

        for (contract_id, beneficiaries) in asset_beneficiaries {
            let mut beneficiaries_witness = vec![];
            let mut beneficiaries_blinded = vec![];
            for beneficiary in beneficiaries {
                match beneficiary {
                    BuilderSeal::Revealed(seal) => {
                        let explicit_seal = ExplicitSeal::new(Outpoint::new(witness_id, seal.vout));
                        beneficiaries_witness.push(explicit_seal);
                    }
                    BuilderSeal::Concealed(secret_seal) => {
                        beneficiaries_blinded.push(secret_seal);
                    }
                }
            }
            transfers.push(
                stock
                    .transfer(
                        contract_id,
                        beneficiaries_witness,
                        beneficiaries_blinded,
                        Some(witness_id),
                    )
                    .unwrap(),
            )
        }
        transfers
    }
}
