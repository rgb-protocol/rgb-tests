use super::*;

pub type BpWalletImpl = BpWallet<XpubDerivable, RgbDescr<XpubDerivable>>;
pub type BpTestWallet = TestWallet<BpWalletImpl, Option<TestnetSigner>>;

pub enum WalletAccount {
    Private(XprivAccount),
    Public(XpubAccount),
}

impl BpTestWallet {
    pub fn new(
        descriptor_type: Option<&DescriptorType>,
        network: BpNetwork,
        wallet_dir: PathBuf,
        wallet_account: WalletAccount,
        instance: u8,
        import_kits: bool,
        keychains: Vec<Keychain>,
    ) -> Self {
        std::fs::create_dir_all(&wallet_dir).unwrap();
        let name = "bp_wallet_name";
        let bp_dir = wallet_dir.join(name);
        let mut wallet = if let Some(dt) = descriptor_type {
            // new wallet
            let xpub_account = match wallet_account {
                WalletAccount::Private(ref xpriv_account) => xpriv_account.to_xpub_account(),
                WalletAccount::Public(ref xpub_account) => xpub_account.clone(),
            };
            let xpub_derivable = XpubDerivable::try_custom(
                *xpub_account.xpub(),
                xpub_account.origin().clone(),
                keychains,
            )
            .unwrap();
            let descriptor = match dt {
                DescriptorType::Wpkh => RgbDescr::Wpkh(WpkhDescr::with_key(xpub_derivable)),
                DescriptorType::Tr => RgbDescr::TapretKey(TapretKey::with_key(xpub_derivable)),
            };
            let mut bp_wallet = BpWallet::new_layer1(descriptor.clone(), network);
            let bp_wallet_provider = FsTextStore::new(bp_dir).unwrap();
            bp_wallet.make_persistent(bp_wallet_provider, true).unwrap();
            bp_wallet.set_name(name.to_string());
            let mut stock = Stock::in_memory();
            let stock_provider = FsBinStore::new(wallet_dir.clone()).unwrap();
            stock.make_persistent(stock_provider, true).unwrap();
            RgbWallet::new(stock, bp_wallet)
        } else {
            // load wallet
            RgbWallet::load(wallet_dir.clone(), bp_dir, true).unwrap()
        };
        println!(
            "wallet dir: {wallet_dir:?} ({})",
            if wallet.wallet().is_taproot() {
                "tapret"
            } else {
                "opret"
            }
        );

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

        let mut wallet = Self {
            wallet,
            aux: signer,
            wallet_dir,
            instance,
            network: network_bp_to_bitcoin(network),
        };

        wallet.sync();

        wallet
    }

    pub fn with_descriptor(descriptor_type: &DescriptorType) -> Self {
        Self::with(descriptor_type, None, true)
    }

    pub fn with(descriptor_type: &DescriptorType, instance: Option<u8>, import_kits: bool) -> Self {
        Self::with_rng(descriptor_type, instance, import_kits, None).0
    }

    pub fn gen_keys(seed: &[u8]) -> (XprivAccount, PathBuf) {
        let xpriv_account = XprivAccount::with_seed(true, seed).derive([
            HardenedIndex::try_from_child_number(PURPOSE_BIP86).unwrap(),
            HardenedIndex::try_from_child_number(COIN_RGB_TESTNET).unwrap(),
            HardenedIndex::try_from_child_number(ACCOUNT).unwrap(),
        ]);

        let fingerprint = xpriv_account.account_fp().to_string();
        let wallet_dir = PathBuf::from(TEST_DATA_DIR)
            .join(INTEGRATION_DATA_DIR)
            .join(fingerprint);
        (xpriv_account, wallet_dir)
    }

