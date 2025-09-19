use super::*;

mod bdk;
mod bp;

pub use bdk::*;
pub use bp::*;

pub enum AllocationFilter {
    Stock,
    Wallet,
    WalletAll,
    WalletTentative,
}

impl AllocationFilter {
    pub fn filter_for<W: WalletProvider, D>(self, wlt: &TestWallet<W, D>) -> Filter<'_, W> {
        match self {
            Self::WalletAll => Filter::WalletAll(&wlt.wallet),
            Self::WalletTentative => Filter::WalletTentative(&wlt.wallet),
            Self::Wallet => Filter::Wallet(&wlt.wallet),
            Self::Stock => Filter::NoWallet,
        }
    }
}

pub enum Filter<'w, W: WalletProvider> {
    NoWallet,
    Wallet(&'w RgbWallet<W>),
    WalletAll(&'w RgbWallet<W>),
    WalletTentative(&'w RgbWallet<W>),
}

impl<W: WalletProvider> AssignmentsFilter for Filter<'_, W> {
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
impl<W: WalletProvider> Filter<'_, W> {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DescriptorType {
    Wpkh,
    Tr,
}

impl fmt::Display for DescriptorType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
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
        write!(f, "{}", format!("{self:?}").to_lowercase())
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WitnessInfo {
    pub derived_address: DerivedAddr,
    pub tap_internal_key: Option<InternalPk>,
    pub amount_sats: Option<u64>,
}

impl WitnessInfo {
    pub fn btc_beneficiary(&self) -> (BpAddress, Option<u64>) {
        (self.address(), self.amount_sats)
    }

    pub fn address(&self) -> BpAddress {
        self.derived_address.addr
    }

    pub fn script_pubkey(&self) -> ScriptPubkey {
        self.address().script_pubkey()
    }

    pub fn terminal(&self) -> Terminal {
        self.derived_address.terminal.into()
    }
}

pub struct Coin {
    pub height: u64,
    pub amount: u64,
    pub outpoint: Outpoint,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, From)]
pub enum AssetDestination {
    #[from]
    Blinded(SecretSeal),
    #[from]
    Witness(WitnessInfo),
}

