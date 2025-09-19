use super::*;

pub const INDEXER_STOP_GAP: usize = 20;
pub const INDEXER_BATCH_SIZE: usize = 5;
pub const INDEXER_PARALLEL_REQUESTS: usize = 5;
pub const KEYCHAIN_EXTERNAL: u8 = 0;
pub const KEYCHAIN_INTERNAL: u8 = 1;

pub type BdkWalletImpl = PersistedWallet<Store<ChangeSet>>;
pub type BdkTestWallet = TestWallet<BdkWalletImpl, Store<ChangeSet>>;

pub enum BdkIndexer {
    Electrum(Box<BdkElectrumClient<ElectrumClient>>),
    Esplora(Box<EsploraClient>),
}

impl BdkIndexer {
    pub fn full_scan<K: Ord + Clone, R: Into<FullScanRequest<K>>>(
        &self,
        request: R,
    ) -> FullScanResponse<K> {
        match self {
            BdkIndexer::Electrum(client) => client
                .full_scan(request, INDEXER_STOP_GAP, INDEXER_BATCH_SIZE, true)
                .unwrap(),
            BdkIndexer::Esplora(client) => client
                .full_scan(request, INDEXER_STOP_GAP, INDEXER_PARALLEL_REQUESTS)
                .unwrap(),
        }
    }
}

impl BdkTestWallet {
    pub fn new(
        network: Network,
        wallet_dir: PathBuf,
        descriptor: String,
        change_descriptor: String,
    ) -> Self {
        std::fs::create_dir_all(&wallet_dir).unwrap();
        let db_path = wallet_dir.join("bdk.db");
        let mut db = Store::<ChangeSet>::create(b"RGB", db_path).unwrap();

        let bdk_wallet = BdkWallet::create(descriptor, change_descriptor)
            .network(network)
            .create_wallet(&mut db)
            .unwrap();

        let stock_path = wallet_dir.join("stock");
        let mut stock = Stock::in_memory();
        stock
            .make_persistent(FsBinStore::new(stock_path).unwrap(), true)
            .unwrap();

        let wallet = RgbWallet::new(stock, bdk_wallet);

        Self {
            wallet,
            aux: db,
            wallet_dir,
            instance: INSTANCE_1,
            network,
        }
    }

    pub fn with_descriptor(descriptor_type: &DescriptorType) -> Self {
        if descriptor_type != &DescriptorType::Wpkh {
            panic!("cannot use bdk in tapret mode");
        }
        let mut seed = vec![0u8; 128];
        rand::thread_rng().fill_bytes(&mut seed);

        let network = Network::Regtest;

        let xpriv = Xpriv::new_master(network, &seed).unwrap();
        let descriptor = format!(
            "wpkh({}/{PURPOSE_BIP84}'/{COIN_RGB_TESTNET}'/0'/{KEYCHAIN_EXTERNAL}/*)",
            xpriv
        );
        let change_descriptor = format!(
            "wpkh({}/{PURPOSE_BIP84}'/{COIN_RGB_TESTNET}'/0'/{KEYCHAIN_INTERNAL}/*)",
            xpriv
        );

        let wallet_dir = PathBuf::from(TEST_DATA_DIR)
            .join(INTEGRATION_DATA_DIR)
            .join(xpriv.fingerprint(&Secp256k1::new()).to_string());
        Self::new(network, wallet_dir, descriptor, change_descriptor)
    }

    pub fn keychain(&self, keychain_kind: KeychainKind) -> u8 {
        match keychain_kind {
            KeychainKind::External => KEYCHAIN_EXTERNAL,
            KeychainKind::Internal => KEYCHAIN_INTERNAL,
        }
    }
}

impl TestWalletExt for BdkTestWallet {
    type Psbt = Psbt;
    type PsbtMeta = PsbtMeta;
    type Outpoint = Outpoint;

    fn get_derived_address(&mut self) -> DerivedAddr {
        let addr_info = self
            .wallet
            .wallet_mut()
            .reveal_next_address(KeychainKind::External);
        self.wallet.wallet_mut().persist(&mut self.aux).unwrap();
        DerivedAddr::new(
            address_bitcoin_to_bp(addr_info.address),
            Keychain::from(KEYCHAIN_EXTERNAL),
            NormalIndex::try_from_index(addr_info.index).unwrap(),
        )
    }

    fn sync(&mut self) {
        let indexer_url = self.indexer_url();
        let client = match get_indexer_client(&indexer_url) {
            IndexerClient::Electrum(client) => {
                BdkIndexer::Electrum(Box::new(BdkElectrumClient::new(*client)))
            }
            IndexerClient::Esplora(client) => BdkIndexer::Esplora(client),
        };
        let request = self.wallet.wallet().start_full_scan().build();
        let update: Update = client.full_scan(request).into();
        self.wallet.wallet_mut().apply_update(update).unwrap();
        self.wallet.wallet_mut().persist(&mut self.aux).unwrap();
    }

    fn sign_finalize(&self, psbt: &mut Self::Psbt) {
        self.wallet
            .wallet()
            .sign(psbt, SignOptions::default())
            .unwrap();
    }

    fn extract(&self, psbt: &Self::Psbt) -> Tx {
        tx_bitcoin_to_bp(psbt.clone().extract_tx().unwrap())
    }

    fn tap_address(&mut self) -> (BpAddress, InternalPk, NormalIndex) {
        todo!("wait for taproot tweaks support")
    }

    fn pay(
        &mut self,
        invoice: RgbInvoice,
        params: TransferParams,
    ) -> (Self::Psbt, Self::PsbtMeta, Transfer) {
        self.wallet
            .pay::<ProprietaryKey, BdkOutput>(&invoice, params)
            .unwrap()
    }

    fn list_coins(&self) -> HashMap<(Address, Terminal), Vec<Coin>> {
        self.wallet
            .wallet()
            .list_output()
            .map(|o| {
                let address = Address::from_script(&o.txout.script_pubkey, self.network).unwrap();
                let terminal = Terminal::new(self.keychain(o.keychain), o.derivation_index);
                let coin = Coin {
                    height: match o.chain_position {
                        ChainPosition::Confirmed { anchor, .. } => anchor.block_id.height.into(),
                        ChainPosition::Unconfirmed { .. } => 0,
                    },
                    amount: o.txout.value.to_sat(),
                    outpoint: o.outpoint,
                };
                ((address, terminal), coin)
            })
            .fold(HashMap::new(), |mut map, (address, coin)| {
                map.entry(address).or_insert_with(Vec::new).push(coin);
                map
            })
    }

    fn list_unspents(&self) -> HashMap<Outpoint, u64> {
        self.wallet
            .wallet()
            .list_unspent()
            .map(|o| (o.outpoint, o.txout.value.to_sat()))
            .collect()
    }

    fn balance(&self) -> u64 {
        self.wallet.wallet().balance().total().to_sat()
    }
}