    pub fn with_rng(
        descriptor_type: &DescriptorType,
        instance: Option<u8>,
        import_kits: bool,
        rng: Option<&mut StdRng>,
    ) -> (Self, Vec<u8>) {
        let mut seed = vec![0u8; 128];
        if let Some(rng) = rng {
            rng.fill_bytes(&mut seed);
        } else {
            rand::thread_rng().fill_bytes(&mut seed);
        }

        let (xpriv_account, wallet_dir) = Self::gen_keys(&seed);

        let wallet = Self::new(
            Some(descriptor_type),
            BpNetwork::Regtest,
            wallet_dir,
            WalletAccount::Private(xpriv_account),
            instance.unwrap_or(INSTANCE_1),
            import_kits,
            vec![Keychain::OUTER, Keychain::INNER],
        );
        (wallet, seed)
    }

    pub fn new_mainnet() -> Self {
        let xpub_account = XpubAccount::from_str(
            "[c32338a7/86h/0h/0h]xpub6CmiK1xc7YwL472qm4zxeURFX8yMCSasioXujBjVMMzA3AKZr6KLQEmkzDge1Ezn2p43ZUysyx6gfajFVVnhtQ1AwbXEHrioLioXXgj2xW5"
        ).unwrap();

        let wallet_dir = PathBuf::from(TEST_DATA_DIR)
            .join(INTEGRATION_DATA_DIR)
            .join("mainnet");

        Self::new(
            Some(&DescriptorType::Wpkh),
            BpNetwork::Mainnet,
            wallet_dir,
            WalletAccount::Public(xpub_account),
            INSTANCE_1,
            true,
            vec![Keychain::from(9)],
        )
    }

    pub fn bp_network(&self) -> BpNetwork {
        self.wallet.wallet().network()
    }

    pub fn keychain(&self) -> Keychain {
        self.wallet.wallet().default_keychain()
    }

    fn get_next_index(&mut self, keychain: impl Into<Keychain>, shift: bool) -> NormalIndex {
        self.wallet
            .wallet_mut()
            .next_derivation_index(keychain, shift)
    }

    pub fn utxo(&self, outpoint: &BpOutpoint) -> (Utxo, ScriptPubkey) {
        self.wallet.wallet().utxo(*outpoint).unwrap()
    }

    pub fn descriptor(&self) -> &RgbDescr<XpubDerivable> {
        self.wallet.wallet().descriptor()
    }

    fn get_next_internal_pk(&mut self) -> (InternalPk, NormalIndex) {
        let keychain = self.keychain();
        let index = self.get_next_index(keychain, true);
        let descr = self.descriptor();
        let internal_pk = descr
            .derive(keychain, index)
            .next()
            .unwrap()
            .to_internal_pk()
            .expect("not a taproot wallet");
        (internal_pk, index)
    }