impl AssetDestination {
    fn define_seal(
        &self,
        vout: Option<u32>,
        static_blinding: Option<u64>,
    ) -> BuilderSeal<BlindSeal<TxPtr>> {
        match self {
            AssetDestination::Blinded(secret_seal) => BuilderSeal::Concealed(*secret_seal),
            AssetDestination::Witness(_witness_info) => {
                let vout = vout.expect("must be provided in this case");
                let graph_seal = if let Some(blinding) = static_blinding {
                    GraphSeal::with_blinded_vout(vout, blinding)
                } else {
                    GraphSeal::new_random_vout(vout)
                };
                BuilderSeal::Revealed(graph_seal)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssetAssignment {
    pub destination: AssetDestination,
    pub amount: u64,
}

/// RGB asset-specific information to color a transaction
#[derive(Clone, Debug)]
pub struct AssetColoringInfo {
    /// Input outpoints of the assets being spent
    pub input_outpoints: Vec<Outpoint>,
    /// Information to construct RGB assignments
    pub assignments: Vec<AssetAssignment>,
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
    /// Close method to use for the transaction
    pub close_method: CloseMethod,
}

impl ColoringInfo {
    pub fn first_tweakable_beneficiary(&self) -> Option<WitnessInfo> {
        // this assumes PSBT outputs have been added with the same order of the ColoringInfo one
        for asset_coloring_info in self.asset_info_map.values() {
            for asset_assignment in &asset_coloring_info.assignments {
                if let AssetDestination::Witness(ref witness_info) = asset_assignment.destination
                    && witness_info.tap_internal_key.is_some()
                    && witness_info.script_pubkey().is_p2tr()
                {
                    return Some(witness_info.clone());
                }
            }
        }
        None
    }
}

/// Map of contract ID and list of its beneficiaries
pub type AssetBeneficiariesMap = BTreeMap<ContractId, Vec<BuilderSeal<GraphSeal>>>;

/// Map of contract IDs and their consignments
type ConsignmentsMap = HashMap<ContractId, Transfer>;

/// Info needed to add a tapret tweak to a terminal
type TweakInfo = (WitnessInfo, TapretCommitment);

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
        write!(f, "{}", format!("{self:?}").to_lowercase())
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

    pub fn types(&self) -> TypeSystem {
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

    pub fn default_state_type(&self) -> StateType {
        match self {
            Self::Cfa | Self::Nia | Self::Pfa | Self::Ifa => StateType::Fungible,
            Self::Uda => StateType::Structured,
        }
    }

    pub fn allocated_state(&self, value: u64) -> AllocatedState {
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
        pubkey: CompressedPublicKey,
    },
    Ifa {
        spec: AssetSpec,
        terms: ContractTerms,
        reject_list_url: Option<RejectListUrl>,
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

    pub fn issued_amt(&self) -> u64 {
        match self {
            Self::Nia { issue_amounts, .. }
            | Self::Cfa { issue_amounts, .. }
            | Self::Pfa { issue_amounts, .. }
            | Self::Ifa { issue_amounts, .. } => issue_amounts.iter().sum(),
            Self::Uda { .. } => 1,
        }
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

    pub fn default_pfa(issue_amounts: Vec<u64>, pubkey: CompressedPublicKey) -> Self {
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
            Some(REJECT_LIST_URL),
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
        pubkey: CompressedPublicKey,
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
        reject_list_url: Option<&str>,
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
            reject_list_url: reject_list_url
                .map(|u| RejectListUrl::try_from(u.to_owned()).unwrap()),
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
                reject_list_url,
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
                if let Some(reject_list_url) = reject_list_url {
                    builder = builder
                        .add_global_state("rejectListUrl", reject_list_url.clone())
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

    pub fn write_displayable(&self, content: impl Display) {
        let mut file = OpenOptions::new()
            .append(true)
            .open(&self.report_path)
            .unwrap();
        file.write_all(format!("{content};").as_bytes()).unwrap();
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

fn get_bp_indexer(indexer_url: &str) -> AnyIndexer {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => {
            AnyIndexer::Electrum(Box::new(BpElectrumClient::new(indexer_url).unwrap()))
        }
        Indexer::Esplora => {
            AnyIndexer::Esplora(Box::new(BpEsploraClient::new_esplora(indexer_url).unwrap()))
        }
    }
}

fn get_resolver(indexer_url: &str) -> AnyResolver {
    match INDEXER.get().unwrap() {
        Indexer::Electrum => AnyResolver::electrum_blocking(indexer_url, None).unwrap(),
        Indexer::Esplora => {
            AnyResolver::esplora_blocking(EsploraBuilder::new(indexer_url)).unwrap()
        }
    }
}

fn broadcast_tx(tx: &Tx, indexer_url: &str) {
    match get_bp_indexer(indexer_url) {
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

pub struct TestWallet<W: WalletProvider, D> {
    pub wallet: RgbWallet<W>,
    aux: D,
    wallet_dir: PathBuf,
    instance: u8,
    network: Network,
}

pub trait TestWalletExt {
    type Psbt;
    type PsbtMeta;
    type Outpoint;

    fn get_derived_address(&mut self) -> DerivedAddr;

    fn sync(&mut self);

    fn sign_finalize(&self, psbt: &mut Self::Psbt);

    fn extract(&self, psbt: &Self::Psbt) -> Tx;

    fn sign_finalize_extract(&self, psbt: &mut Self::Psbt) -> Tx {
        self.sign_finalize(psbt);
        self.extract(psbt)
    }

    fn tap_address(&mut self) -> (BpAddress, InternalPk, NormalIndex);

    fn pay(
        &mut self,
        invoice: RgbInvoice,
        params: TransferParams,
    ) -> (Self::Psbt, Self::PsbtMeta, Transfer);

    fn list_coins(&self) -> HashMap<(Address, Terminal), Vec<Coin>>;

    fn list_unspents(&self) -> HashMap<Outpoint, u64>;

    fn list_unspent_outpoints(&self) -> Vec<Outpoint> {
        self.list_unspents().into_keys().collect()
    }

    fn balance(&self) -> u64;
}

impl<W: WalletProvider, D> TestWallet<W, D>
where
    Self: TestWalletExt,
    <Self as TestWalletExt>::Psbt: Serialize,
{
    pub fn network(&self) -> Network {
        self.network
    }

    pub fn chain_net(&self) -> ChainNet {
        match self.network() {
            Network::Bitcoin => ChainNet::BitcoinMainnet,
            Network::Regtest => ChainNet::BitcoinRegtest,
            Network::Signet => ChainNet::BitcoinSignet,
            Network::Testnet => ChainNet::BitcoinTestnet3,
            Network::Testnet4 => ChainNet::BitcoinTestnet4,
        }
    }

    pub fn testnet(&self) -> bool {
        self.network() != Network::Bitcoin
    }

    pub fn get_address(&mut self) -> BpAddress {
        self.get_derived_address().addr
    }

    pub fn get_utxo(&mut self, sats: Option<u64>) -> Outpoint {
        let address = self.get_address();
        let txid = Txid::from_str(&fund_wallet(address.to_string(), sats, self.instance)).unwrap();
        self.sync();
        let mut vout = None;
        let utxos = self.list_unspent_outpoints();
        assert!(!utxos.is_empty());
        for utxo in utxos {
            if utxo.txid == txid {
                vout = Some(utxo.vout);
                break;
            }
        }
        Outpoint {
            txid,
            vout: vout.unwrap(),
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

    pub fn get_resolver(&self) -> AnyResolver {
        get_resolver(&self.indexer_url())
    }

    pub fn broadcast_tx(&self, tx: &Tx) {
        broadcast_tx(tx, &self.indexer_url());
    }

    pub fn get_witness_ord(&self, txid: &Txid) -> WitnessOrd {
        self.get_resolver()
            .resolve_witness(*txid)
            .unwrap()
            .witness_ord()
    }

    pub fn get_tx_height(&self, txid: &Txid) -> Option<u32> {
        match self.get_witness_ord(txid) {
            WitnessOrd::Mined(witness_pos) => Some(witness_pos.height().get()),
            _ => None,
        }
    }

    pub fn close_method(&self) -> CloseMethod {
        self.wallet.wallet().close_method()
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

    pub fn stock(&self) -> &Stock {
        self.wallet.stock()
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
        let outpoints = if asset_info.issued_amt() == 0 {
            vec![]
        } else if outpoints.is_empty() {
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
        pubkey: CompressedPublicKey,
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
        let beneficiary = match invoice_type.into() {
            InvoiceType::Blinded(outpoint) => {
                Beneficiary::BlindedSeal(self.get_secret_seal(outpoint, None))
            }
            InvoiceType::Witness => {
                let address = self.get_address();
                let address_payload =
                    address_payload_bitcoin_from_script_pubkey(&address.payload.script_pubkey());
                Beneficiary::WitnessVout(Pay2Vout::new(address_payload), None)
            }
            InvoiceType::WitnessTapret => {
                let (address, tap_internal_key, _) = self.tap_address();
                let address_payload =
                    address_payload_bitcoin_from_script_pubkey(&address.payload.script_pubkey());
                let tap_internal_key = internal_pk_to_untweakedpublickey(tap_internal_key);
                Beneficiary::WitnessVout(Pay2Vout::new(address_payload), Some(tap_internal_key))
            }
        };

        let mut builder = RgbInvoiceBuilder::new(XChainNet::bitcoin(self.network(), beneficiary))
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

    pub fn consign_transfer(
        &self,
        contract_id: ContractId,
        outputs: impl AsRef<[OutputSeal]>,
        secret_seals: impl AsRef<[SecretSeal]>,
        opids: impl IntoIterator<Item = OpId>,
        witness_id: Option<BpTxid>,
    ) -> Transfer {
        let witness_id = witness_id.map(txid_bp_to_bitcoin);
        self.wallet
            .stock()
            .transfer(contract_id, outputs, secret_seals, opids, witness_id)
            .unwrap()
    }

    pub fn pay_invoice(
        &mut self,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
    ) -> (
        <Self as TestWalletExt>::Psbt,
        <Self as TestWalletExt>::PsbtMeta,
        Transfer,
    ) {
        let fee = fee.unwrap_or(DEFAULT_FEE_ABS);
        let sats = sats.unwrap_or(2000);
        let params = TransferParams::with(fee, sats);
        self.pay(invoice, params)
    }

    pub fn pay_full(
        &mut self,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
        broadcast: bool,
        report: Option<&Report>,
    ) -> (
        Transfer,
        Tx,
        <Self as TestWalletExt>::Psbt,
        <Self as TestWalletExt>::PsbtMeta,
    ) {
        self.sync();

        let pay_start = Instant::now();
        let (mut psbt, psbt_meta, consignment) = self.pay_invoice(invoice, sats, fee);
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

    pub fn accept_transfer(&mut self, consignment: Transfer, report: Option<&Report>) -> Status {
        let resolver = self.get_resolver();
        self.accept_transfer_custom(consignment, report, &resolver, bset![])
    }

    pub fn accept_transfer_custom(
        &mut self,
        consignment: Transfer,
        report: Option<&Report>,
        resolver: &impl ResolveWitness,
        trusted_op_seals: BTreeSet<OpId>,
    ) -> Status {
        self.sync();
        let validate_start = Instant::now();
        let validated_consignment = consignment
            .clone()
            .validate(
                &resolver,
                &ValidationConfig {
                    chain_net: self.chain_net(),
                    trusted_typesystem: AssetSchema::from(consignment.schema_id()).types(),
                    trusted_op_seals,
                    build_opouts_dag: true,
                    ..Default::default()
                },
            )
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
        validation_status
    }

    pub fn add_tapret_tweak(&mut self, terminal: Terminal, tapret_commitment: TapretCommitment) {
        self.wallet
            .wallet_mut()
            .add_tapret_tweak(terminal, tapret_commitment)
            .unwrap();
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

    pub fn get_contract_balance(&self, contract_id: ContractId) -> u64 {
        let asset_schema = self
            .stock()
            .contract_data(contract_id)
            .unwrap()
            .schema
            .schema_id()
            .into();
        match asset_schema {
            AssetSchema::Nia | AssetSchema::Cfa | AssetSchema::Ifa | AssetSchema::Pfa => {
                // balance can overflow with show_tentative=true
                let allocations = self.contract_fungible_allocations(contract_id, false);
                let mut balance = 0;
                for a in allocations {
                    let outpoint = a.seal.outpoint().unwrap();
                    let amount = a.state;
                    if self.list_unspent_outpoints().contains(&outpoint) {
                        balance += amount.value();
                    }
                }
                balance
            }
            AssetSchema::Uda => {
                unimplemented!("todo");
            }
        }
    }

    pub fn history(&self, contract_id: ContractId) -> Vec<ContractOp> {
        self.wallet.history(contract_id).unwrap()
    }

    pub fn list_contracts(&self) -> Vec<ContractInfo> {
        self.wallet.stock().contracts().unwrap().collect()
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
        for ((address, terminal), coins) in self.list_coins() {
            println!("{}\t{}", address, terminal);
            for coin in coins {
                println!(
                    "{}\t{: >12}\t{:68}",
                    coin.height, coin.amount, coin.outpoint
                );
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

    pub fn contract_assignments_for(
        &self,
        contract_id: ContractId,
        outpoints: impl IntoIterator<Item = impl Into<Outpoint>>,
    ) -> ContractAssignments {
        self.stock()
            .contract_assignments_for(contract_id, outpoints)
            .unwrap()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn send<W2: WalletProvider, D2>(
        &mut self,
        recv_wlt: &mut TestWallet<W2, D2>,
        invoice_type: impl Into<InvoiceType>,
        contract_id: ContractId,
        amount: u64,
        sats: u64,
        report: Option<&Report>,
    ) -> (Transfer, Tx)
    where
        TestWallet<W2, D2>: TestWalletExt,
        <TestWallet<W2, D2> as TestWalletExt>::Psbt: Serialize,
    {
        let schema_id = self.schema_id(contract_id);
        let invoice = recv_wlt.invoice(contract_id, schema_id, amount, invoice_type.into());
        self.send_to_invoice(recv_wlt, invoice, Some(sats), None, report)
    }

    pub fn send_to_invoice<W2: WalletProvider, D2>(
        &mut self,
        recv_wlt: &mut TestWallet<W2, D2>,
        invoice: RgbInvoice,
        sats: Option<u64>,
        fee: Option<u64>,
        report: Option<&Report>,
    ) -> (Transfer, Tx)
    where
        TestWallet<W2, D2>: TestWalletExt,
        <TestWallet<W2, D2> as TestWalletExt>::Psbt: Serialize,
    {
        let (consignment, tx, _, _) = self.pay_full(invoice, sats, fee, true, report);
        self.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
        recv_wlt.accept_transfer(consignment.clone(), report);
        self.sync();
        (consignment, tx)
    }

    pub fn send_ifa<W2: WalletProvider, D2>(
        &mut self,
        recv_wlt: &mut TestWallet<W2, D2>,
        invoice_type: impl Into<InvoiceType>,
        contract_id: ContractId,
        amount: u64,
    ) -> (Transfer, Tx)
    where
        TestWallet<W2, D2>: TestWalletExt,
        <TestWallet<W2, D2> as TestWalletExt>::Psbt: Serialize,
    {
        let schema_id = self.schema_id(contract_id);
        let invoice = recv_wlt.invoice(contract_id, schema_id, amount, invoice_type.into());
        self.send_ifa_to_invoice(recv_wlt, invoice)
    }

    pub fn send_ifa_to_invoice<W2: WalletProvider, D2>(
        &mut self,
        recv_wlt: &mut TestWallet<W2, D2>,
        invoice: RgbInvoice,
    ) -> (Transfer, Tx)
    where
        TestWallet<W2, D2>: TestWalletExt,
        <TestWallet<W2, D2> as TestWalletExt>::Psbt: Serialize,
    {
        let (consignment, tx, _, _) = self.pay_full(invoice, None, None, true, None);
        let txid = tx.txid();
        self.mine_tx(&txid_bp_to_bitcoin(txid), false);
        let trusted_op_seals = consignment.replace_transitions_input_ops();
        recv_wlt.accept_transfer_custom(
            consignment.clone(),
            None,
            &recv_wlt.get_resolver(),
            trusted_op_seals,
        );
        self.sync();
        (consignment, tx)
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
        txid: Option<&BpTxid>,
        direction: OpDirection,
        amount: u64,
    ) {
        let txid = txid.map(|txid| txid_bp_to_bitcoin(*txid));
        let operation = self
            .history(*contract_id)
            .into_iter()
            .find(|co| co.direction == direction && co.witness.is_none_or(|w| Some(w.id) == txid))
            .unwrap();
        assert!(matches!(operation.state, AllocatedState::Amount(amt) if amt.as_u64() == amount));
    }

    pub fn consume_fascia(&mut self, fascia: Fascia, witness_id: BpTxid) {
        let witness_id = txid_bp_to_bitcoin(witness_id);
        struct FasciaResolver {
            witness_id: Txid,
        }
        impl WitnessOrdProvider for FasciaResolver {
            fn witness_ord(&self, witness_id: Txid) -> Result<WitnessOrd, WitnessResolverError> {
                assert_eq!(witness_id, self.witness_id);
                Ok(WitnessOrd::Tentative)
            }
        }

        let resolver = FasciaResolver { witness_id };

        self.consume_fascia_custom_resolver(fascia, resolver);
    }

    pub fn consume_fascia_custom_resolver(
        &mut self,
        fascia: Fascia,
        witness_provider: impl WitnessOrdProvider,
    ) {
        self.wallet
            .stock_mut()
            .consume_fascia(fascia, witness_provider)
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
        witness_id: BpTxid,
    ) -> ConsignmentsMap {
        let witness_id = txid_bp_to_bitcoin(witness_id);
        let mut consignments_map = ConsignmentsMap::new();
        let stock = self.wallet.stock();

        for (contract_id, beneficiaries) in asset_beneficiaries {
            let mut beneficiaries_witness = vec![];
            let mut beneficiaries_blinded = vec![];
            for beneficiary in beneficiaries {
                match beneficiary {
                    BuilderSeal::Revealed(seal) => {
                        let explicit_seal = ExplicitSeal::with(witness_id, seal.vout);
                        beneficiaries_witness.push(explicit_seal);
                    }
                    BuilderSeal::Concealed(secret_seal) => {
                        beneficiaries_blinded.push(secret_seal);
                    }
                }
            }
            consignments_map.insert(
                contract_id,
                stock
                    .transfer(
                        contract_id,
                        beneficiaries_witness,
                        beneficiaries_blinded,
                        [],
                        Some(witness_id),
                    )
                    .unwrap(),
            );
        }
        consignments_map
    }

    pub fn reveal_fascia(
        &mut self,
        fascia: Fascia,
        contract_filter: &[ContractId],
    ) -> Option<Fascia> {
        let seal_witness = fascia.seal_witness.clone();
        let mut revealed_bundles = BTreeMap::new();
        for (cid, bundle) in fascia.into_bundles() {
            if !contract_filter.contains(&cid) {
                continue;
            }
            let mut revealed_bundle = bundle.clone();
            revealed_bundle
                .known_transitions
                .iter_mut()
                .flat_map(|t| t.transition.assignments.values_mut())
                .for_each(|a| {
                    if let TypedAssigns::Fungible(assignments) = a.clone() {
                        for assignment in assignments {
                            if let Some(seal) = self
                                .wallet
                                .stock()
                                .as_stash_provider()
                                .seal_secret(assignment.to_confidential_seal())
                                .unwrap()
                            {
                                a.reveal_seal(seal)
                            }
                        }
                    }
                });
            if revealed_bundle != bundle {
                revealed_bundles.insert(cid, revealed_bundle);
            }
        }
        if !revealed_bundles.is_empty() {
            let revealed_fascia = Fascia {
                seal_witness,
                bundles: NonEmptyOrdMap::from_checked(revealed_bundles),
            };
            return Some(revealed_fascia);
        }
        None
    }
}