    pub fn send_pfa<W2: WalletProvider, D2>(
        &mut self,
        recv_wlt: &mut TestWallet<W2, D2>,
        transfer_type: TransferType,
        contract_id: ContractId,
        amount: u64,
        secret_key: SecretKey,
    ) where
        TestWallet<W2, D2>: TestWalletExt,
        <TestWallet<W2, D2> as TestWalletExt>::Psbt: Serialize,
    {
        let transition_signer = |witness_bundle: &mut WitnessBundle| {
            for KnownTransition { opid, transition } in
                witness_bundle.bundle_mut().known_transitions.iter_mut()
            {
                let transition_id: [u8; 32] = opid.as_ref().into_inner();
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
        let bp_txid = tx.txid();
        let txid = txid_bp_to_bitcoin(bp_txid);
        consignment.modify_bundle(txid, transition_signer);
        self.accept_transfer(consignment.clone(), None);
        let output_seal: OutputSeal =
            ExplicitSeal::new(Outpoint::new(txid, psbt_meta.change_vout.unwrap()));
        for cid in psbt.rgb_contract_ids().unwrap() {
            if cid == contract_id {
                continue;
            }
            let mut extra_cons = self.consign_transfer(cid, [output_seal], [], [], Some(bp_txid));
            let changed = extra_cons.modify_bundle(txid, transition_signer);
            assert!(changed);
            self.accept_transfer(extra_cons.clone(), None);
        }
        self.mine_tx(&txid, false);
        recv_wlt.accept_transfer(consignment.clone(), None);
        self.sync();
    }

    pub fn inflate_ifa(
        &mut self,
        contract_id: ContractId,
        inflation_outpoints: Vec<Outpoint>,
        inflation_amounts: Vec<u64>,
    ) -> Tx {
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
            .map(|txin| outpoint_bp_to_bitcoin(txin.previous_outpoint))
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
        psbt.push_rgb_transition(transition).unwrap();
        psbt.set_opret_host();
        psbt.set_rgb_close_method(CloseMethod::OpretFirst);
        psbt.set_as_unmodifiable();
        let fascia = psbt.rgb_commit().unwrap();
        let txid = psbt.txid();
        self.consume_fascia(fascia, txid);
        let tx = self.sign_finalize_extract(&mut psbt);
        self.broadcast_tx(&tx);
        self.mine_tx(&psbt.get_txid(), false);
        println!("inflation txid: {}", txid);
        self.sync();
        let consignment_map = self.create_consignments(bmap![contract_id => beneficiaries], txid);
        for consignment in consignment_map.values() {
            consignment
                .clone()
                .validate(
                    &self.get_resolver(),
                    &ValidationConfig {
                        chain_net: self.chain_net(),
                        trusted_typesystem: AssetSchema::from(consignment.schema_id()).types(),
                        ..Default::default()
                    },
                )
                .unwrap();
        }
        tx
    }

    pub fn psbt_add_input(&self, psbt: &mut BpPsbt, utxo: BpOutpoint) {
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

    pub fn replace_ifa(
        &mut self,
        right_owner: &mut BpTestWallet,
        right_utxo: Outpoint,
        contract_id: ContractId,
    ) -> Tx {
        let address = self.get_address();
        let allocations = self.contract_fungible_allocations(contract_id, false);
        let replaced_amount: u64 = allocations.iter().map(|a| a.state.value()).sum();
        let utxos = allocations
            .iter()
            .map(|a| a.seal.to_outpoint())
            .collect::<Vec<_>>();
        let (mut psbt, _) = self.construct_psbt(utxos, vec![(address, None)], None);
        let right_utxo = outpoint_bitcoin_to_bp(right_utxo);
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
            .map(|txin| outpoint_bp_to_bitcoin(txin.previous_outpoint))
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
        psbt.push_rgb_transition(transition).unwrap();
        psbt.set_opret_host();
        psbt.set_rgb_close_method(CloseMethod::OpretFirst);
        psbt.set_as_unmodifiable();
        let fascia = psbt.rgb_commit().unwrap();
        let txid = psbt.txid();
        self.consume_fascia(fascia.clone(), txid);
        right_owner.consume_fascia(fascia, txid);
        right_owner.sign_finalize(&mut psbt);
        let tx = self.sign_finalize_extract(&mut psbt);
        self.broadcast_tx(&tx);
        self.mine_tx(&psbt.get_txid(), false);
        println!("replace txid: {}", txid);
        self.sync();
        right_owner.sync();

        let consignment_map = self.create_consignments(bmap![contract_id => beneficiaries], txid);
        for consignment in consignment_map.values() {
            let trusted_op_seals = consignment.replace_transitions_input_ops();
            let validated_consignment = consignment
                .clone()
                .validate(
                    &self.get_resolver(),
                    &ValidationConfig {
                        chain_net: self.chain_net(),
                        trusted_typesystem: AssetSchema::from(consignment.schema_id()).types(),
                        trusted_op_seals,
                        ..Default::default()
                    },
                )
                .unwrap();
            let resolver = right_owner.get_resolver();
            right_owner
                .wallet
                .stock_mut()
                .accept_transfer(validated_consignment.clone(), &resolver)
                .unwrap();
        }
        tx
    }

    pub fn burn_ifa(&mut self, contract_id: ContractId, utxos: Vec<Outpoint>) -> (Transfer, Tx) {
        let address = self.get_address();
        let (mut psbt, _) = self.construct_psbt(utxos, vec![(address, None)], None);
        let mut asset_transition_builder = self
            .wallet
            .stock()
            .transition_builder(contract_id, "burn")
            .unwrap();
        let prev_outputs = psbt
            .inputs()
            .map(|txin| outpoint_bp_to_bitcoin(txin.previous_outpoint))
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
        let opid = transition.id();
        psbt.push_rgb_transition(transition).unwrap();
        psbt.set_opret_host();
        psbt.set_rgb_close_method(CloseMethod::OpretFirst);
        psbt.set_as_unmodifiable();
        let fascia = psbt.rgb_commit().unwrap();
        let txid = psbt.txid();
        self.consume_fascia(fascia, txid);
        let tx = self.sign_finalize_extract(&mut psbt);
        self.broadcast_tx(&tx);
        self.mine_tx(&psbt.get_txid(), false);
        println!("burn txid: {}", txid);
        self.sync();
        let consignment = self.consign_transfer(contract_id, [], [], [opid], Some(txid));
        (consignment, tx)
    }

    fn _construct_psbt_offchain(
        &mut self,
        input_outpoints: Vec<(BpOutpoint, u64, Terminal, ScriptPubkey)>,
        beneficiaries: Vec<&PsbtBeneficiary>,
        tx_params: TxParams,
    ) -> (BpPsbt, BpPsbtMeta) {
        let mut psbt = BpPsbt::create(PsbtVer::V2);

        for (outpoint, value, terminal, spk) in input_outpoints {
            psbt.construct_input_expect(
                Prevout::new(outpoint, Sats::from(value)),
                self.descriptor(),
                terminal.into(),
                spk,
                SeqNo::from_consensus_u32(tx_params.seq_no),
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
            .checked_sub(tx_params.fee_sats)
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
            let change_terminal = BpTerminal::new(tx_params.change_keychain, change_index);
            let change_vout = psbt
                .construct_change_expect(self.descriptor(), change_terminal, remaining_value)
                .vout();
            (Some(change_vout), Some(change_terminal))
        } else {
            (None, None)
        };

        (
            psbt,
            BpPsbtMeta {
                change_vout,
                change_terminal,
            },
        )
    }

    fn _construct_beneficiaries(
        &self,
        beneficiaries: Vec<(BpAddress, Option<u64>)>,
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
        input_outpoints: Vec<(BpOutpoint, u64, Terminal, ScriptPubkey)>,
        beneficiaries: Vec<(BpAddress, Option<u64>)>,
        fee: Option<u64>,
    ) -> (BpPsbt, BpPsbtMeta) {
        let tx_params = TxParams::with(fee.unwrap_or(DEFAULT_FEE_ABS));
        let beneficiaries = self._construct_beneficiaries(beneficiaries);
        let beneficiaries: Vec<&PsbtBeneficiary> = beneficiaries.iter().collect();

        self._construct_psbt_offchain(input_outpoints, beneficiaries, tx_params)
    }

    pub fn construct_psbt(
        &mut self,
        input_outpoints: impl IntoIterator<Item = Outpoint>,
        beneficiaries: Vec<(BpAddress, Option<u64>)>,
        fee: Option<u64>,
    ) -> (BpPsbt, BpPsbtMeta) {
        let tx_params = TxParams::with(fee.unwrap_or(DEFAULT_FEE_ABS));
        let beneficiaries = self._construct_beneficiaries(beneficiaries);
        let beneficiaries: Vec<&PsbtBeneficiary> = beneficiaries.iter().collect();
        let input_outpoints = input_outpoints
            .into_iter()
            .map(outpoint_bitcoin_to_bp)
            .collect::<Vec<_>>();

        self.wallet
            .wallet_mut()
            .construct_psbt(input_outpoints, beneficiaries, tx_params.into())
            .unwrap()
    }

    pub fn get_witness_info(
        &mut self,
        amount_sats: Option<u64>,
        close_method: Option<CloseMethod>,
    ) -> WitnessInfo {
        match close_method.unwrap_or_else(|| self.close_method()) {
            CloseMethod::OpretFirst => WitnessInfo {
                derived_address: self.get_derived_address(),
                tap_internal_key: None,
                amount_sats,
            },
            CloseMethod::TapretFirst => {
                let (address, tap_internal_key, index) = self.tap_address();
                let derived_address = DerivedAddr::new(address, self.keychain(), index);
                WitnessInfo {
                    derived_address,
                    tap_internal_key: Some(tap_internal_key),
                    amount_sats,
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn get_change_seal(
        &mut self,
        psbt: &BpPsbt,
        psbt_meta: &BpPsbtMeta,
        change_utxo_option: &mut Option<Outpoint>,
        blinding_factor: Option<u64>,
        contract_id: ContractId,
        blinded_to_self: &mut Vec<ContractId>,
    ) -> BuilderSeal<GraphSeal> {
        let destination = match (*change_utxo_option, psbt_meta.change_vout) {
            (Some(change_utxo), _) => self
                .get_secret_seal(Some(change_utxo), blinding_factor)
                .into(),
            (None, Some(change_vout)) => {
                let output = psbt.outputs().find(|o| o.vout() == change_vout).unwrap();
                let addr = BpAddress::with(&output.script, self.bp_network()).unwrap();
                AssetDestination::Witness(WitnessInfo {
                    derived_address: DerivedAddr {
                        addr,
                        terminal: psbt_meta.change_terminal.unwrap(),
                    },
                    tap_internal_key: None,
                    amount_sats: Some(output.amount.sats()),
                })
            }
            (None, None) => {
                let change_utxo = self.get_utxo(None);
                *change_utxo_option = Some(change_utxo);
                self.get_secret_seal(Some(change_utxo), blinding_factor)
                    .into()
            }
        };
        let seal = destination.define_seal(psbt_meta.change_vout.map(|v| v.to_u32()), None);
        if matches!(destination, AssetDestination::Blinded(_)) {
            blinded_to_self.push(contract_id);
        }
        seal
    }

    pub fn color_psbt(
        &mut self,
        psbt: &mut BpPsbt,
        psbt_meta: &mut BpPsbtMeta,
        coloring_info: ColoringInfo,
        rgb_change: Option<Outpoint>,
    ) -> (
        Fascia,
        AssetBeneficiariesMap,
        Vec<ContractId>,
        Option<WitnessInfo>,
    ) {
        let (asset_beneficiaries, blinded_to_self, tweaked_witness_info) =
            self.color_psbt_init(psbt, psbt_meta, coloring_info, rgb_change);
        psbt.set_as_unmodifiable();
        let fascia = psbt.rgb_commit().unwrap();
        (
            fascia,
            asset_beneficiaries,
            blinded_to_self,
            tweaked_witness_info,
        )
    }

    pub fn color_psbt_init(
        &mut self,
        psbt: &mut BpPsbt,
        psbt_meta: &mut BpPsbtMeta,
        coloring_info: ColoringInfo,
        mut rgb_change: Option<Outpoint>,
    ) -> (AssetBeneficiariesMap, Vec<ContractId>, Option<WitnessInfo>) {
        let change_script = psbt_meta
            .change_vout
            .and_then(|vout| psbt.output(vout.to_u32() as usize))
            .map(|output| output.script.clone());
        let mut tweaked_witness_info = None;
        let mut close_method = coloring_info.close_method;
        let mut scripts_map: HashMap<ScriptPubkey, usize> = HashMap::new();
        if close_method == CloseMethod::TapretFirst {
            let tap_out_script = if let Some(change_script) = change_script.clone() {
                psbt.set_rgb_tapret_host_on_change();
                Some(change_script)
            } else if let Some(witness_info) = coloring_info.first_tweakable_beneficiary() {
                tweaked_witness_info = Some(witness_info.clone());
                Some(witness_info.script_pubkey())
            } else {
                None
            };
            if let Some(tap_out_script) = tap_out_script {
                let output = psbt
                    .outputs_mut()
                    .find(|o| o.script == tap_out_script)
                    .unwrap();
                output.set_tapret_host();
                psbt.sort_outputs_by(|output| !output.is_tapret_host())
                    .unwrap();
            } else {
                close_method = CloseMethod::OpretFirst;
            }
        }
        if close_method == CloseMethod::OpretFirst {
            let output = {
                let output_opt = psbt.outputs_mut().find(|o| o.script.is_op_return());
                match output_opt {
                    Some(o) => o,
                    None => psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO),
                }
            };
            output.set_opret_host();
            psbt.sort_outputs_by(|output| !output.is_opret_host())
                .unwrap();
        }
        if let Some(ref change_script) = change_script {
            for output in psbt.outputs() {
                if output.script == *change_script {
                    psbt_meta.change_vout = Some(output.vout());
                    break;
                }
            }
        }

        // set MPC entropy on commitment output
        let commitment_output = psbt
            .outputs_mut()
            .find(|o| o.script.is_p2tr() || o.script.is_op_return())
            .expect("just created");
        if let Some(blinding) = coloring_info.static_blinding {
            commitment_output.set_mpc_entropy(blinding).unwrap();
        }

        let prev_outputs = psbt
            .inputs()
            .map(|txin| outpoint_bp_to_bitcoin(txin.previous_outpoint))
            .collect::<HashSet<Outpoint>>();

        let mut all_transitions: HashMap<ContractId, Vec<Transition>> = HashMap::new();
        let mut asset_beneficiaries: AssetBeneficiariesMap = bmap![];
        let mut blinded_to_self: Vec<ContractId> = vec![];

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

            let mut outpoints = vec![];
            let mut asset_available_amt = 0;
            for (explicit_seal, opout_state_map) in self
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
                                .contains(*op)
                        })
                        .copied(),
                )
                .unwrap()
            {
                for (opout, state) in opout_state_map {
                    if let AllocatedState::Amount(amt) = &state {
                        asset_available_amt += amt.as_u64();
                    }
                    outpoints.push(explicit_seal.to_outpoint());
                    asset_transition_builder =
                        asset_transition_builder.add_input(opout, state).unwrap();
                }
            }

            let mut beneficiaries = vec![];
            let mut sending_amt = 0;
            for assignment in asset_coloring_info.assignments {
                if assignment.amount == 0 {
                    continue;
                }

                sending_amt += assignment.amount;

                let destination = assignment.destination.clone();
                let vout = if let AssetDestination::Witness(ref witness_info) = destination {
                    let script = witness_info.script_pubkey();
                    // support address reuse by selecting the appropriate vouts
                    let vouts: Vec<u32> = psbt
                        .outputs()
                        .filter(|o| o.script == script)
                        .map(|o| o.vout().to_u32())
                        .collect();
                    let n = *scripts_map.entry(script.clone()).or_insert(0) + 1;
                    let vout = vouts[n - 1];
                    Some(vout)
                } else {
                    None
                };
                let seal = destination.define_seal(vout, coloring_info.static_blinding);
                beneficiaries.push(seal);

                asset_transition_builder = asset_transition_builder
                    .add_owned_state_raw(
                        *assignment_type,
                        seal,
                        asset_schema.allocated_state(assignment.amount),
                    )
                    .unwrap();

                if let AssetDestination::Witness(witness_info) = assignment.destination {
                    psbt.output_mut(vout.unwrap() as usize)
                        .unwrap()
                        .tap_internal_key = witness_info.tap_internal_key;
                }
            }
            if sending_amt > asset_available_amt {
                panic!("total amount in output_map greater than available ({asset_available_amt})");
            }

            if let Some(nonce) = coloring_info.nonce {
                asset_transition_builder = asset_transition_builder.set_nonce(nonce);
            }

            let change_amt = asset_available_amt - sending_amt;
            if change_amt > 0 {
                let seal = self.get_change_seal(
                    psbt,
                    psbt_meta,
                    &mut rgb_change,
                    coloring_info.static_blinding,
                    contract_id,
                    &mut blinded_to_self,
                );
                asset_transition_builder = asset_transition_builder
                    .add_owned_state_raw(
                        *assignment_type,
                        seal,
                        asset_schema.allocated_state(change_amt),
                    )
                    .unwrap();
            }

            let transition = asset_transition_builder.complete_transition().unwrap();
            all_transitions
                .entry(contract_id)
                .or_default()
                .push(transition.clone());
            psbt.push_rgb_transition(transition).unwrap();
            asset_beneficiaries.insert(contract_id, beneficiaries);
        }

        let mut extra_state =
            HashMap::<ContractId, HashMap<OutputSeal, HashMap<Opout, AllocatedState>>>::new();
        let previous_outpoints = prev_outputs.into_iter().collect::<Vec<_>>();
        for id in self
            .wallet
            .stock()
            .contracts_assigning(previous_outpoints.iter().copied())
            .unwrap()
        {
            if coloring_info.asset_info_map.contains_key(&id) {
                continue;
            }
            let state = self
                .wallet
                .stock()
                .contract_assignments_for(id, previous_outpoints.iter().copied())
                .unwrap();
            let entry = extra_state.entry(id).or_default();
            for (seal, assigns) in state {
                entry.entry(seal).or_default().extend(assigns);
            }
        }

        // construct transitions for extra state
        for (cid, seal_map) in extra_state {
            let contract = self.wallet.stock().contract_data(cid).unwrap();
            let schema = contract.schema;

            for (_explicit_seal, assigns) in seal_map {
                for (opout, state) in assigns {
                    let transition_type = schema.default_transition_for_assignment(&opout.ty);
                    let mut extra_builder = self
                        .wallet
                        .stock()
                        .transition_builder_raw(cid, transition_type)
                        .unwrap();
                    let seal = self.get_change_seal(
                        psbt,
                        psbt_meta,
                        &mut rgb_change,
                        coloring_info.static_blinding,
                        cid,
                        &mut blinded_to_self,
                    );
                    extra_builder = extra_builder
                        .add_input(opout, state.clone())
                        .unwrap()
                        .add_owned_state_raw(opout.ty, seal, state)
                        .unwrap();
                    if !extra_builder.has_inputs() {
                        continue;
                    }
                    let extra_transition = extra_builder.complete_transition().unwrap();
                    all_transitions
                        .entry(cid)
                        .or_default()
                        .push(extra_transition.clone());
                    psbt.push_rgb_transition(extra_transition).unwrap();
                }
            }
        }

        for (cid, transitions) in &all_transitions {
            for transition in transitions {
                for opout in transition.inputs() {
                    psbt.set_rgb_contract_consumer(*cid, opout, transition.id())
                        .unwrap();
                }
            }
        }

        psbt.set_rgb_close_method(close_method);

        (asset_beneficiaries, blinded_to_self, tweaked_witness_info)
    }

    pub fn pay_full_flexible(
        &mut self,
        coloring_info: ColoringInfo,
        fee: Option<u64>,
        rgb_change: Option<Outpoint>,
    ) -> (ConsignmentsMap, Tx, BpPsbtMeta, Option<TweakInfo>) {
        let beneficiaries = coloring_info
            .asset_info_map
            .values()
            .flat_map(|c| c.assignments.clone())
            .filter_map(|a| match a.destination {
                AssetDestination::Witness(witness_info) => Some(witness_info.btc_beneficiary()),
                _ => None,
            })
            .collect::<Vec<_>>();
        let input_outpoints = coloring_info
            .asset_info_map
            .values()
            .flat_map(|c| c.input_outpoints.clone())
            .collect::<HashSet<_>>(); // remove duplicates
        let (mut psbt, mut psbt_meta) = self.construct_psbt(input_outpoints, beneficiaries, fee);

        let (fascia, rgb_beneficiaries, blinded_to_self, tweaked_witness_info) =
            self.color_psbt(&mut psbt, &mut psbt_meta, coloring_info, rgb_change);

        let mut tweak_info = None;
        if psbt.rgb_close_method().unwrap().unwrap() == CloseMethod::TapretFirst {
            let tweak_on_change = psbt.rgb_tapret_host_on_change();
            let tapret_op = psbt.dbc_output::<TapretProof>().unwrap();
            let tapret_commitment = tapret_op.tapret_commitment().unwrap();
            if tweak_on_change {
                assert_eq!(Some(tapret_op.vout()), psbt_meta.change_vout);
                let terminal = tapret_op.terminal_derivation().unwrap();
                self.add_tapret_tweak(terminal.into(), tapret_commitment);
            } else {
                tweak_info = Some((tweaked_witness_info.unwrap(), tapret_commitment))
            }
        }

        let tx = self.sign_finalize_extract(&mut psbt);

        self.broadcast_tx(&tx);
        let txid = tx.txid();
        self.consume_fascia(fascia.clone(), txid);
        let consignment_map = self.create_consignments(rgb_beneficiaries, txid);
        if !blinded_to_self.is_empty()
            && let Some(revealed_fascia) = self.reveal_fascia(fascia.clone(), &blinded_to_self)
        {
            self.consume_fascia(revealed_fascia, txid);
        }

        (consignment_map, tx, psbt_meta, tweak_info)
    }
}

impl TestWalletExt for BpTestWallet {
    type Psbt = BpPsbt;
    type PsbtMeta = PsbtMeta;
    type Outpoint = BpOutpoint;

    fn get_derived_address(&mut self) -> DerivedAddr {
        let keychain = self.keychain();
        let index = self.get_next_index(keychain, true);
        self.wallet
            .wallet()
            .addresses(keychain)
            .nth(index.index() as usize)
            .expect("address iterator always can produce address")
    }

    fn sync(&mut self) {
        let indexer = get_bp_indexer(&self.indexer_url());
        self.wallet
            .wallet_mut()
            .sync_from_scratch(&indexer)
            .into_result()
            .unwrap();
    }

    fn sign_finalize(&self, psbt: &mut Self::Psbt) {
        let _sig_count = psbt.sign(self.aux.as_ref().unwrap()).unwrap();
        psbt.finalize(self.descriptor());
    }

    fn extract(&self, psbt: &Self::Psbt) -> Tx {
        psbt.extract().unwrap()
    }

    fn tap_address(&mut self) -> (BpAddress, InternalPk, NormalIndex) {
        let (tap_internal_key, index) = self.get_next_internal_pk();
        let address = BpAddress::with(
            &ScriptPubkey::p2tr_key_only(tap_internal_key),
            self.bp_network(),
        )
        .unwrap();
        (address, tap_internal_key, index)
    }

    fn pay(
        &mut self,
        invoice: RgbInvoice,
        params: TransferParams,
    ) -> (Self::Psbt, Self::PsbtMeta, Transfer) {
        self.wallet
            .pay::<PropKey, Output>(&invoice, params)
            .unwrap()
    }

    fn list_coins(&self) -> HashMap<(Address, Terminal), Vec<Coin>> {
        self.wallet
            .wallet()
            .address_coins()
            .iter()
            .map(|(a, coins)| {
                (
                    (address_bp_to_bitcoin(a.addr), a.terminal.into()),
                    coins
                        .iter()
                        .map(|c| Coin {
                            height: match c.height {
                                TxStatus::Mined(info) => info.get().into(),
                                TxStatus::Mempool => 0,
                                TxStatus::Channel => 0,
                                TxStatus::Unknown => 0,
                            },
                            amount: c.amount.sats(),
                            outpoint: outpoint_bp_to_bitcoin(c.outpoint),
                        })
                        .collect(),
                )
            })
            .collect()
    }

    fn list_unspents(&self) -> HashMap<Outpoint, u64> {
        self.wallet
            .wallet()
            .address_coins()
            .values()
            .flatten()
            .map(|u| (outpoint_bp_to_bitcoin(u.outpoint), u.amount.sats()))
            .collect()
    }

    fn balance(&self) -> u64 {
        self.wallet.wallet().balance().0
    }
}
