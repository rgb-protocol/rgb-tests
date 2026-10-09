pub mod utils;

use utils::*;

#[derive(Clone)]
enum MockResolvePubWitness {
    Success(WitnessStatus),
    Error(WitnessResolverError),
}

#[derive(Clone)]
struct MockResolver {
    pub_witnesses: HashMap<Txid, MockResolvePubWitness>,
    check_chain_net_err: Option<WitnessResolverError>,
}

impl ResolveWitness for MockResolver {
    fn resolve_witness(&self, witness_id: Txid) -> Result<WitnessStatus, WitnessResolverError> {
        if let Some(res) = self.pub_witnesses.get(&witness_id) {
            match res {
                MockResolvePubWitness::Success(witness_status) => Ok(witness_status.clone()),
                MockResolvePubWitness::Error(err) => Err(err.clone()),
            }
        } else {
            Ok(WitnessStatus::Unresolved)
        }
    }

    fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
        self.check_chain_net_err.clone().map_or(Ok(()), Err)
    }
}
impl MockResolver {
    pub fn with_new_transaction(&self, witness: Transaction) -> Self {
        let mut resolver = self.clone();
        let witness_id = witness.compute_txid();
        resolver.pub_witnesses.insert(
            witness_id,
            MockResolvePubWitness::Success(WitnessStatus::Resolved(witness, WitnessOrd::Tentative)),
        );
        resolver
    }
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
enum Scenario {
    A,
    B,
    C,
    D,
    E,
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Scenario {
    fn txs_folder(&self) -> String {
        format!("tests/fixtures/txs_{self}/")
    }

    fn txs_folder_v0(&self) -> String {
        format!("tests/fixtures/v0/txs_{self}/")
    }

    fn resolver(&self) -> MockResolver {
        self.resolver_from(&self.txs_folder())
    }

    /// Whether this scenario has a v0 version
    fn has_v0(&self) -> bool {
        matches!(self, Scenario::A | Scenario::B | Scenario::D)
    }

    fn resolver_v0(&self) -> MockResolver {
        self.resolver_from(&self.txs_folder_v0())
    }

    fn resolver_from(&self, txs_folder: &str) -> MockResolver {
        let mut txs = map![];
        for entry in std::fs::read_dir(txs_folder).unwrap() {
            let file = std::fs::File::open(entry.unwrap().path()).unwrap();
            let tx: Transaction = serde_json::from_reader(file).unwrap();
            txs.insert(tx.compute_txid(), tx);
        }
        MockResolver {
            pub_witnesses: txs
                .into_iter()
                .map(|(txid, tx)| {
                    (
                        txid,
                        MockResolvePubWitness::Success(WitnessStatus::Resolved(
                            tx,
                            WitnessOrd::Mined(
                                // TODO: store actual values instead of the hardcoded WitnessPos
                                WitnessPos::bitcoin(NonZeroU32::new(106).unwrap(), 1726062111)
                                    .unwrap(),
                            ),
                        )),
                    )
                })
                .collect(),
            check_chain_net_err: None,
        }
    }
}

fn replace_transition_in_bundle(
    witness_bundle: &mut WitnessBundle,
    old_opid: OpId,
    transition: Transition,
) {
    let mut known_transitions = witness_bundle
        .bundle
        .known_transitions
        .clone()
        .into_iter()
        .filter(|kt| kt.opid != old_opid)
        .collect::<Vec<_>>();
    let transition_id = transition.id();
    known_transitions.push(KnownTransition::new(transition_id, transition.clone()));
    let input_map = witness_bundle
        .bundle
        .input_map
        .clone()
        .into_iter()
        .map(|(opout, opid)| {
            let new_opid = if opid == old_opid {
                transition_id
            } else {
                opid
            };
            (opout, new_opid)
        })
        .collect();
    let bundle = TransitionBundle {
        input_map: NonEmptyOrdMap::from_checked(input_map),
        known_transitions: NonEmptyVec::from_checked(known_transitions),
    };
    witness_bundle.bundle = bundle;
    update_anchor(witness_bundle, None)
}

/// Append a bundle carrying `transition` to a consignment.
fn append_bundle(
    consignment: &mut Transfer,
    resolver: &MockResolver,
    transition: Transition,
) -> MockResolver {
    let mut bundles = consignment.bundles.clone().release();
    let parent = bundles.last().unwrap();
    let opid = transition.id();
    let input_map = transition
        .inputs
        .iter()
        .map(|opout| (*opout, opid))
        .collect::<BTreeMap<_, _>>();
    let bundle = TransitionBundle {
        input_map: NonEmptyOrdMap::from_checked(input_map),
        known_transitions: NonEmptyVec::with(KnownTransition::new(opid, transition)),
    };
    let parent_txid = parent.tx.compute_txid();
    let tx = Transaction {
        input: (0..parent.tx.output.len() as u32)
            .map(|vout| TxIn {
                previous_output: bitcoin::OutPoint::new(parent_txid, vout),
                ..Default::default()
            })
            .collect(),
        ..parent.tx.clone()
    };
    let mut witness_bundle = WitnessBundle {
        tx,
        spv_proof: None,
        anchor: parent.anchor.clone(),
        bundle,
    };
    update_anchor(&mut witness_bundle, None);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    bundles.push(witness_bundle);
    consignment.bundles = LargeVec::from_checked(bundles);
    consignment.terminals = empty!(); // terminals are now outdated
    alt_resolver
}

fn update_anchor(witness_bundle: &mut WitnessBundle, contract_id: Option<ContractId>) {
    let contract_id = contract_id.unwrap_or(
        witness_bundle
            .bundle
            .known_transitions
            .last()
            .unwrap()
            .transition
            .contract_id,
    );
    let protocol_id = mpc::ProtocolId::from(contract_id);
    let message = mpc::Message::from(witness_bundle.bundle.bundle_id());
    let mut tx = witness_bundle.tx.clone();
    let idx = tx
        .output
        .iter()
        .enumerate()
        .find(|(_, o)| o.script_pubkey.is_op_return())
        .map(|(i, _)| i)
        .unwrap();
    tx.output[idx].script_pubkey = ScriptBuf::new_op_return([]);
    let mut witness_psbt = Psbt::from_unsigned_tx(tx).unwrap();
    witness_psbt.outputs.get_mut(idx).unwrap().set_opret_host();
    witness_psbt
        .outputs
        .get_mut(idx)
        .unwrap()
        .set_mpc_message(protocol_id, message)
        .unwrap();
    let (commitment, proof) = witness_psbt
        .outputs
        .get_mut(idx)
        .unwrap()
        .mpc_commit()
        .unwrap();
    witness_psbt
        .outputs
        .get_mut(idx)
        .unwrap()
        .opret_commit(commitment)
        .unwrap();
    witness_psbt.set_opret_commitment(idx);
    let witness = witness_psbt.unsigned_tx();

    let mut anchor = witness_bundle.anchor.clone();
    anchor.mpc_proof = proof.to_merkle_proof(protocol_id).unwrap();
    witness_bundle.tx = witness.clone();
    witness_bundle.anchor = anchor;
}

/// Update children bundles to keep consistency with some modified transitions
fn update_transition_children(
    witness_bundles: &mut Vec<WitnessBundle>,
    changed_opids: HashMap<OpId, OpId>,
    changed_txids: HashMap<Txid, Txid>,
    new_contract_id: Option<ContractId>,
) {
    let mut changed_opids = changed_opids;
    let mut changed_txids = changed_txids;
    let mut something_changed = false;
    for wbundle in witness_bundles.iter_mut() {
        let old_txid = wbundle.witness_id();
        if new_contract_id.is_none()
            && !changed_txids.contains_key(&old_txid)
            && !wbundle
                .bundle
                .input_map
                .keys()
                .any(|o| changed_opids.contains_key(&o.op))
        {
            continue; // ignore unrelated bundles
        }
        // map current opouts to their new value
        let opout_map = wbundle
            .bundle
            .input_map
            .keys()
            .map(|opout| {
                (
                    opout,
                    Opout {
                        op: *changed_opids.get(&opout.op).unwrap_or(&opout.op),
                        ..*opout
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        // update known transitions: change their input opouts
        wbundle.bundle.known_transitions =
            NonEmptyVec::from_iter_checked(wbundle.bundle.known_transitions.iter().map(
                |KnownTransition { opid, transition }| {
                    let new_transition = Transition {
                        inputs: NonEmptyOrdSet::from_iter_checked(
                            transition.inputs.iter().map(|i| *opout_map.get(i).unwrap()),
                        )
                        .into(),
                        contract_id: new_contract_id.unwrap_or_else(|| transition.contract_id()),
                        ..transition.clone()
                    };
                    let new_opid = new_transition.id();
                    if *opid != new_opid {
                        changed_opids.insert(*opid, new_opid);
                        something_changed = true;
                    }
                    KnownTransition {
                        opid: new_opid,
                        transition: new_transition,
                    }
                },
            ));
        // update input map: change input opouts and known transitions' opids
        wbundle.bundle.input_map = NonEmptyOrdMap::from_iter_checked(
            wbundle.bundle.input_map.iter().map(|(opout, opid)| {
                (
                    *opout_map.get(opout).unwrap(),
                    *changed_opids.get(opid).unwrap_or(opid),
                )
            }),
        );
        // update transition: change inputs according to modified txids
        let mut witness = wbundle.tx.clone();
        witness.input.iter_mut().for_each(|i| {
            let txid = &i.previous_output.txid;
            i.previous_output.txid = *changed_txids.get(txid).unwrap_or(txid);
        });
        wbundle.tx = witness;
        update_anchor(wbundle, None);
        if old_txid != wbundle.witness_id() {
            changed_txids.insert(old_txid, wbundle.witness_id());
            something_changed = true;
        }
    }
    if something_changed {
        update_transition_children(witness_bundles, changed_opids, changed_txids, None)
    }
}

/// Remove bundles that depend on some opids, optionally only the ones spending a given allocation type
fn remove_transition_children(
    witness_bundles: &mut Vec<WitnessBundle>,
    affected_opids: BTreeSet<OpId>,
    assignment_type: Option<AssignmentType>,
) {
    let mut removed_opids = bset![];
    witness_bundles.retain(|wbundle| {
        let delete = wbundle
            .bundle
            .input_map
            .keys()
            .filter(|o| assignment_type.is_none_or(|t| t == o.ty))
            .any(|o| affected_opids.contains(&o.op));
        if delete {
            // overkill, removing whole bundle for just one transition
            removed_opids.extend(wbundle.bundle.input_map.values());
        }
        !delete
    });
    if !removed_opids.is_empty() {
        remove_transition_children(witness_bundles, removed_opids, None);
    }
}

fn get_consignment(scenario: Scenario) -> (Transfer, Vec<Tx>) {
    initialize();
    if let Scenario::D = scenario {
        let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Tr);
        let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Tr);

        let issued_supply = 999;

        let sats = 9000;

        let utxo = wlt_1.get_utxo(None);
        let contract_id_1 = wlt_1.issue_nia(issued_supply, Some(&utxo));

        let mut txes = vec![];

        let (_consignment, tx) = wlt_1.send(
            &mut wlt_2,
            TransferType::Blinded,
            contract_id_1,
            666,
            sats,
            None,
        );
        txes.push(tx);
        let (consignment, tx) = wlt_2.send(
            &mut wlt_1,
            TransferType::Witness,
            contract_id_1,
            300,
            sats,
            None,
        );
        txes.push(tx);
        return (consignment, txes);
    }

    if let Scenario::E = scenario {
        let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

        let contract_id = wlt_1.issue_bfa();

        let mut txes = vec![];

        let mint_amt = 1000;
        let (_consignment, tx) = wlt_1.mint_bfa(contract_id, mint_amt, 20_000_000);
        txes.push(tx);

        // Self-transfer moving both OS_ASSET and OS_MINT forward
        let (consignment, tx) = wlt_1.bfa_transfer_with_mint_right(contract_id);
        txes.push(tx);

        return (consignment, txes);
    }

    let transfer_type = match scenario {
        Scenario::A => TransferType::Blinded,
        Scenario::B => TransferType::Witness,
        Scenario::C => TransferType::Witness,
        _ => unreachable!(),
    };

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_supply_1 = 999;
    let issued_supply_2 = 666;

    let sats = 9000;

    let utxo = wlt_1.get_utxo(None);
    let contract_id_1 = wlt_1.issue_nia(issued_supply_1, Some(&utxo));
    let contract_id_2 = match scenario {
        Scenario::C => wlt_1.issue_ifa(issued_supply_2, Some(&utxo), vec![(utxo, 100)]),
        _ => wlt_1.issue_nia(issued_supply_2, Some(&utxo)),
    };

    let mut txes = vec![];

    let (_consignment, tx) = wlt_1.send(&mut wlt_2, transfer_type, contract_id_1, 66, sats, None);
    txes.push(tx);

    if scenario == Scenario::C {
        // get inflation right out of the way
        let schema_id_2 = wlt_1.schema_id(contract_id_2);
        let mut invoice =
            wlt_1.invoice(contract_id_2, schema_id_2, 100, InvoiceType::Blinded(None));
        invoice.assignment_name = Some(FieldName::from_str("inflationAllowance").unwrap());
        let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
        wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
        wlt_1.accept_transfer(consignment, None);
        wlt_1.sync();
        txes.push(tx);
    }

    let (tx, next_amt) = if scenario == Scenario::C {
        // inflate asset using right that was moved automatically
        let contract = wlt_1.contract_wrapper::<InflatableFungibleAsset>(contract_id_2);
        let inflation_allocations = contract
            .inflation_allocations(AllocationFilter::Wallet.filter_for(&wlt_1))
            .map(|res| res.unwrap())
            .collect::<Vec<_>>();
        let inflation_outpoints = inflation_allocations
            .iter()
            .map(|oa| oa.seal.outpoint().unwrap())
            .collect::<Vec<_>>();
        let tx = wlt_1.inflate_ifa(contract_id_2, inflation_outpoints, vec![60]);
        let next_amt = issued_supply_2 + 5; //make sure we spend the new allocation
        (tx, next_amt)
    } else {
        // spend asset that was moved automatically
        let (_consignment, tx) =
            wlt_1.send(&mut wlt_2, transfer_type, contract_id_2, 50, sats, None);
        (tx, 77)
    };
    txes.push(tx);

    let (consignment, tx) = if scenario == Scenario::C {
        // burn all allocations
        let wlt_1_utxos = wlt_1.list_unspent_outpoints();
        wlt_1.burn_ifa(
            contract_id_2,
            wlt_1_utxos,
            Some(map! {OS_ASSET => next_amt}),
        )
    } else {
        // spend change of previous send
        wlt_1.send(
            &mut wlt_2,
            transfer_type,
            contract_id_2,
            next_amt,
            sats,
            None,
        )
    };
    txes.push(tx);
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap();

    (consignment, txes)
}

struct OfflineResolver<'cons, const TRANSFER: bool> {
    consignment: &'cons Consignment<TRANSFER>,
}
impl<const TRANSFER: bool> ResolveWitness for OfflineResolver<'_, TRANSFER> {
    fn resolve_witness(&self, witness_id: Txid) -> Result<WitnessStatus, WitnessResolverError> {
        self.consignment
            .bundled_witnesses()
            .find(|bw| bw.witness_id() == witness_id)
            .map(|p| p.tx.clone())
            .map_or_else(
                || Ok(WitnessStatus::Unresolved),
                |tx| Ok(WitnessStatus::Resolved(tx, WitnessOrd::Tentative)),
            )
    }
    fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
        Ok(())
    }
}

// run once to generate tests/fixtures/consignment_<scenario>.{json,rgb}
// for example:
// SCENARIO=B cargo test --test validation validate_consignment_generate -- --ignored --show-output
#[test]
#[ignore = "one-shot"]
fn validate_consignment_generate() {
    let scenario = match std::env::var("SCENARIO") {
        Ok(val) if val.to_uppercase() == Scenario::A.to_string() => Scenario::A,
        Ok(val) if val.to_uppercase() == Scenario::B.to_string() => Scenario::B,
        Ok(val) if val.to_uppercase() == Scenario::C.to_string() => Scenario::C,
        Ok(val) if val.to_uppercase() == Scenario::D.to_string() => Scenario::D,
        Ok(val) if val.to_uppercase() == Scenario::E.to_string() => Scenario::E,
        Err(VarError::NotPresent) => Scenario::A,
        _ => panic!("invalid scenario"),
    };
    let (consignment, txes) = get_consignment(scenario);
    println!();
    let cons_json_path = format!("tests/fixtures/consignment_{scenario}.json");
    let json = serde_json::to_string_pretty(&consignment).unwrap();
    std::fs::write(&cons_json_path, json).unwrap();
    println!("written consignment in: {cons_json_path}");
    let cons_strict_path = format!("tests/fixtures/consignment_{scenario}.rgb");
    consignment
        .strict_serialize_to_file::<{ usize::MAX }>(&cons_strict_path)
        .unwrap();
    println!("written consignment in: {cons_strict_path}");
    let _ = std::fs::remove_dir_all(scenario.txs_folder());
    std::fs::create_dir_all(scenario.txs_folder()).unwrap();
    for tx in txes {
        let tx = tx_bp_to_bitcoin(tx);
        let txid = tx.compute_txid().to_string();
        let json = serde_json::to_string_pretty(&tx).unwrap();
        let json_path = format!("{}/{txid}.json", scenario.txs_folder());
        std::fs::write(&json_path, json).unwrap();
        println!("written tx: {txid}");
    }
}

fn get_consignment_from_json(fname: &str) -> Transfer {
    let cons_path = format!("tests/fixtures/{fname}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let consignment: UncheckedTransfer = serde_json::from_reader(file).unwrap();
    consignment.into_checked().unwrap()
}

fn get_consignment_v0(scenario: Scenario, resolver: &impl ResolveWitness) -> Transfer {
    let cons_path = format!("tests/fixtures/v0/consignment_{scenario}.rgb");
    TransferV0::strict_deserialize_from_file::<{ usize::MAX }>(&cons_path)
        .unwrap()
        .into_v1(Some(resolver))
        .unwrap()
}

fn transfer_from_json_str(s: &str) -> Transfer {
    serde_json::from_str::<UncheckedTransfer>(s)
        .unwrap()
        .into_checked()
        .unwrap()
}

fn transfer_from_json_value(v: &serde_json::Value) -> Transfer {
    transfer_from_json_str(&serde_json::to_string(v).unwrap())
}

#[test]
fn validate_consignment_success() {
    for scenario in Scenario::iter() {
        println!(" ---- {scenario}");
        let resolver = scenario.resolver();
        let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
        assert_consignment_valid(&consignment, &resolver);
    }
}

#[test]
fn validate_consignment_success_v0() {
    for scenario in Scenario::iter().filter(Scenario::has_v0) {
        let resolver = scenario.resolver_v0();
        println!(" ---- {scenario}_v0");
        let consignment = get_consignment_v0(scenario, &resolver);
        assert_consignment_valid(&consignment, &resolver);
    }
}

fn assert_consignment_valid(consignment: &Transfer, resolver: &impl ResolveWitness) {
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    // BFA carries external anchors, which `validate` cannot discharge on its own
    let res = if matches!(asset_schema, AssetSchema::Bfa) {
        let mut pending_consignment = consignment
            .clone()
            .validate_deterministic(&asset_schema_rules, &validation_config)
            .unwrap();
        pending_consignment
            .resolve_all_anchors(
                BridgedFungibleAsset::bridge_location(pending_consignment.consignment().genesis())
                    .unwrap(),
                &MockAnchorResolver::new(BFA_CHAIN_ID, |_anchor| true),
            )
            .unwrap();
        pending_consignment.pending().resolve_all(resolver).unwrap();
        pending_consignment.finalize()
    } else {
        consignment
            .clone()
            .validate(&asset_schema_rules, resolver, &validation_config)
            .unwrap()
    };
    let validation_status = res.validation_status();
    dbg!(&validation_status);
    assert!(validation_status.warnings.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Valid);
    assert!(validation_status.dag_data_opt.is_none());
}

#[test]
fn consignment_data_reads_rules_from_the_store() {
    let scenario = Scenario::A;
    let resolver = scenario.resolver();
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let valid = consignment
        .validate(&asset_schema.schema_rules(), &resolver, &validation_config)
        .unwrap();

    let mut stock = SqliteStock::in_memory().unwrap();

    // the schema is unknown until its definition is imported
    assert!(stock.consignment_data(&valid).is_err());

    stock
        .import_schema_definition(asset_schema.schema_definition())
        .unwrap();
    let data = stock.consignment_data(&valid).unwrap();

    // the rules come back whole, so global state decodes
    assert_eq!(data.rules.schema_id(), valid.schema_id());
    assert!(!data.rules.types().is_empty());
    assert_eq!(data.contract_id(), valid.contract_id());
    let spec = data.global("spec").next().expect("spec global");
    dbg!(spec);
}

/// A schema definition ships strict type libraries, never the type system built
/// from them: the recipient derives every semantic id itself, so the ids the
/// schema commits to are what authenticate the type definitions.
#[test]
fn schema_definition_derives_the_type_system_from_its_libs() {
    for asset_schema in AssetSchema::iter() {
        let schema_def = asset_schema.schema_definition();
        assert!(
            !schema_def.libs.is_empty(),
            "a definition must carry its type libraries"
        );

        // deriving reproduces exactly the code-side type system
        let rules = schema_def.verify().expect("honest definition must verify");
        assert_eq!(rules.types(), &asset_schema.types());
        assert_eq!(rules.schema_id(), asset_schema.schema().schema_id());
    }
}

/// A stock keeps a definition's type libraries, not the type system derived
/// from them, so what it took in comes back out unchanged and the derivation is
/// redone - and re-authenticated against the schema - on the way.
#[test]
fn stock_exports_the_schema_definition_it_imported() {
    let mut stock = SqliteStock::in_memory().unwrap();
    for asset_schema in AssetSchema::iter() {
        let schema_def = asset_schema.schema_definition();
        let schema_id = schema_def.schema_id();
        stock
            .import_schema_definition(schema_def.clone())
            .expect("an honest definition must import");

        assert_eq!(
            stock.export_schema_definition(schema_id).unwrap(),
            schema_def,
            "a definition must survive the store whole"
        );
        // and the type system the store hands to the validator is the one the
        // libraries derive, not one it was given
        assert_eq!(
            stock.schema_rules(schema_id).unwrap().types(),
            &asset_schema.types()
        );
    }

    // sharing between schemata is preserved: every definition still exports its
    // own libraries, common ones included
    let shared = AssetSchema::Nia.schema_definition().libs;
    assert!(
        shared
            .keys()
            .all(|id| AssetSchema::Uda.schema_definition().libs.contains_key(id)),
        "the standard schemata must share their type libraries"
    );
}

/// The derivation is what a cold read does, not something only the importing
/// process has: a stock opened on a store it did not write rebuilds the rules
/// from the stored type libraries.
#[test]
fn reopened_stock_rederives_the_schema_rules() {
    let dir = std::env::temp_dir().join("rgb-tests-sdf-reopen");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("stock.db");

    let schema_def = AssetSchema::Nia.schema_definition();
    let schema_id = schema_def.schema_id();
    {
        let mut stock = SqliteStock::open(&path).unwrap();
        stock.import_schema_definition(schema_def.clone()).unwrap();
    }

    let stock = SqliteStock::open(&path).unwrap();
    assert_eq!(
        stock.export_schema_definition(schema_id).unwrap(),
        schema_def
    );
    assert_eq!(
        stock.schema_rules(schema_id).unwrap().types(),
        &AssetSchema::Nia.types()
    );

    std::fs::remove_dir_all(&dir).unwrap();
}

/// A schema which was never imported has no definition to export, and that is
/// the user not having imported it rather than the store being broken.
#[test]
fn exporting_an_unknown_schema_definition_fails() {
    let stock = SqliteStock::in_memory().unwrap();
    let schema_id = AssetSchema::Nia.schema().schema_id();
    assert!(matches!(
        stock.export_schema_definition(schema_id),
        Err(StockError::SchemaNotImported(id)) if id == schema_id
    ));
}

/// Type libraries are keyed by their own commitment, so a definition cannot
/// present a library under a foreign id.
#[test]
fn schema_definition_rejects_miskeyed_type_lib() {
    let schema_def = AssetSchema::Nia.schema_definition();
    let mut libs = schema_def.libs.clone().release();
    let (_, lib) = libs.pop_last().unwrap();
    let wrong_id = TypeLibId::from([0xADu8; 32]);
    libs.insert(wrong_id, lib.clone());
    let tampered = SchemaDefinition::new(
        schema_def.schema.clone(),
        TypeLibs::from_checked(libs),
        schema_def.scripts.clone(),
    );

    assert_eq!(
        tampered.verify().unwrap_err(),
        SchemaDefError::TypeLibIdMismatch(wrong_id, lib.id())
    );
}

/// Tampering with a type definition changes the semantic id derived from it, so
/// the type the schema commits to is simply no longer there.
#[test]
fn schema_definition_rejects_tampered_type_lib() {
    let schema_def = AssetSchema::Nia.schema_definition();

    // drop a field from a struct type, keeping the type's name
    let mut libs = schema_def.libs.clone().release();
    let mut tampered_name = None;
    for lib in libs.values_mut() {
        let types = lib.types.clone();
        for (name, ty) in types.iter() {
            if let Ty::Struct(fields) = ty
                && fields.len() > 1
            {
                let mut fields: Vec<_> = fields.iter().cloned().collect();
                fields.pop();
                let ty = Ty::Struct(NamedFields::try_from(fields).unwrap());
                lib.types.insert(name.clone(), ty).unwrap();
                tampered_name = Some(name.clone());
                break;
            }
        }
        if tampered_name.is_some() {
            break;
        }
    }
    assert!(tampered_name.is_some(), "no struct type to tamper with");
    // re-key the library by its new commitment, so only the content differs
    let libs = TypeLibs::from_iter_checked(libs.into_values().map(|lib| (lib.id(), lib)));
    let tampered =
        SchemaDefinition::new(schema_def.schema.clone(), libs, schema_def.scripts.clone());

    assert!(
        matches!(tampered.verify(), Err(SchemaDefError::TypeAbsent(_))),
        "a tampered type library must not verify"
    );

    // and a store refuses to take it
    let mut stock = SqliteStock::in_memory().unwrap();
    assert!(stock.import_schema_definition(tampered).is_err());
    assert!(stock.import_schema_definition(schema_def).is_ok());
}

/// A definition survives a file round trip whole, and the bumped magic keeps a
/// reader of the previous layout from misparsing it.
#[test]
fn schema_definition_file_round_trip() {
    let schema_def = AssetSchema::Nia.schema_definition();
    let dir = std::env::temp_dir().join("rgb-tests-sdf-round-trip");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("nia.rgb");
    schema_def.save_file(&path).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(&bytes[..7], b"RGB\0SD2", "the SDF magic must be bumped");

    let restored = SchemaDefinition::load_file(&path).unwrap();
    assert_eq!(restored, schema_def);
    assert_eq!(
        restored.verify().unwrap().types(),
        &AssetSchema::Nia.types()
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A definition must carry nothing the schema does not need, so that it is
/// exactly determined by the schema it defines.
#[test]
fn schema_definition_rejects_extraneous_type_lib() {
    let schema_def = AssetSchema::Nia.schema_definition();

    // a well-formed library the schema has no use for
    let extra = rgbcore::stl::rgb_contract_id_stl();
    assert!(
        !schema_def.libs.contains_key(&extra.id()),
        "the extra library must not already be needed"
    );
    let mut libs = schema_def.libs.clone().release();
    libs.insert(extra.id(), extra.clone());
    let bloated = SchemaDefinition::new(
        schema_def.schema.clone(),
        TypeLibs::from_checked(libs),
        schema_def.scripts.clone(),
    );

    assert_eq!(
        bloated.verify().unwrap_err(),
        SchemaDefError::TypeLibExtraneous(extra.id())
    );
}

/// AluVM libraries the schema cannot reach are rejected too - but reachability
/// is the closure of cross-library calls, not just the validators the schema
/// names, so a helper library a validator calls is kept.
#[test]
fn schema_definition_checks_script_reachability() {
    let schema_def = AssetSchema::Nia.schema_definition();

    // an AluVM library the schema never enters and nothing calls
    let helper = AssetSchema::Ifa.scripts().values().next().unwrap().clone();
    assert!(!schema_def.scripts.contains_key(&helper.id()));
    let mut scripts = schema_def.scripts.clone().release();
    scripts.insert(helper.id(), helper.clone());
    let bloated = SchemaDefinition::new(
        schema_def.schema.clone(),
        schema_def.libs.clone(),
        Scripts::from_checked(scripts),
    );
    assert_eq!(
        bloated.verify().unwrap_err(),
        SchemaDefError::ScriptExtraneous(helper.id())
    );

    // ...but once the validator library calls it, it is needed and accepted
    let (entry_id, entry) = schema_def.scripts.iter().next().unwrap();
    let mut caller = entry.clone();
    caller.libs = LibSeg::try_from_iter([helper.id()]).unwrap();
    let caller_id = caller.id();
    assert_ne!(
        caller_id, *entry_id,
        "changing the libs segment must re-key the library"
    );

    let mut schema = schema_def.schema.clone();
    let mut validator = schema.genesis.validator.unwrap();
    validator.lib = caller_id;
    schema.genesis.validator = Some(validator);
    schema.transitions.values_mut().for_each(|t| {
        let mut validator = t.transition_schema.validator.unwrap();
        validator.lib = caller_id;
        t.transition_schema.validator = Some(validator);
    });

    let reachable = SchemaDefinition::new(
        schema,
        schema_def.libs.clone(),
        Scripts::from_checked(bmap! { caller_id => caller, helper.id() => helper }),
    );
    reachable
        .verify()
        .expect("a library reachable from a validator must be kept");
}

#[test]
fn validate_consignment_chain_fail() {
    let scenario = Scenario::A;
    let resolver = scenario.resolver();

    // genesis chainNet: change from bitcoinRegtest to liquidTestnet
    let file = std::fs::File::open("tests/fixtures/consignment_A.json").unwrap();
    let mut json_consignment: Value = serde_json::from_reader(file).unwrap();
    *json_consignment
        .get_mut("genesis")
        .unwrap()
        .get_mut("chainNet")
        .unwrap() = Value::String(s!("liquidTestnet"));
    let consignment = transfer_from_json_value(&json_consignment);
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ContractChainNetMismatch(
            ChainNet::BitcoinRegtest
        ))
    );
}

#[test]
fn validate_consignment_genesis_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    // schema ID: change genesis[schemaId] with CFA schema ID, while still
    // validating against the schema the consignment was issued under
    let mut consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let actual = asset_schema.schema().schema_id();
    consignment.genesis.schema_id = CFA_SCHEMA_ID;
    let expected = consignment.genesis.schema_id;
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaMismatch { expected, actual })
    );

    // genesis chainNet: change from bitcoinRegtest to bitcoinMainnet
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let mut json_consignment: Value = serde_json::from_reader(file).unwrap();
    *json_consignment
        .get_mut("genesis")
        .unwrap()
        .get_mut("chainNet")
        .unwrap() = Value::String(s!("bitcoinMainnet"));
    let consignment = transfer_from_json_value(&json_consignment);
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ContractChainNetMismatch(
            ChainNet::BitcoinRegtest
        ))
    );

    // genesis seal closing strategy: only FirstOpretOrTapret is currently supported
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let base_consignment: Value = serde_json::from_reader(file).unwrap();
    let mut json_consignment = base_consignment.clone();
    *json_consignment
        .get_mut("genesis")
        .unwrap()
        .get_mut("sealClosingStrategy")
        .unwrap() = Value::String(s!("FirstOpretThenTapret"));
    assert!(
        serde_json::from_str::<UncheckedTransfer>(
            &serde_json::to_string(&json_consignment).unwrap()
        )
        .is_err()
    );
}

#[test]
fn validate_consignment_bundles_fail() {
    let scenario = Scenario::A;
    let resolver = scenario.resolver();

    // bundles first in time pubWitness inputs[0] sequence: change from 0 to 1
    let file = std::fs::File::open("tests/fixtures/consignment_A.json").unwrap();
    let mut json_consignment: Value = serde_json::from_reader(file).unwrap();
    *json_consignment
        .get_mut("bundles")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .get_mut("tx")
        .unwrap()
        .get_mut("input")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .get_mut("sequence")
        .unwrap() = Value::Number(1.into());
    let consignment = transfer_from_json_value(&json_consignment);
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::SealsInvalid(_, _, _))
    ));
}

#[test]
fn validate_consignment_terminal_spent_fail() {
    let scenario = Scenario::A;
    let resolver = scenario.resolver();

    let mut consignment = get_consignment_from_json(&format!("consignment_{scenario}"));

    let spent_opouts = consignment
        .bundles
        .iter()
        .flat_map(|wbundle| wbundle.bundle.input_map.keys().copied())
        .collect::<BTreeSet<_>>();
    // look for an assignment that gets spent
    let mut candidate = None;
    'outer: for wbundle in consignment.bundles.iter() {
        for kt in &wbundle.bundle.known_transitions {
            for (ty, typed_assigns) in kt.transition.assignments.iter() {
                for (no, seal) in typed_assigns.seals().enumerate() {
                    let opout = Opout::new(kt.opid, *ty, no as u16);
                    if spent_opouts.contains(&opout) {
                        candidate = Some((wbundle.bundle.bundle_id(), *seal, opout));
                        break 'outer;
                    }
                }
            }
        }
    }
    let (bundle_id, seal, opout) = candidate.unwrap();

    // terminals: point to an assignment which is spent inside the consignment
    consignment.terminals =
        SmallOrdMap::from_iter_checked([(bundle_id, TerminalSeals::from(NonEmptyVec::with(seal)))]);

    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::TerminalSealSpent(bundle_id, opout))
    );
}

#[test]
fn validate_resolver_errors() {
    let scenario = Scenario::A;
    let base_resolver = scenario.resolver();
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    // the witness of the second bundle is the one whose resolution we make fail
    let txid = consignment.bundles.iter().nth(1).unwrap().witness_id();

    // resolve_pub_witness: ResolverIssue
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::ResolverIssue(Some(txid), s!("connection error"));
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // resolve_pub_witness: IdMismatch
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::IdMismatch {
        actual: Txid::strict_dumb(),
        expected: txid,
    };
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // resolve_pub_witness: InvalidResolverData
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::InvalidResolverData;
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // resolve_pub_witness: WrongChainNet
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::WrongChainNet;
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // check_chain_net: ResolverIssue
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::ResolverIssue(Some(txid), s!("connection error"));
    resolver.check_chain_net_err = Some(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // check_chain_net: IdMismatch
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::IdMismatch {
        actual: Txid::strict_dumb(),
        expected: txid,
    };
    resolver.check_chain_net_err = Some(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // check_chain_net: InvalidResolverData
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::InvalidResolverData;
    resolver.check_chain_net_err = Some(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // check_chain_net: WrongChainNet
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::WrongChainNet;
    resolver.check_chain_net_err = Some(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));
}

#[test]
fn validate_consignment_unknown_tx() {
    let scenario = Scenario::A;
    let base_resolver = scenario.resolver();
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let wbundle = consignment.bundles.iter().nth(1).unwrap();
    let txid = wbundle.witness_id();
    let bundle_id = wbundle.bundle.bundle_id();

    let mut resolver = base_resolver.clone();
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Success(WitnessStatus::Unresolved);
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SealNoPubWitness(bundle_id, txid))
    );
}

#[test]
fn validate_consignment_schema_fail() {
    let scenario = Scenario::B;
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(base_consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let base_schema = asset_schema.schema();
    let transition_type = *base_schema.transitions.keys().last().unwrap();

    // SchemaOpMetaTypeUnknown: schema transition has unknown metatype
    let mut schema = base_schema.clone();
    let meta_type = MetaType::with(42);
    schema
        .transitions
        .get_mut(&transition_type)
        .unwrap()
        .transition_schema
        .metadata = TinyOrdSet::from_checked(bset![meta_type]);
    // Schema/type-system consistency is now checked when the rules are built,
    // so the failure surfaces before the consignment is even looked at.
    let res = asset_schema_rules.with_schema(schema.clone()).unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        SchemaDefError::Schema(ValidationError::InvalidConsignment(
            Failure::SchemaOpMetaTypeUnknown(
                OpFullType::StateTransition(transition_type),
                meta_type
            )
        ))
    );

    // SchemaOpEmptyInputs: schema transition has no inputs
    let mut schema = base_schema.clone();
    schema
        .transitions
        .get_mut(&transition_type)
        .unwrap()
        .transition_schema
        .inputs = TinyOrdMap::new();
    // Schema/type-system consistency is now checked when the rules are built,
    // so the failure surfaces before the consignment is even looked at.
    let res = asset_schema_rules.with_schema(schema.clone()).unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        SchemaDefError::Schema(ValidationError::InvalidConsignment(
            Failure::SchemaOpEmptyInputs(OpFullType::StateTransition(transition_type))
        ))
    );

    // SchemaOpGlobalTypeUnknown: schema transition has unknown global type
    let mut schema = base_schema.clone();
    let global_state_type = GlobalStateType::with(42);
    schema
        .transitions
        .get_mut(&transition_type)
        .unwrap()
        .transition_schema
        .globals = TinyOrdMap::from_checked(bmap! {
        global_state_type => Occurrences::Once
    });
    // Schema/type-system consistency is now checked when the rules are built,
    // so the failure surfaces before the consignment is even looked at.
    let res = asset_schema_rules.with_schema(schema.clone()).unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        SchemaDefError::Schema(ValidationError::InvalidConsignment(
            Failure::SchemaOpGlobalTypeUnknown(
                OpFullType::StateTransition(transition_type),
                global_state_type
            )
        ))
    );

    // SchemaOpAssignmentTypeUnknown: schema transition has unknown assignment type
    let mut schema = base_schema.clone();
    let assignment_type = AssignmentType::with(42);
    schema
        .transitions
        .get_mut(&transition_type)
        .unwrap()
        .transition_schema
        .assignments = TinyOrdMap::from_checked(bmap! {
        assignment_type => Occurrences::Once
    });
    // Schema/type-system consistency is now checked when the rules are built,
    // so the failure surfaces before the consignment is even looked at.
    let res = asset_schema_rules.with_schema(schema.clone()).unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        SchemaDefError::Schema(ValidationError::InvalidConsignment(
            Failure::SchemaOpAssignmentTypeUnknown(
                OpFullType::StateTransition(transition_type),
                assignment_type
            )
        ))
    );

    // SchemaMetaSemIdUnknown: schema meta type has unknown sem id
    let mut schema = base_schema.clone();
    let meta_type = MetaType::with(42);
    let sem_id = SemId::from([42u8; 32]);
    schema.meta_types = TinyOrdMap::from_checked(bmap! {meta_type => MetaDetails {
        sem_id,
        name: fname!("foo")
    }});
    // Schema/type-system consistency is now checked when the rules are built,
    // so the failure surfaces before the consignment is even looked at.
    let res = asset_schema_rules.with_schema(schema.clone()).unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        SchemaDefError::Schema(ValidationError::InvalidConsignment(
            Failure::SchemaMetaSemIdUnknown(meta_type, sem_id)
        ))
    );

    // SchemaGlobalSemIdUnknown: schema global type has unknown sem id
    let mut schema = base_schema.clone();
    let mut global_types = schema.global_types.release();
    let global_state_type = GlobalStateType::with(42);
    let sem_id = SemId::from([42u8; 32]);
    global_types.insert(
        global_state_type,
        GlobalDetails {
            global_state_schema: GlobalStateSchema {
                sem_id,
                max_items: u24::from_le_bytes([42u8; 3]),
            },
            name: fname!("foo"),
        },
    );
    schema.global_types = TinyOrdMap::from_checked(global_types);
    // Schema/type-system consistency is now checked when the rules are built,
    // so the failure surfaces before the consignment is even looked at.
    let res = asset_schema_rules.with_schema(schema.clone()).unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        SchemaDefError::Schema(ValidationError::InvalidConsignment(
            Failure::SchemaGlobalSemIdUnknown(global_state_type, sem_id)
        ))
    );

    // SchemaOwnedSemIdUnknown: schema owned type has unknown sem id
    let mut schema = base_schema.clone();
    let mut owned_types = schema.owned_types.release();
    let assignment_type = AssignmentType::with(56);
    let sem_id = SemId::from([42u8; 32]);
    owned_types.insert(
        assignment_type,
        AssignmentDetails {
            owned_state_schema: OwnedStateSchema::Structured(sem_id),
            default_transition: TransitionType::with(42),
            name: fname!("foo"),
        },
    );
    schema.owned_types = TinyOrdMap::from_checked(owned_types);
    // Schema/type-system consistency is now checked when the rules are built,
    // so the failure surfaces before the consignment is even looked at.
    let res = asset_schema_rules.with_schema(schema.clone()).unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        SchemaDefError::Schema(ValidationError::InvalidConsignment(
            Failure::SchemaOwnedSemIdUnknown(assignment_type, sem_id)
        ))
    );
}

#[test]
fn validate_consignment_commitments_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(base_consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    // NoPrevState: duplicate transition within a bundle, it'll try to spend inputs twice
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let existing_transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .clone();
    let opid = existing_transition.opid;
    let fst_opout = *existing_transition.transition.inputs.first().unwrap();
    witness_bundle
        .bundle
        .known_transitions
        .push(existing_transition)
        .unwrap();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoPrevState(opid, fst_opout))
    );

    // InputMapTransitionMismatch: add different transition that spends the same opouts
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let new_bundle = bundles.last_mut().unwrap();
    let mut transition = new_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    transition.nonce -= 1;

    let fst_input = *transition.inputs.first().unwrap();
    let opid = transition.id();
    new_bundle
        .bundle
        .known_transitions
        .push(KnownTransition::new(transition.id(), transition))
        .unwrap();
    let bundle_id = new_bundle.bundle().bundle_id();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::InputMapTransitionMismatch(
            bundle_id, opid, fst_input
        ))
    );

    // OperationAbsent: remove a bundle that contains spent assignments
    let mut consignment = base_consignment.clone();
    let spent_transitions = consignment
        .bundles
        .iter()
        .flat_map(|b| b.bundle.known_transitions.as_unconfined())
        .flat_map(|kt| kt.transition.inputs.iter())
        .map(|ti| ti.op)
        .collect::<HashSet<_>>();
    let bundle_to_remove = consignment
        .bundles
        .iter()
        .map(|wb| wb.clone().bundle)
        .find(|b| {
            spent_transitions
                .iter()
                .any(|st| b.known_transitions_contain_opid(st))
        })
        .unwrap();
    let bundle_id_to_remove = bundle_to_remove.bundle_id();
    let missing_opid = bundle_to_remove
        .known_transitions
        .iter()
        .find(|kt| spent_transitions.contains(&kt.opid))
        .unwrap()
        .opid;
    consignment.bundles = LargeVec::from_checked(
        consignment
            .bundles
            .into_iter()
            .filter(|b| b.bundle.bundle_id() != bundle_id_to_remove)
            .collect::<Vec<_>>(),
    );
    let (missing_opout, child_opid) = consignment
        .bundles
        .iter()
        .flat_map(|wb| wb.bundle.input_map.iter())
        .find(|(inp, _)| inp.op == missing_opid)
        .map(|(opout, opid)| (*opout, *opid))
        .unwrap();
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoPrevState(child_opid, missing_opout))
    );

    // NoPrevState: modify a transition input to use a missing assignment type
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    let mut transition_inputs = transition.inputs.as_unconfined().clone();
    let mut fst_input = *transition_inputs.first().unwrap();
    let state_type = AssignmentType::with(42);
    fst_input.ty = state_type;
    transition_inputs.insert(fst_input);
    transition.inputs = NonEmptyOrdSet::from_checked(transition_inputs).into();
    replace_transition_in_bundle(witness_bundle, old_opid, transition.clone());
    update_anchor(witness_bundle, None);
    let opid = transition.id();
    let mut input_map = witness_bundle.bundle.input_map.clone().release();
    input_map.insert(fst_input, transition.id());
    witness_bundle.bundle.input_map = NonEmptyOrdMap::from_checked(input_map);
    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &consignment_resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoPrevState(opid, fst_input))
    );

    // NoPrevOut: modify input to reference non-existing assignment number
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    let mut transition_inputs = transition.inputs.as_unconfined().clone();
    let mut fst_input = transition_inputs.pop_first().unwrap();
    fst_input.no = 42;
    transition_inputs.insert(fst_input);
    transition.inputs = NonEmptyOrdSet::from_checked(transition_inputs).into();
    replace_transition_in_bundle(witness_bundle, old_opid, transition.clone());
    update_anchor(witness_bundle, None);
    let opid = transition.id();
    let mut input_map = witness_bundle.bundle.input_map.clone().release();
    input_map.insert(fst_input, transition.id());
    witness_bundle.bundle.input_map = NonEmptyOrdMap::from_checked(input_map);
    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &consignment_resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoPrevState(opid, fst_input))
    );

    // NoPrevState: one of the transitions includes blinded assignments
    let mut consignment = base_consignment.clone();
    let spent_transitions = consignment
        .bundles
        .iter()
        .flat_map(|b| b.bundle.known_transitions.as_unconfined())
        .flat_map(|kt| kt.transition.inputs.iter())
        .map(|ti| ti.op)
        .collect::<HashSet<_>>();
    let mut bundles = consignment.bundles.release().clone();
    let new_bundle = bundles
        .iter_mut()
        .find(|wb| {
            spent_transitions
                .iter()
                .any(|st| wb.bundle.known_transitions_contain_opid(st))
        })
        .unwrap();
    let mut transitions = new_bundle.clone().bundle.known_transitions;
    let transition_kt = transitions
        .iter_mut()
        .find(|kt| spent_transitions.contains(&kt.opid))
        .unwrap();
    let op = transition_kt.opid;
    let transition = &mut transition_kt.transition;
    let opout = Opout {
        op,
        ty: AssignmentType::ASSET,
        no: 0,
    };
    let assignments = transition
        .assignments
        .remove(&AssignmentType::ASSET)
        .unwrap()
        .unwrap()
        .as_fungible()
        .iter()
        .map(|a| {
            let (seal, state) = a.to_revealed().unwrap();
            rgb::Assign::with(BuilderSeal::Concealed(seal.to_secret_seal()), state)
        })
        .collect::<Vec<_>>();
    let assignments =
        TypedAssigns::Fungible(AssignVec::with(NonEmptyVec::from_checked(assignments)));
    transition
        .assignments
        .insert(AssignmentType::ASSET, assignments)
        .unwrap();
    new_bundle.bundle.known_transitions = transitions;
    let child_opid = *bundles
        .iter()
        .flat_map(|wb| wb.bundle.input_map.iter())
        .find(|(inp, _)| opout == **inp)
        .map(|(_, opid)| opid)
        .unwrap();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoPrevState(child_opid, opout))
    );

    // InputMapTransitionMismatch: replace known_transition referenced in input map
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let new_bundle = bundles.last_mut().unwrap();
    let mut transition = new_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    transition.nonce -= 1;
    let opid = transition.id();
    let fst_input = *transition.inputs.first().unwrap();
    new_bundle.bundle.known_transitions =
        NonEmptyVec::from_checked(vec![KnownTransition::new(transition.id(), transition)]);
    let bundle_id = new_bundle.bundle().bundle_id();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::InputMapTransitionMismatch(
            bundle_id, opid, fst_input
        ))
    );

    // InputMapTransitionMismatch
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    let new_input = Opout {
        no: 9,
        ..*transition.inputs.as_unconfined().first().unwrap()
    };
    transition.inputs.push(new_input).unwrap();
    replace_transition_in_bundle(witness_bundle, old_opid, transition.clone());
    update_anchor(witness_bundle, None);
    let bundle_id = witness_bundle.bundle.bundle_id();
    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &consignment_resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::InputMapTransitionMismatch(
            bundle_id,
            transition.id(),
            new_input
        ))
    );

    // MpcInvalid: cannot edit consignment since fields are private
    let bundle = base_consignment.bundles[0].clone();
    let bundle_id = bundle.bundle.bundle_id();
    let witness_id = bundle.witness_id();
    let mut consignment: Value =
        serde_json::from_str(&serde_json::to_string(&base_consignment).unwrap()).unwrap();
    *consignment
        .get_mut("bundles")
        .unwrap()
        .get_mut(0)
        .unwrap()
        .get_mut("anchor")
        .unwrap()
        .get_mut("mpcProof")
        .unwrap()
        .get_mut("cofactor")
        .unwrap() = Value::Number(42.into());
    let consignment = transfer_from_json_value(&consignment);
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::MpcInvalid(bid, wid, _)) if bid == bundle_id && wid == witness_id
    ));

    // NoDbcOutput
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let new_bundle = bundles.last_mut().unwrap();
    let mut witness_tx = new_bundle.tx.clone();
    let mut outputs = witness_tx.output.clone();
    outputs.retain(|o| !o.script_pubkey.is_op_return());
    witness_tx.output = outputs;
    let witness_id = witness_tx.compute_txid();
    new_bundle.tx = witness_tx;
    //update_witness_and_anchor(witness_bundle, contract_id);
    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &consignment_resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoDbcOutput(witness_id))
    );

    // InvalidProofType
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let new_bundle = bundles.last_mut().unwrap();
    let witness_id = new_bundle.witness_id();
    new_bundle.anchor.dbc_proof = DbcProof::Tapret(TapretProof::strict_dumb());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::InvalidProofType(
            witness_id,
            CloseMethod::TapretFirst
        ))
    );

    // WitnessMissingInput: remove input from witness transaction
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let new_bundle = bundles.last_mut().unwrap();
    let bundle_id = new_bundle.bundle.bundle_id();
    let mut witness_tx = new_bundle.tx.clone();
    let mut inputs = witness_tx.input.clone();
    let missing_outpoint = inputs.pop().unwrap().previous_output;
    witness_tx.input = inputs;
    let witness_id = witness_tx.compute_txid();
    new_bundle.tx = witness_tx;
    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &consignment_resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    let msg = format!(
        "the provided witness transaction does not closes seal {}.",
        missing_outpoint
    );
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SealsInvalid(bundle_id, witness_id, msg))
    );

    // NoPrevState: invert first two bundles
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    bundles.swap(0, 1);
    let known_transition = bundles[0].bundle.known_transitions[0].clone();
    let opid = known_transition.opid;
    let opout = *known_transition.transition.inputs.first().unwrap();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoPrevState(opid, opout))
    );

    // DBC-related error cases
    //  EmbedVerifyError::CommitmentMismatch
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles.last_mut().unwrap();
    let bundle_id = wbundle.bundle().bundle_id();
    let mut witness_tx = wbundle.tx.clone();
    let mut outputs = witness_tx.output.clone();
    let output = outputs
        .iter_mut()
        .find(|o| o.script_pubkey.is_op_return())
        .unwrap();
    let mut script_pubkey = output.script_pubkey.clone().into_bytes();
    script_pubkey.swap(1, 2);
    output.script_pubkey = ScriptBuf::from_bytes(script_pubkey);
    witness_tx.output = outputs;
    let witness_id = witness_tx.compute_txid();
    wbundle.tx = witness_tx;

    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &consignment_resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    let expected_msg = s!("commitment doesn't match the message.");
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            bundle_id,
            witness_id,
            expected_msg
        ))
    );
    //  EmbedVerifyError::InvalidMessage
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles.last_mut().unwrap();
    let bundle_id = wbundle.bundle().bundle_id();
    let mut witness_tx = wbundle.tx.clone();
    let mut outputs = witness_tx.output.clone();
    outputs
        .iter_mut()
        .find(|o| o.script_pubkey.is_op_return())
        .unwrap()
        .script_pubkey
        .push_slice([42]);
    witness_tx.output = outputs;
    let witness_id = witness_tx.compute_txid();
    wbundle.tx = witness_tx;

    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &consignment_resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    let expected_msg =
        s!("first OP_RETURN output inside the transaction already contains some data.");
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            bundle_id,
            witness_id,
            expected_msg
        ))
    );
    //  EmbedVerifyError::InvalidMessage
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles.last_mut().unwrap();
    let bundle_id = wbundle.bundle().bundle_id();
    let mut witness_tx = wbundle.tx.clone();
    let mut outputs = witness_tx.output.clone();
    outputs
        .iter_mut()
        .find(|o| o.script_pubkey.is_op_return())
        .unwrap()
        .script_pubkey
        .push_slice([42]);
    witness_tx.output = outputs;
    let witness_id = witness_tx.compute_txid();
    wbundle.tx = witness_tx;

    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &consignment_resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    let expected_msg =
        s!("first OP_RETURN output inside the transaction already contains some data.");
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            bundle_id,
            witness_id,
            expected_msg
        ))
    );
}

#[test]
fn validate_consignment_logic_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(base_consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    // SchemaMismatch: validate against a compatible schema with a different id
    let consignment = base_consignment.clone();
    let schema_id = consignment.schema_id();
    let mut alt_schema = NonInflatableAsset::schema();
    alt_schema.name = tn!("NonInflatableAsset2");
    let alt_schema_id = alt_schema.schema_id();
    let res = consignment
        .validate(
            &asset_schema_rules.with_schema(alt_schema.clone()).unwrap(),
            &resolver,
            &validation_config,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaMismatch {
            expected: schema_id,
            actual: alt_schema_id
        })
    );

    // SchemaUnknownTransitionType: replace transition with unsupported transition type
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    transition.transition_type = TransitionType::with(42);
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaUnknownTransitionType(
            transition_id,
            TransitionType::with(42)
        ))
    );

    // SchemaUnknownMetaType: replace transition with unsupported meta type
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    transition
        .metadata
        .add_value(MetaType::with(42), MetaValue::strict_dumb())
        .unwrap();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaUnknownMetaType(
            transition_id,
            MetaType::with(42)
        ))
    );

    // SchemaUnknownGlobalStateType: replace transition with unsupported global state type
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    transition
        .globals
        .add_state(GlobalStateType::with(42), RevealedData::strict_dumb())
        .unwrap();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaUnknownGlobalStateType(
            transition_id,
            GlobalStateType::with(42)
        ))
    );

    // SchemaUnknownAssignmentType: add unsupported assignment type to transition
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    transition
        .assignments
        .insert(AssignmentType::with(42), TypedAssigns::strict_dumb())
        .unwrap();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaUnknownAssignmentType(
            transition_id,
            AssignmentType::with(42)
        ))
    );

    // SchemaAssignmentOccurrences: add transition with no assignments
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    transition.assignments = SmallOrdMap::new().into();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaAssignmentOccurrences(
            transition_id,
            AssignmentType::with(4000),
            OccurrencesMismatch {
                min: 1,
                max: 65535,
                found: 0
            }
        ))
    );

    // StateTypeMismatch
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    let assignment_type = AssignmentType::with(4000);
    transition
        .assignments
        .insert(
            assignment_type,
            TypedAssigns::Declarative(
                NonEmptyVec::with(Assign::with(
                    BuilderSeal::Concealed(SecretSeal::strict_dumb()),
                    VoidState::strict_dumb(),
                ))
                .into(),
            ),
        )
        .unwrap();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::StateTypeMismatch {
            opid: transition_id,
            state_type: assignment_type,
            expected: StateType::Fungible,
            found: StateType::Void
        })
    );

    // ScriptFailure: e.g. one can't do simple inflation
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    let assignment_type = AssignmentType::with(4000);
    let output_sum = transition
        .assignments
        .get(&assignment_type)
        .unwrap()
        .as_fungible()
        .iter()
        .map(|a| a.as_state().as_u64())
        .sum::<u64>();
    transition
        .assignments
        .insert(
            assignment_type,
            TypedAssigns::Fungible(
                NonEmptyVec::with(Assign::with(
                    BuilderSeal::Concealed(SecretSeal::strict_dumb()),
                    RevealedValue::new(output_sum + 1),
                ))
                .into(),
            ),
        )
        .unwrap();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ScriptFailure(
            transition_id,
            Some(ERRNO_NON_EQUAL_IN_OUT),
            None
        ))
    );

    // ContractMismatch: operations should commit to the correct contract
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    let old_contract_id = transition.contract_id;
    transition.contract_id = ContractId::strict_dumb();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    // update again with the correct contract_id, otherwise we get SealsInvalid
    update_anchor(witness_bundle, Some(old_contract_id));
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ContractMismatch(
            transition_id,
            ContractId::strict_dumb()
        ))
    );

    // Error: zero-amount allocations are not allowed
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    if let TypedAssigns::Fungible(assign) = transition.assignments.get_mut(&OS_ASSET).unwrap() {
        assign
            .push(Assign::with(
                BuilderSeal::Concealed(SecretSeal::strict_dumb()),
                RevealedValue::new(Amount::ZERO),
            ))
            .unwrap();
    } else {
        panic!("unexpected asssignment type")
    };
    let opid = transition.id();
    assert_ne!(opid, old_opid);
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ScriptFailure(
            opid,
            Some(ERRNO_NON_EQUAL_IN_OUT),
            None
        ))
    );

    // UnsafeHistory
    let consignment = base_consignment.clone();
    let witness_tx = consignment.bundles.last().unwrap().tx.clone();
    let witness_id = witness_tx.compute_txid();
    // transaction is added as tentative
    let alt_resolver = resolver.with_new_transaction(witness_tx);
    let mut validation_config_mod = validation_config.clone();
    validation_config_mod.safe_height = Some(NonZeroU32::new(1000).unwrap());
    let res = consignment.validate(&asset_schema_rules, &alt_resolver, &validation_config_mod);
    let warnings = res.unwrap().validation_status().warnings.clone();
    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0],
        Warning::UnsafeHistory(map! {0 => set![ witness_id ]})
    );

    // the following test cases require a more complex schema (IFA)
    let scenario = Scenario::C;
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(base_consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    // find a "inflation" transition
    let mut old_txid = None;
    let mut base_transition = None;
    let _spent_opouts = base_consignment
        .bundles
        .iter()
        .flat_map(|b| b.bundle.known_transitions.iter())
        .flat_map(|kt| kt.transition.inputs.iter().map(|ti| ti.op))
        .collect::<HashSet<_>>();
    for wbun in base_consignment.bundled_witnesses() {
        for KnownTransition { transition, .. } in wbun.bundle.known_transitions.iter() {
            if transition.transition_type == TS_INFLATION {
                old_txid = Some(wbun.witness_id());
                base_transition = Some(transition.clone());
                break;
            }
        }
    }
    let old_txid = old_txid.unwrap();
    let base_transition = base_transition.unwrap();
    let old_opid = base_transition.id();

    // SchemaNoMetadata
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_txid)
        .unwrap();
    let mut transition = base_transition.clone();
    transition.metadata = none!();
    let opid = transition.id();
    replace_transition_in_bundle(wbundle, old_opid, transition);
    let txid = wbundle.witness_id();
    update_transition_children(
        &mut bundles,
        HashMap::from([(old_opid, opid)]),
        HashMap::from([(old_txid, txid)]),
        None,
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaNoMetadata(opid, MS_ALLOWED_INFLATION))
    );

    // SchemaInvalidMetadata
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_txid)
        .unwrap();
    let mut transition = base_transition.clone();
    transition
        .metadata
        .insert(MS_ALLOWED_INFLATION, MetaValue::from_hex("42").unwrap())
        .unwrap();
    let opid = transition.id();
    replace_transition_in_bundle(wbundle, old_opid, transition);
    let txid = wbundle.witness_id();
    update_transition_children(
        &mut bundles,
        HashMap::from([(old_opid, opid)]),
        HashMap::from([(old_txid, txid)]),
        None,
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    let sem_id = StandardTypes::with(rgb_contract_stl()).get("RGBContract.Amount");
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaInvalidMetadata(opid, sem_id))
    );

    // SchemaGlobalStateOccurrences
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_txid)
        .unwrap();
    let mut transition = base_transition.clone();
    let globals = transition.globals.clone();
    let global_state_type = globals.keys().next().unwrap();
    transition.globals = none!();
    let opid = transition.id();
    replace_transition_in_bundle(wbundle, old_opid, transition);
    let txid = wbundle.witness_id();
    update_transition_children(
        &mut bundles,
        HashMap::from([(old_opid, opid)]),
        HashMap::from([(old_txid, txid)]),
        None,
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaGlobalStateOccurrences(
            opid,
            *global_state_type,
            OccurrencesMismatch {
                min: 1,
                max: 1,
                found: 0
            }
        ))
    );

    // SchemaInvalidGlobalValue
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_txid)
        .unwrap();
    let mut transition = base_transition.clone();
    *transition
        .globals
        .get_mut(&GS_ISSUED_SUPPLY)
        .unwrap()
        .get_mut(0)
        .unwrap() = RevealedData::strict_dumb();
    let opid = transition.id();
    replace_transition_in_bundle(wbundle, old_opid, transition);
    let txid = wbundle.witness_id();
    update_transition_children(
        &mut bundles,
        HashMap::from([(old_opid, opid)]),
        HashMap::from([(old_txid, txid)]),
        None,
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    let sem_id = StandardTypes::with(rgb_contract_stl()).get("RGBContract.Amount");
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaInvalidGlobalValue(
            opid,
            GS_ISSUED_SUPPLY,
            sem_id
        ))
    );

    // SchemaUnknownAssignmentType: unexpected assignment type in transition input
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let mut witness_id = None;
    let mut transfer_transition = None;
    for wbun in bundles.iter() {
        for KnownTransition { transition, .. } in wbun.bundle.known_transitions.iter() {
            if transition.transition_type == TS_TRANSFER
                && transition
                    .inputs
                    .iter()
                    .map(|i| i.ty)
                    .collect::<HashSet<_>>()
                    .is_superset(&set![OS_ASSET, OS_INFLATION])
            {
                witness_id = Some(wbun.witness_id());
                transfer_transition = Some(transition);
                break;
            }
        }
    }
    let witness_id = witness_id.unwrap();
    let mut transition = transfer_transition.unwrap().clone();
    let old_opid = transition.id();
    transition.transition_type = TS_INFLATION;
    let val = Amount::from(42u64)
        .to_strict_serialized::<{ u16::MAX as usize }>()
        .unwrap();
    transition
        .metadata
        .add_value(MS_ALLOWED_INFLATION, val.clone().into())
        .unwrap();
    transition
        .globals
        .add_state(GS_ISSUED_SUPPLY, val.into())
        .unwrap();
    let opid = transition.id();
    assert_ne!(opid, old_opid);
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == witness_id)
        .unwrap();
    replace_transition_in_bundle(wbundle, old_opid, transition);
    remove_transition_children(&mut bundles, bset![old_opid], None);
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaUnknownAssignmentType(opid, OS_ASSET))
    );
}

/// The last revealed transition of the consignment's last bundle.
fn last_transition(consignment: &Transfer) -> Transition {
    consignment
        .bundles
        .last()
        .unwrap()
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone()
}

fn assert_bfa_valid(consignment: Transfer, rules: &SchemaRules, resolver: &MockResolver) {
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    // a witness resolver alone cannot validate a consignment carrying external anchors
    assert!(matches!(
        consignment
            .clone()
            .validate(rules, resolver, &validation_config)
            .unwrap_err(),
        ValidationError::ExternalAnchorsPending(_)
    ));
    let mut pending_consignment = consignment
        .validate_deterministic(rules, &validation_config)
        .unwrap();
    pending_consignment
        .resolve_all_anchors(
            BridgedFungibleAsset::bridge_location(pending_consignment.consignment().genesis())
                .unwrap(),
            &MockAnchorResolver::new(BFA_CHAIN_ID, |_anchor| true),
        )
        .unwrap();
    pending_consignment.pending().resolve_all(resolver).unwrap();
    let res = pending_consignment.finalize();
    let validation_status = res.validation_status();
    dbg!(&validation_status);
    assert_eq!(validation_status.validity(), Validity::Valid);
}

/// Assert a BFA consignment is rejected by a schema script.
///
/// Scripts run in phase 1, so this needs neither a resolver nor the external anchors: the
/// consignment is rejected before anything is asked of a chain.
fn assert_bfa_script_failure(consignment: Transfer, rules: &SchemaRules, opid: OpId, errno: u8) {
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let res = consignment
        .validate_deterministic(rules, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ScriptFailure(opid, Some(errno), None))
    );
}

#[test]
fn validate_consignment_bfa_mint_right_transfer() {
    fn bfa_mutated_transfer(
        base_consignment: &Transfer,
        resolver: &MockResolver,
        mutate: impl FnOnce(&mut Transition),
    ) -> (Transfer, MockResolver, OpId) {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let witness_bundle = bundles.last_mut().unwrap();
        let mut transition = witness_bundle
            .bundle
            .known_transitions
            .last()
            .unwrap()
            .transition
            .clone();
        let old_opid = transition.id();
        assert_eq!(
            transition.transition_type, TS_TRANSFER,
            "fixture must end with a transfer transition"
        );
        mutate(&mut transition);
        let transition_id = transition.id();
        assert_ne!(
            transition_id, old_opid,
            "mutation left the transition unchanged"
        );
        let inputs = transition.inputs.iter().copied().collect::<BTreeSet<_>>();
        replace_transition_in_bundle(witness_bundle, old_opid, transition);
        let input_map = witness_bundle
            .bundle
            .input_map
            .clone()
            .release()
            .into_iter()
            .filter(|(opout, opid)| *opid != transition_id || inputs.contains(opout))
            .collect();
        witness_bundle.bundle.input_map = NonEmptyOrdMap::from_checked(input_map);
        update_anchor(witness_bundle, None);
        let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
        consignment.bundles = LargeVec::from_checked(bundles);
        consignment.terminals = empty!(); // terminals are now outdated
        (consignment, alt_resolver, transition_id)
    }

    /// Count the `OS_MINT` inputs and outputs of a transition.
    fn mint_right_counts(transition: &Transition) -> (usize, usize) {
        let inputs = transition
            .inputs
            .iter()
            .filter(|opout| opout.ty == OS_MINT)
            .count();
        let outputs = transition
            .assignments
            .get(&OS_MINT)
            .map(|assigns| assigns.as_declarative().len())
            .unwrap_or(0);
        (inputs, outputs)
    }

    /// Drop the single `OS_MINT` input of a transition, leaving its other inputs untouched.
    fn drop_mint_right_input(transition: &mut Transition) {
        let remaining = transition
            .inputs
            .iter()
            .copied()
            .filter(|opout| opout.ty != OS_MINT)
            .collect::<Vec<_>>();
        assert_eq!(
            remaining.len() + 1,
            transition.inputs.len(),
            "fixture transition must carry exactly one OS_MINT input to drop"
        );
        transition.inputs = NonEmptyOrdSet::from_iter_checked(remaining).into();
    }

    /// Assign a second `OS_MINT` output to a transition which already has one.
    fn split_mint_right_output(transition: &mut Transition) {
        let mut rights = transition
            .assignments
            .get(&OS_MINT)
            .expect("fixture transition must carry an OS_MINT output")
            .as_declarative()
            .to_vec();
        rights.push(Assign::with(
            BuilderSeal::Revealed(GraphSeal::new_random_vout(0)),
            VoidState::strict_dumb(),
        ));
        transition
            .assignments
            .insert(
                OS_MINT,
                TypedAssigns::Declarative(NonEmptyVec::from_checked(rights).into()),
            )
            .unwrap();
    }

    /// Append a `transfer` bundle spending every assignment of the consignment's last transition
    fn append_mint_right_spend(
        consignment: &mut Transfer,
        resolver: &MockResolver,
        rights_out: usize,
    ) -> (MockResolver, OpId) {
        let parent_transition = last_transition(consignment);
        let parent_opid = parent_transition.id();

        // spend every assignment of the parent transition
        let inputs = parent_transition
            .assignments
            .iter()
            .flat_map(|(ty, assigns)| {
                (0..assigns.len_u16()).map(|no| Opout::new(parent_opid, *ty, no))
            })
            .collect::<Vec<_>>();

        // re-assign everything, keeping only `rights_out` of the mint rights
        let mut rights = parent_transition
            .assignments
            .get(&OS_MINT)
            .expect("parent transition must carry OS_MINT outputs")
            .as_declarative()
            .to_vec();
        assert!(
            rights_out <= rights.len(),
            "parent transition carries fewer than {rights_out} OS_MINT outputs"
        );
        rights.truncate(rights_out);
        let mut transition = Transition {
            inputs: NonEmptyOrdSet::from_iter_checked(inputs).into(),
            ..parent_transition
        };
        transition
            .assignments
            .insert(
                OS_MINT,
                TypedAssigns::Declarative(NonEmptyVec::from_checked(rights).into()),
            )
            .unwrap();
        let opid = transition.id();
        (append_bundle(consignment, resolver, transition), opid)
    }
    let scenario = Scenario::E;
    let resolver = scenario.resolver();
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema_rules = AssetSchema::from(base_consignment.schema_id()).schema_rules();

    // valid, 1 -> 1: the unmutated fixture
    let consignment = base_consignment.clone();
    assert_eq!(mint_right_counts(&last_transition(&consignment)), (1, 1));
    assert_bfa_valid(consignment, &asset_schema_rules, &resolver);

    // valid, 1 -> 2: a right may be split
    let (consignment, alt_resolver, _) =
        bfa_mutated_transfer(&base_consignment, &resolver, |transition| {
            split_mint_right_output(transition);
            assert_eq!(mint_right_counts(transition), (1, 2));
        });
    assert_bfa_valid(consignment, &asset_schema_rules, &alt_resolver);

    // valid, 0 -> 0: a plain asset transfer, carrying no mint right
    let (consignment, alt_resolver, _) =
        bfa_mutated_transfer(&base_consignment, &resolver, |transition| {
            drop_mint_right_input(transition);
            transition
                .assignments
                .remove(&OS_MINT)
                .expect("fixture transition must carry an OS_MINT output");
            assert_eq!(mint_right_counts(transition), (0, 0));
        });
    assert_bfa_valid(consignment, &asset_schema_rules, &alt_resolver);

    // valid, 2 -> 2: two rights can be moved in the same transfer
    let (mut consignment, alt_resolver, _) =
        bfa_mutated_transfer(&base_consignment, &resolver, split_mint_right_output);
    let (alt_resolver, _) = append_mint_right_spend(&mut consignment, &alt_resolver, 2);
    assert_eq!(mint_right_counts(&last_transition(&consignment)), (2, 2));
    assert_bfa_valid(consignment, &asset_schema_rules, &alt_resolver);

    // ScriptFailure, 0 -> 1: a right may not be created from thin air
    let (consignment, _, transition_id) =
        bfa_mutated_transfer(&base_consignment, &resolver, |transition| {
            drop_mint_right_input(transition);
            assert_eq!(mint_right_counts(transition), (0, 1));
        });
    assert_bfa_script_failure(
        consignment,
        &asset_schema_rules,
        transition_id,
        ERRNO_MISSING_INPUT,
    );

    // ScriptFailure, 1 -> 0: transfer doesn't allow hidden burn
    let (consignment, _, transition_id) =
        bfa_mutated_transfer(&base_consignment, &resolver, |transition| {
            transition
                .assignments
                .remove(&OS_MINT)
                .expect("fixture transition must carry an OS_MINT output to drop");
            assert_eq!(mint_right_counts(transition), (1, 0));
        });
    assert_bfa_script_failure(
        consignment,
        &asset_schema_rules,
        transition_id,
        ERRNO_HIDDEN_BURN,
    );

    // ScriptFailure, 2 -> 1: transfer doesn't allow hidden burn
    let (mut consignment, alt_resolver, _) =
        bfa_mutated_transfer(&base_consignment, &resolver, split_mint_right_output);
    let (_, transition_id) = append_mint_right_spend(&mut consignment, &alt_resolver, 1);
    assert_eq!(mint_right_counts(&last_transition(&consignment)), (2, 1));
    assert_bfa_script_failure(
        consignment,
        &asset_schema_rules,
        transition_id,
        ERRNO_HIDDEN_BURN,
    );
}

#[test]
fn validate_consignment_bfa_burn() {
    /// Append a `burn` bundle spending the `spend` assignment types of the consignment's last
    /// transition.
    fn append_burn(
        consignment: &mut Transfer,
        resolver: &MockResolver,
        spend: &[AssignmentType],
        burned: Option<u64>,
        change: Option<u64>,
    ) -> (MockResolver, OpId) {
        let parent_transition = last_transition(consignment);
        let parent_opid = parent_transition.id();
        let inputs = spend
            .iter()
            .flat_map(|ty| {
                let ty = *ty;
                let count = parent_transition
                    .assignments
                    .get(&ty)
                    .map(|assigns| assigns.len_u16())
                    .unwrap_or(0);
                (0..count).map(move |no| Opout::new(parent_opid, ty, no))
            })
            .collect::<Vec<_>>();

        let mut globals = parent_transition.globals.clone();
        if let Some(burned) = burned {
            let val = Amount::from(burned)
                .to_strict_serialized::<{ u16::MAX as usize }>()
                .unwrap();
            globals.add_state(GS_BURNED_ASSET, val.into()).unwrap();
        }

        // a burn transition defines no OS_MINT assignments: a right it spends is retired
        let mut assignments = parent_transition.assignments.clone();
        assignments.remove(&OS_MINT).unwrap();
        match change {
            Some(amount) => {
                assignments
                    .insert(
                        OS_ASSET,
                        TypedAssigns::Fungible(AssignVec::with(NonEmptyVec::with(
                            AssignFungible::with(
                                BuilderSeal::Revealed(GraphSeal::new_random_vout(0)),
                                RevealedValue::new(amount),
                            ),
                        ))),
                    )
                    .unwrap();
            }
            None => {
                assignments.remove(&OS_ASSET).unwrap();
            }
        }

        let transition = Transition {
            transition_type: TS_BURN,
            inputs: NonEmptyOrdSet::from_iter_checked(inputs).into(),
            globals,
            assignments,
            ..parent_transition
        };
        let opid = transition.id();
        (append_bundle(consignment, resolver, transition), opid)
    }

    let scenario = Scenario::E;
    let resolver = scenario.resolver();
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema_rules = AssetSchema::from(base_consignment.schema_id()).schema_rules();

    // the fixture ends with a transfer holding the whole bridged supply plus the mint right
    let parent = last_transition(&base_consignment);
    let supply = parent
        .assignments
        .get(&OS_ASSET)
        .expect("fixture transition must carry an OS_ASSET output")
        .as_fungible()
        .iter()
        .map(|assign| assign.as_state().as_u64())
        .sum::<u64>();
    assert_ne!(supply, 0);
    assert_eq!(
        parent
            .assignments
            .get(&OS_MINT)
            .expect("fixture transition must carry an OS_MINT output")
            .as_declarative()
            .len(),
        1
    );

    // valid: the whole allocation is burned, leaving no change
    let mut consignment = base_consignment.clone();
    let (alt_resolver, _) =
        append_burn(&mut consignment, &resolver, &[OS_ASSET], Some(supply), None);
    assert_bfa_valid(consignment, &asset_schema_rules, &alt_resolver);

    // valid: a partial burn, the remainder carried over as change
    let mut consignment = base_consignment.clone();
    let burnt = supply / 4;
    let (alt_resolver, _) = append_burn(
        &mut consignment,
        &resolver,
        &[OS_ASSET],
        Some(burnt),
        Some(supply - burnt),
    );
    assert_bfa_valid(consignment, &asset_schema_rules, &alt_resolver);

    // valid: the mint right alone is retired
    let mut consignment = base_consignment.clone();
    let (alt_resolver, _) = append_burn(&mut consignment, &resolver, &[OS_MINT], None, None);
    let burn = last_transition(&consignment);
    assert!(burn.inputs.iter().all(|opout| opout.ty == OS_MINT));
    assert!(burn.globals.get(&GS_BURNED_ASSET).is_none());
    assert!(burn.assignments.is_empty());
    assert_bfa_valid(consignment, &asset_schema_rules, &alt_resolver);

    // valid: asset and mint right retired by the same transition
    let mut consignment = base_consignment.clone();
    let (alt_resolver, _) = append_burn(
        &mut consignment,
        &resolver,
        &[OS_ASSET, OS_MINT],
        Some(supply),
        None,
    );
    assert_bfa_valid(consignment, &asset_schema_rules, &alt_resolver);

    // ScriptFailure: the reported burned amount must match what actually disappears
    let mut consignment = base_consignment.clone();
    let (_, opid) = append_burn(
        &mut consignment,
        &resolver,
        &[OS_ASSET],
        Some(supply - 1),
        None,
    );
    assert_bfa_script_failure(consignment, &asset_schema_rules, opid, ERRNO_BURN_MISMATCH);

    // ScriptFailure: an explicit zero is malformed state
    let mut consignment = base_consignment.clone();
    let (_, opid) = append_burn(
        &mut consignment,
        &resolver,
        &[OS_ASSET],
        Some(0),
        Some(supply),
    );
    assert_bfa_script_failure(consignment, &asset_schema_rules, opid, ERRNO_BURN_MISMATCH);

    // ScriptFailure: a burn transition may not inflate
    let mut consignment = base_consignment.clone();
    let (_, opid) = append_burn(
        &mut consignment,
        &resolver,
        &[OS_ASSET],
        None,
        Some(supply + 1),
    );
    assert_bfa_script_failure(consignment, &asset_schema_rules, opid, ERRNO_BURN_MISMATCH);

    // ScriptFailure: a burn transition cannot be used as a plain transfer
    let mut consignment = base_consignment.clone();
    let (_, opid) = append_burn(&mut consignment, &resolver, &[OS_ASSET], None, Some(supply));
    assert_bfa_script_failure(consignment, &asset_schema_rules, opid, ERRNO_BURN_ZERO);
}

#[test]
fn validate_consignment_remove_scripts_code() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(base_consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    // ScriptFailure: e.g. one can't do simple inflation
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let mut transition = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .clone();
    let old_opid = transition.id();
    let assignment_type = AssignmentType::with(4000);
    let output_sum = transition
        .assignments
        .get(&assignment_type)
        .unwrap()
        .as_fungible()
        .iter()
        .map(|a| a.as_state().as_u64())
        .sum::<u64>();
    transition
        .assignments
        .insert(
            assignment_type,
            TypedAssigns::Fungible(
                NonEmptyVec::with(Assign::with(
                    BuilderSeal::Concealed(SecretSeal::strict_dumb()),
                    RevealedValue::new(output_sum + 1),
                ))
                .into(),
            ),
        )
        .unwrap();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver = resolver.with_new_transaction(witness_bundle.tx.clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let mut scripts = asset_schema.scripts().release();
    let (lib_id, mut lib) = scripts.pop_last().unwrap();
    lib.code = none!();
    let tampered_scripts = Scripts::from_checked(bmap![lib.id() => lib]);
    // Tampering with the library changes its id, so the library the schema
    // references is no longer there. That is caught when the rules are built;
    // the validator keeps its own `MissingScript` check as a backstop.
    let _ = (&consignment, &alt_resolver, &validation_config);
    let res = asset_schema_rules
        .with_scripts(tampered_scripts)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, SchemaDefError::ScriptAbsent(lib_id));
}

#[test]
fn validate_consignment_unmatching_transition_id() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(base_consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let witness_bundle = bundles.last_mut().unwrap();
    let contract_id = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .transition
        .contract_id;

    let mut other_wbundle = witness_bundle.clone();
    let KnownTransition { opid, transition } = witness_bundle
        .bundle
        .known_transitions
        .last()
        .unwrap()
        .clone();
    // modified transition lies in witness_bundle, but is committed to in other_bundle
    let mut transition = transition.clone();
    transition.nonce -= 1;
    witness_bundle
        .bundle
        .known_transitions
        .iter_mut()
        .find(|kt| kt.opid == opid)
        .unwrap()
        .transition = transition.clone();
    let dumb_transition = Transition::strict_dumb();
    let dumb_id = dumb_transition.id();
    // known_transitions can't be empty, so we need to add something
    // we have no free allocations for a meaningful transition so it is a dumb one
    // which causes OperationAbsent(OpId(0000000000000000000000000000000000000000000000000000000000000000))
    other_wbundle.bundle.known_transitions =
        NonEmptyVec::with(KnownTransition::new(dumb_id, dumb_transition));
    other_wbundle
        .bundle
        .input_map
        .insert(Opout::strict_dumb(), dumb_id)
        .unwrap();
    update_anchor(&mut other_wbundle, Some(contract_id));

    let alt_resolver = resolver.with_new_transaction(other_wbundle.tx.clone());
    bundles.push(other_wbundle);
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&asset_schema_rules, &alt_resolver, &validation_config)
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::TransitionIdMismatch(opid, transition.id()))
    );
}

#[test]
fn validate_consignment_ifa() {
    let scenario = Scenario::C;
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(base_consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    // take the last "transfer" transition
    let (old_txid, known_transition) = base_consignment
        .bundles
        .iter()
        .filter_map(|wb| {
            let kt = wb.bundle.known_transitions.last().unwrap();
            if kt.transition.transition_type == TS_TRANSFER {
                Some((wb.witness_id(), kt))
            } else {
                None
            }
        })
        .next_back()
        .unwrap();
    let old_opid = known_transition.opid;
    let base_transition = known_transition.transition.clone();

    // Error: zero-amount allocations are not allowed for fungible assignment types
    for assignment_type in [OS_ASSET, OS_INFLATION] {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        let old_opid = transition.id();
        let TypedAssigns::Fungible(assign) =
            transition.assignments.get_mut(&assignment_type).unwrap()
        else {
            panic!("unexpected asssignment type")
        };
        assign
            .push(Assign::with(
                BuilderSeal::Concealed(SecretSeal::strict_dumb()),
                RevealedValue::new(Amount::ZERO),
            ))
            .unwrap();
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        let txid = wbundle.witness_id();
        update_transition_children(
            &mut bundles,
            HashMap::from([(old_opid, opid)]),
            HashMap::from([(old_txid, txid)]),
            None,
        );
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &consignment,
        };
        let res = consignment
            .clone()
            .validate(&asset_schema_rules, &resolver, &validation_config)
            .unwrap_err();
        dbg!(&res);
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::ScriptFailure(
                opid,
                Some(ERRNO_NON_EQUAL_IN_OUT),
                None
            ))
        );
    }

    // Error: inflation is not allowed for fungible assignment types
    for assignment_type in [OS_ASSET, OS_INFLATION] {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        let TypedAssigns::Fungible(assign) =
            transition.assignments.get_mut(&assignment_type).unwrap()
        else {
            panic!("unexpected asssignment type")
        };
        let value = assign.iter_mut().last().unwrap().as_state_mut();
        *value = RevealedValue::new(value.as_u64() + 1);
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &consignment,
        };
        let res = consignment
            .clone()
            .validate(&asset_schema_rules, &resolver, &validation_config)
            .unwrap_err();
        dbg!(&res);
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::ScriptFailure(
                opid,
                Some(ERRNO_NON_EQUAL_IN_OUT),
                None
            ))
        );
    }

    // test inflation transition
    let mut witness_id = None;
    let mut base_transition = None;
    for wbun in base_consignment.bundled_witnesses() {
        for KnownTransition { transition, .. } in wbun.bundle.known_transitions.iter() {
            if transition.transition_type == TS_INFLATION {
                witness_id = Some(wbun.witness_id());
                base_transition = Some(transition.clone());
                break;
            }
        }
    }
    let old_txid = witness_id.unwrap();
    let base_transition = base_transition.unwrap();
    let old_opid = base_transition.id();

    // Error: inflation transitions can't inflate more than allowed
    for assignment_type in [OS_ASSET, OS_INFLATION] {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        let TypedAssigns::Fungible(assign) =
            transition.assignments.get_mut(&assignment_type).unwrap()
        else {
            panic!("unexpected asssignment type")
        };
        let value = assign.iter_mut().last().unwrap().as_state_mut();
        *value = RevealedValue::new(value.as_u64() + 1);
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &consignment,
        };
        let res = consignment
            .clone()
            .validate(&asset_schema_rules, &resolver, &validation_config)
            .unwrap_err();
        dbg!(&res);
        let errno = match assignment_type {
            OS_ASSET => ERRNO_ISSUED_MISMATCH,
            OS_INFLATION => ERRNO_INFLATION_MISMATCH,
            _ => unreachable!(),
        };
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::ScriptFailure(opid, Some(errno), None))
        );
    }

    // test burn transition
    let mut witness_id = None;
    let mut base_transition = None;
    for wbun in base_consignment.bundled_witnesses() {
        for KnownTransition { transition, .. } in wbun.bundle.known_transitions.iter() {
            if transition.transition_type == TS_BURN {
                witness_id = Some(wbun.witness_id());
                base_transition = Some(transition.clone());
                break;
            }
        }
    }
    let old_txid = witness_id.unwrap();
    let base_transition = base_transition.unwrap();
    let old_opid = base_transition.id();
    let input_assignment_types = base_transition
        .inputs
        .iter()
        .map(|i| i.ty)
        .collect::<HashSet<_>>();
    assert_eq!(input_assignment_types, set![OS_ASSET, OS_INFLATION]);
    let assignment_types = [OS_ASSET, OS_INFLATION];
    let ifa_schema = InflatableFungibleAsset::schema();
    let burn_global_type =
        |at: &AssignmentType| ifa_schema.global_type(burn_global_by_assignment(at));

    // Error: burn transitions can't inflate
    for assignment_type in assignment_types {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        transition
            .assignments
            .insert(
                assignment_type,
                TypedAssigns::Fungible(AssignVec::with(NonEmptyVec::with(AssignFungible::with(
                    BuilderSeal::Concealed(SecretSeal::strict_dumb()),
                    RevealedValue::new(1u64),
                )))),
            )
            .unwrap();
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &consignment,
        };
        let res = consignment
            .clone()
            .validate(&asset_schema_rules, &resolver, &validation_config)
            .unwrap_err();
        dbg!(&res);
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::ScriptFailure(
                opid,
                Some(ERRNO_BURN_MISMATCH),
                None,
            ))
        );
    }

    // Error: burn transitions need to report correct burn amount
    for assignment_type in assignment_types {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        let global_type = burn_global_type(&assignment_type);
        let entry = transition
            .globals
            .get_mut(&global_type)
            .unwrap()
            .get_mut(0)
            .unwrap();
        let burn_amt = u64::from_le_bytes(<[u8; 8]>::try_from(entry.as_slice()).unwrap());
        *entry = SmallBlob::from_iter_checked((burn_amt + 1).to_le_bytes()).into();
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &consignment,
        };
        let res = consignment
            .clone()
            .validate(&asset_schema_rules, &resolver, &validation_config)
            .unwrap_err();
        dbg!(&res);
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::ScriptFailure(
                opid,
                Some(ERRNO_BURN_MISMATCH),
                None,
            ))
        );
    }

    // Error: burn transitions need to burn a nonzero amount.
    // `zeroed` covers both ways of reporting that nothing was burned: an explicit zero, and - now
    // that the burned amount is NoneOrOnce global state - the entry being absent altogether. Both
    // are rejected, but for different reasons, so they report different errnos. An explicit zero
    // is malformed state: absent is the only way to say "nothing of this type was burned", so the
    // script rejects the entry before it ever reaches the economic check, with
    // ERRNO_BURN_MISMATCH. An absent entry is well-formed, and it is the economic check that
    // rejects it with ERRNO_BURN_ZERO: a pure transfer wearing a burn costume. That second case
    // is what guards the check that used to be enforced by the schema requiring the metadata to
    // be present.
    for (assignment_type, zeroed) in assignment_types
        .iter()
        .flat_map(|at| [(*at, true), (*at, false)])
    {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        let global_type = burn_global_type(&assignment_type);
        let burn_amt = u64::from_le_bytes(
            <[u8; 8]>::try_from(transition.globals[&global_type].first().unwrap().as_slice())
                .unwrap(),
        );
        let chg_amt: u64 = if let Some(ta) = transition.assignments.get(&assignment_type) {
            ta.as_fungible().iter().map(|a| a.as_state().as_u64()).sum()
        } else {
            0
        };
        transition
            .assignments
            .insert(
                assignment_type,
                TypedAssigns::Fungible(AssignVec::with(NonEmptyVec::with(AssignFungible::with(
                    BuilderSeal::Concealed(SecretSeal::strict_dumb()),
                    RevealedValue::new(burn_amt + chg_amt),
                )))),
            )
            .unwrap();
        if zeroed {
            *transition
                .globals
                .get_mut(&global_type)
                .unwrap()
                .get_mut(0)
                .unwrap() = SmallBlob::from_iter_checked([0; 8]).into();
        } else {
            transition.globals.remove(&global_type).unwrap();
        }
        assert_ne!(burn_amt, 0);
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &consignment,
        };
        let res = consignment
            .clone()
            .validate(&asset_schema_rules, &resolver, &validation_config)
            .unwrap_err();
        dbg!(&res);
        let errno = if zeroed {
            ERRNO_BURN_MISMATCH
        } else {
            ERRNO_BURN_ZERO
        };
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::ScriptFailure(opid, Some(errno), None))
        );
    }

    // Success: when an assignment type is not burned, its burned amount is simply absent.
    // Under NoneOrOnce this is legal where it previously required an explicit zero.
    for assignment_type in assignment_types {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        let global_type = burn_global_type(&assignment_type);
        transition.inputs = NonEmptyOrdSet::from_iter_checked(
            transition
                .inputs
                .iter()
                .filter(|i| i.ty != assignment_type)
                .cloned(),
        )
        .into();
        transition.assignments.remove(&assignment_type).unwrap();
        transition.globals.remove(&global_type).unwrap();
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &consignment,
        };
        consignment
            .clone()
            .validate(&asset_schema_rules, &resolver, &validation_config)
            .unwrap();
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Step {
    Key(String),
    Idx(usize),
}

type Path = Vec<Step>;

fn get_entry_at_path_mut<'a>(root: &'a mut Value, path: &Path) -> &'a mut Value {
    let mut curr = root;
    for p in path {
        match (curr, p) {
            (Value::Object(map), Step::Key(k)) => {
                curr = map.get_mut(k).unwrap();
            }
            (Value::Object(map), Step::Idx(i)) => {
                curr = map.values_mut().nth(*i).unwrap();
            }
            (Value::Array(arr), Step::Idx(i)) => {
                curr = arr.get_mut(*i).unwrap();
            }
            _ => {
                unreachable!()
            }
        }
    }
    curr
}

#[test]
fn validate_consignment_tapret_partner() {
    let scenario = Scenario::D;
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let base_consignment: Value = serde_json::from_reader(file).unwrap();
    let base_transfer = transfer_from_json_value(&base_consignment);
    let asset_schema = AssetSchema::from(base_transfer.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let wbundle_idx = 1;
    let bundle_path = vec![Step::Key(s!("bundles")), Step::Idx(wbundle_idx)];
    let partner_node_path = vec![
        Step::Key(s!("anchor")),
        Step::Key(s!("dbcProof")),
        Step::Key(s!("pathProof")),
        Step::Key(s!("partnerNode")),
    ];
    let spk_path = vec![
        Step::Key(s!("tx")),
        Step::Key(s!("output")),
        Step::Idx(0),
        Step::Key(s!("script_pubkey")),
    ];

    // ERROR: validation fails if unexpected partnerNode is provided
    // scriptPubKey is not updated according to the new DBC proof
    let partner_node = json!({
        "rightLeaf":{
            "version":192,
            "script":"6a20fefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefe"
        }
    });
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    let consignment = transfer_from_json_value(&json_consignment);
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    assert!(matches!(
        res.unwrap_err(),
        ValidationError::InvalidConsignment(Failure::SealsInvalid(_, _, _))
    ));

    // SUCCESS (PartnerNode::RightLeaf)
    let test_case = Case::SuccessRightLeaf;
    let partner_node = json!({
        "rightLeaf":{
            "version":192,
            "script":"6a20ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        }
    });
    let spk = json!("5120266a61fb2f5fc8f33bb2dd5a823fda4019ebd8fcc558b147343d14b65ed90416");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    res.unwrap();

    // SUCCESS (PartnerNode::RightLeaf with future version)
    let test_case = Case::RightLeafFutureVersion;
    let partner_node = json!({
        "rightLeaf":{
            "version": 4,
            "script":"6a20ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
        }
    });
    let spk = json!("5120ecc27afab08de13ae51763232588b580f98729fd2c042fdda1453f61bb5db4c5");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    res.unwrap();

    // ERROR deserialization (PartnerNode::RightLeaf with odd version)
    let partner_node = json!({
        "rightLeaf":{
            "version": 5,
            "script":"6a20fdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfd"
        }
    });
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    serde_json::from_str::<UncheckedTransfer>(&serde_json::to_string(&json_consignment).unwrap())
        .unwrap_err();

    // SUCCESS (PartnerNode::RightBranch)
    let test_case = Case::SuccessRightBranch;
    let partner_node = json!({
        "rightBranch": {
            "leftNodeHash": "cec6cd42645c3d426925940d320e3204fadba480aa7ffff98911d00f6e2124ff",
            "rightNodeHash": "fa02621f8168bda0ba049d71e82f1a341e38287c10127e009cd9e58c68e5050e"
        }
    });
    let spk = json!("51204d5833424e936fdcfc0a694903009a1a381d108b3c2093c1bb02095ba012a2ac");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    res.unwrap();

    // SUCCESS (PartnerNode::LeftNode)
    let test_case = Case::SuccessLeftNode;
    let partner_node = json!({
        "leftNode":"2fca1237a2b0915c3840cb035bf3d697c100bd131f1cacba734c46d2827dce90"
    });
    let spk = json!("512040bc8c42b3abf1cdcf021d704b382526867b8409edb51cca296fa9d372bc15e8");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    res.unwrap();

    // ERROR (PartnerNode::RightLeaf looks like a commitment)
    let test_case = Case::ErrorRightLeaf;
    let partner_node = json!({
        "rightLeaf": {
            "script": "50505050505050505050505050505050505050505050505050505050506a210000000000000000000000000000000000000000000000000000000000000000ff",
            "version": 192
        }
    });
    let spk = json!("5120c61bc67c5fbcd55870697490860b8b8e57f4b63221abdf3f463801bedc640fab");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    let wbundle = consignment.bundles[wbundle_idx].clone();
    assert_eq!(
        res.unwrap_err(),
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            wbundle.bundle.bundle_id(),
            wbundle.witness_id(),
            s!("the message is invalid since a valid commitment to it can't be created.")
        ))
    );

    // ERROR (PartnerNode::RightBranch's left branch looks like a commitment)
    // TODO: should this be an error?
    let test_case = Case::ErrorRightBranch;
    let partner_node = json!({
        "rightBranch": {
            "leftNodeHash": "50505050505050505050505050505050505050505050505050505050506a2100",
            "rightNodeHash": "b2c459126150e0d47063ea7b6d0474a24c39e25908aae5740dd4787b67c6e19a"
        }
    });
    let spk = json!("51207e8f35a70bb6c092135e8093eac763c2c80e349539536250733126e15b5d5491");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    let wbundle = consignment.bundles[wbundle_idx].clone();
    assert_eq!(
        res.unwrap_err(),
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            wbundle.bundle.bundle_id(),
            wbundle.witness_id(),
            s!("the message is invalid since a valid commitment to it can't be created.")
        ))
    );

    // ERROR (PartnerNode::RightLeaf should be on the left)
    let test_case = Case::UnorderedRightLeaf;
    let partner_node = json!({
        "rightLeaf":{
            "version":192,
            "script":"6a20fdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfdfd"
        }
    });
    let spk = json!("512039421e26fa3962eb4b270962ac292552bd34427c5a76d9f7abea11c0ec3f1915");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    let wbundle = consignment.bundles[wbundle_idx].clone();
    assert_eq!(
        res.unwrap_err(),
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            wbundle.bundle.bundle_id(),
            wbundle.witness_id(),
            s!("the message is invalid since a valid commitment to it can't be created.")
        ))
    );

    // ERROR (PartnerNode::RightBranch should be on the left)
    let test_case = Case::UnorderedRightBranch;
    let partner_node = json!({
        "rightBranch": {
            "leftNodeHash": "b2c459126150e0d47063ea7b6d0474a24c39e25908aae5740dd4787b67c6e19a",
            "rightNodeHash": "cec6cd42645c3d426925940d320e3204fadba480aa7ffff98911d00f6e2124ff"
        }
    });
    let spk = json!("512040bc8c42b3abf1cdcf021d704b382526867b8409edb51cca296fa9d372bc15e8");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    let wbundle = consignment.bundles[wbundle_idx].clone();
    assert_eq!(
        res.unwrap_err(),
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            wbundle.bundle.bundle_id(),
            wbundle.witness_id(),
            s!("the message is invalid since a valid commitment to it can't be created.")
        ))
    );

    // ERROR (PartnerNode::LeftNode should be on the left)
    let test_case = Case::UnorderedLeftNode;
    let partner_node = json!({
        "leftNode":"a1117afd36bd1195c7765e1fdecaa8ce511cce72874bfb8ff444639df34901af"
    });
    let spk = json!("51204d5833424e936fdcfc0a694903009a1a381d108b3c2093c1bb02095ba012a2ac");
    assert_eq!(
        gen_tapret_values(test_case),
        (partner_node.clone(), spk.clone())
    );
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(bundle_val, &partner_node_path) = partner_node;
    *get_entry_at_path_mut(bundle_val, &spk_path) = spk;
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    let wbundle = consignment.bundles[wbundle_idx].clone();
    assert_eq!(
        res.unwrap_err(),
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            wbundle.bundle.bundle_id(),
            wbundle.witness_id(),
            s!("the message is invalid since a valid commitment to it can't be created.")
        ))
    );

    // ERROR (CommitmentMismatch): provide wrong internal pk in the DBC proof
    let mut json_consignment = base_consignment.clone();
    let bundle_val = get_entry_at_path_mut(&mut json_consignment, &bundle_path);
    *get_entry_at_path_mut(
        bundle_val,
        &vec![
            Step::Key(s!("anchor")),
            Step::Key(s!("dbcProof")),
            Step::Key(s!("internalPk")),
        ],
    ) = "0101010101010101010101010101010101010101010101010101010101010101".into();
    let consignment = transfer_from_json_value(&json_consignment);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    let wbundle = consignment.bundles[wbundle_idx].clone();
    assert_eq!(
        res.unwrap_err(),
        ValidationError::InvalidConsignment(Failure::SealsInvalid(
            wbundle.bundle.bundle_id(),
            wbundle.witness_id(),
            s!("commitment doesn't match the message.")
        ))
    );
}

#[derive(Debug)]
enum Case {
    SuccessRightLeaf,
    SuccessRightBranch,
    SuccessLeftNode,
    ErrorRightLeaf,
    ErrorRightBranch,
    UnorderedRightLeaf,
    UnorderedRightBranch,
    UnorderedLeftNode,
    RightLeafFutureVersion,
}

fn gen_tapret_values(case: Case) -> (Value, Value) {
    let scenario = Scenario::D;
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let wbundle = consignment.bundles.last().unwrap().clone();
    let DbcProof::Tapret(tapret_proof) = wbundle.anchor.dbc_proof.clone() else {
        panic!()
    };
    let nonce = tapret_proof.path_proof.nonce();
    let protocol_id = mpc::ProtocolId::from(consignment.contract_id());
    let message = mpc::Message::from(wbundle.bundle.bundle_id());
    let commitment = wbundle
        .anchor
        .mpc_proof
        .convolve(protocol_id, message)
        .unwrap();
    let tapret_commitment = TapretCommitment::with(commitment, nonce);
    let script_commitment = tapret_commitment.commit();

    let commitment_leaf = script_commitment.tapscript_leaf_hash();
    let commitment_hash = TapNodeHash::from(commitment_leaf);
    let random_hash = "cec6cd42645c3d426925940d320e3204fadba480aa7ffff98911d00f6e2124ff";
    let mut val = 0xff;
    let partner = loop {
        let partner = match case {
            Case::SuccessRightLeaf | Case::UnorderedRightLeaf => {
                TapretNodePartner::RightLeaf(LeafScript {
                    version: LeafVersion::TapScript,
                    script: ScriptBuf::new_op_return([val; 32]),
                })
            }
            Case::SuccessRightBranch | Case::UnorderedRightBranch => {
                TapretNodePartner::RightBranch(TapretRightBranch::with(
                    TapNodeHash::from_str(random_hash).unwrap(),
                    TapNodeHash::from(TapLeafHash::from_script(
                        &ScriptBuf::new_op_return([val; 32]),
                        LeafVersion::TapScript,
                    )),
                ))
            }
            Case::SuccessLeftNode | Case::UnorderedLeftNode => {
                TapretNodePartner::LeftNode(TapNodeHash::from_node_hashes(
                    TapNodeHash::from_str(random_hash).unwrap(),
                    TapNodeHash::from(TapLeafHash::from_script(
                        &ScriptBuf::new_op_return([val; 32]),
                        LeafVersion::TapScript,
                    )),
                ))
            }
            Case::ErrorRightLeaf => {
                let alt_commitment =
                    TapretCommitment::with(mpc::Commitment::strict_dumb(), val).commit();
                TapretNodePartner::RightLeaf(LeafScript {
                    version: LeafVersion::TapScript,
                    script: alt_commitment,
                })
            }
            Case::ErrorRightBranch => TapretNodePartner::RightBranch(TapretRightBranch::with(
                TapNodeHash::from_str(
                    "50505050505050505050505050505050505050505050505050505050506a2100",
                )
                .unwrap(),
                TapNodeHash::from(TapLeafHash::from_script(
                    &ScriptBuf::new_op_return([val; 32]),
                    LeafVersion::TapScript,
                )),
            )),
            Case::RightLeafFutureVersion => TapretNodePartner::RightLeaf(LeafScript {
                version: LeafVersion::from_consensus(4).unwrap(),
                script: ScriptBuf::new_op_return([val; 32]),
            }),
        };

        if match case {
            Case::SuccessRightLeaf
            | Case::SuccessRightBranch
            | Case::ErrorRightLeaf
            | Case::ErrorRightBranch
            | Case::UnorderedLeftNode
            | Case::RightLeafFutureVersion => partner.tap_node_hash() > commitment_hash,
            Case::SuccessLeftNode | Case::UnorderedRightLeaf | Case::UnorderedRightBranch => {
                partner.tap_node_hash() < commitment_hash
            }
        } {
            break partner;
        };
        val -= 1;
    };
    let merkle_root: TapNodeHash =
        TapNodeHash::from_node_hashes(commitment_hash, partner.tap_node_hash());
    let new_spk = ScriptBuf::new_p2tr(
        &BitcoinSecp256k1::new(),
        tapret_proof.internal_pk,
        Some(merkle_root),
    );
    (
        serde_json::from_str::<Value>(&serde_json::to_string(&partner).unwrap()).unwrap(),
        serde_json::from_str::<Value>(&serde_json::to_string(&new_spk).unwrap()).unwrap(),
    )
}

#[test]
fn validate_consignment_strict_roundtrip() {
    for scenario in Scenario::iter() {
        let cons_json = get_consignment_from_json(&format!("consignment_{scenario}"));
        let cons_strict_path = format!("tests/fixtures/consignment_{scenario}.rgb");
        let cons_strict =
            Transfer::strict_deserialize_from_file::<{ usize::MAX }>(&cons_strict_path).unwrap();
        let json_bytes = cons_json
            .to_strict_serialized::<{ usize::MAX }>()
            .unwrap()
            .release();
        let strict_bytes = std::fs::read(&cons_strict_path).unwrap();
        // JSON and .rgb fixtures must encode the same consignment
        assert_eq!(cons_json, cons_strict);
        assert_eq!(json_bytes, strict_bytes);
        // .rgb must be a canonical strict encoding (round-trip)
        assert_eq!(
            cons_strict
                .to_strict_serialized::<{ usize::MAX }>()
                .unwrap()
                .release(),
            strict_bytes
        );
    }
}

#[test]
fn consignment_json_roundtrip() {
    for scenario in Scenario::iter() {
        let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
        // Consignment has no Deserialize impl: serde ingestion goes through
        // UncheckedTransfer, and into_checked enforces the structural bounds
        let roundtripped = serde_json::from_str::<UncheckedTransfer>(
            &serde_json::to_string(&consignment).unwrap(),
        )
        .unwrap()
        .into_checked()
        .unwrap();
        assert_eq!(consignment, roundtripped);
    }
}

#[test]
fn validate_consignment_contract_state_evolve_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    #[derive(Clone, Eq, PartialEq, Debug, StrictType, StrictDumb, StrictEncode, StrictDecode)]
    #[strict_type(lib = "dumb")]
    struct SmallContractState();
    struct DumbGlobalsIter();
    impl Iterator for DumbGlobalsIter {
        type Item = GlobalStateEntry;
        fn next(&mut self) -> Option<Self::Item> {
            None
        }
    }
    impl GlobalsIter for DumbGlobalsIter {
        fn at_depth(&self, _depth: usize) -> Option<Self::Item> {
            None
        }
    }
    impl ContractStateAccess for SmallContractState {
        fn data(
            &self,
            _outpoint: Outpoint,
            _ty: AssignmentType,
        ) -> impl DoubleEndedIterator<Item = impl Borrow<RevealedData>> {
            Vec::<RevealedData>::new().into_iter()
        }
        fn global(
            &self,
            _ty: GlobalStateType,
        ) -> Result<impl GlobalsIter<Item = impl Borrow<GlobalStateEntry>>, UnknownGlobalStateType>
        {
            Ok(DumbGlobalsIter())
        }
        fn rights(&self, _outpoint: Outpoint, _ty: AssignmentType) -> u32 {
            0
        }
        fn fungible(
            &self,
            _outpoint: Outpoint,
            _ty: AssignmentType,
        ) -> impl DoubleEndedIterator<Item = FungibleState> {
            Vec::<FungibleState>::new().into_iter()
        }
    }
    impl ContractStateEvolve for SmallContractState {
        type Error = confinement::Error;
        type Context<'ctx> = String;
        fn init(_context: Self::Context<'_>) -> Self {
            Self()
        }
        fn evolve_state(&mut self, _op: rgb::vm::OrdOpRef) -> Result<(), Self::Error> {
            Err(confinement::Error::OutOfBoundary { index: 3, len: 6 })
        }
    }
    let res = Validator::<SmallContractState, _>::validate(
        &consignment,
        &asset_schema_rules,
        &resolver,
        "".to_string(),
        &validation_config,
    )
    .unwrap_err();
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::ContractStateFilled(_))
    ));
}

/// Phase 1 can be driven one bundle at a time, handing each witness to the
/// caller as soon as the bundle it belongs to has been validated - so a caller
/// can start resolving while the remaining bundles are still being checked.
#[test]
fn phase_1_hands_out_witnesses_as_it_walks() {
    let scenario = Scenario::A;
    let resolver = scenario.resolver();
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        build_opouts_dag: true,
        ..Default::default()
    };
    let context = (asset_schema_rules.schema(), consignment.contract_id());

    let mut validator = Validator::<FilteredContractState<UnfilteredContractState>, _>::start(
        &consignment,
        &asset_schema_rules,
        context,
        &validation_config,
    )
    .unwrap();

    // witnesses arrive during the walk, not at the end of it
    let mut stepped = Vec::new();
    while let Some(task) = validator.next_bundle().unwrap() {
        stepped.push(task);
    }
    assert!(!stepped.is_empty());
    // calling it again once exhausted is harmless
    assert!(validator.next_bundle().unwrap().is_none());

    let mut pending = validator.finish().unwrap();

    // the DAG is readable before a single witness has been resolved: this is
    // what lets a caller decide whether resolving is worth it at all
    assert!(pending.dag_data_opt.is_some());
    assert!(!pending.is_resolved());
    assert_eq!(
        pending.unresolved_witnesses().count(),
        stepped.len(),
        "every witness handed out during the walk is still outstanding"
    );

    // resolve them the way a parallel driver would: resolve off the task, feed
    // the answer back
    let checked = pending.check_resolver(&resolver).unwrap();
    let tasks = pending.unresolved_witnesses().collect::<Vec<_>>();
    for task in tasks {
        let res = task.resolve(&checked).unwrap();
        assert_eq!(res.txid, task.txid);
        pending.resolve_witness(res).unwrap();
    }
    assert!(pending.is_resolved());

    let status = pending.finalize();
    assert_eq!(status.validity(), Validity::Valid);
}

/// Driving the walk and letting `finish` do it must give the same result.
#[test]
fn stepping_matches_running_phase_1_in_one_go() {
    let scenario = Scenario::A;
    let resolver = scenario.resolver();
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let context = (asset_schema_rules.schema(), consignment.contract_id());

    let mut stepped = Validator::<FilteredContractState<UnfilteredContractState>, _>::start(
        &consignment,
        &asset_schema_rules,
        context,
        &validation_config,
    )
    .unwrap();
    let mut stepped_txids = Vec::new();
    while let Some(task) = stepped.next_bundle().unwrap() {
        stepped_txids.push(task.txid);
    }
    let mut stepped = stepped.finish().unwrap();

    let context = (asset_schema_rules.schema(), consignment.contract_id());
    let mut in_one_go =
        Validator::<FilteredContractState<UnfilteredContractState>, _>::validate_deterministic(
            &consignment,
            &asset_schema_rules,
            context,
            &validation_config,
        )
        .unwrap();

    assert_eq!(
        in_one_go
            .unresolved_witnesses()
            .map(|t| t.txid)
            .collect::<std::collections::BTreeSet<_>>(),
        stepped_txids
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>()
    );

    stepped.resolve_all(&resolver).unwrap();
    in_one_go.resolve_all(&resolver).unwrap();
    assert_eq!(
        stepped.finalize().validity(),
        in_one_go.finalize().validity()
    );
}

/// `finalize` refuses to conclude while witnesses are outstanding: nothing has
/// been checked against a chain yet, so getting there is a bug in the caller.
#[test]
#[should_panic(expected = "still outstanding")]
fn finalize_refuses_unresolved_witnesses() {
    let scenario = Scenario::A;
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let context = (asset_schema_rules.schema(), consignment.contract_id());

    let pending =
        Validator::<FilteredContractState<UnfilteredContractState>, _>::validate_deterministic(
            &consignment,
            &asset_schema_rules,
            context,
            &validation_config,
        )
        .unwrap();
    assert!(!pending.is_resolved());

    pending.finalize();
}

/// An answer for a witness this validation never asked about is rejected rather
/// than silently ignored.
#[test]
fn resolve_witness_rejects_an_unknown_witness() {
    let scenario = Scenario::A;
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let context = (asset_schema_rules.schema(), consignment.contract_id());

    let mut pending =
        Validator::<FilteredContractState<UnfilteredContractState>, _>::validate_deterministic(
            &consignment,
            &asset_schema_rules,
            context,
            &validation_config,
        )
        .unwrap();

    let stranger =
        Txid::from_str("0909090909090909090909090909090909090909090909090909090909090909").unwrap();
    let res = pending
        .resolve_witness(WitnessResolution {
            txid: stranger,
            ord: WitnessOrd::Tentative,
            warning: None,
        })
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::UnknownWitness(stranger));
}

/// An archived witness fails when it is reported, not at finalization, so the
/// caller can stop instead of resolving the rest first.
#[test]
fn resolve_witness_rejects_an_archived_witness() {
    let scenario = Scenario::A;
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let context = (asset_schema_rules.schema(), consignment.contract_id());

    let mut pending =
        Validator::<FilteredContractState<UnfilteredContractState>, _>::validate_deterministic(
            &consignment,
            &asset_schema_rules,
            context,
            &validation_config,
        )
        .unwrap();

    let task = pending.unresolved_witnesses().next().unwrap();
    let res = pending
        .resolve_witness(WitnessResolution {
            txid: task.txid,
            ord: WitnessOrd::Archived,
            warning: None,
        })
        .unwrap_err();
    dbg!(&res);
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::SealNoPubWitness(_, txid)) if txid == task.txid
    ));
}

/// A caller can stop at the first witness with an unsafe height instead of
/// resolving the rest and reading the warning afterwards.
///
/// Consensus only warns, because it cannot know the caller's policy - rgb-lib,
/// for one, wants to stop. So the verdict is reported per witness and the
/// caller decides.
#[test]
fn caller_can_stop_at_the_first_unsafe_witness() {
    let scenario = Scenario::A;
    let resolver = scenario.resolver();
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    // every witness in the fixture is mined above height 1, so all are unsafe
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        safe_height: Some(NonZeroU32::new(1).unwrap()),
        ..Default::default()
    };
    let context = (asset_schema_rules.schema(), consignment.contract_id());

    let mut pending =
        Validator::<FilteredContractState<UnfilteredContractState>, _>::validate_deterministic(
            &consignment,
            &asset_schema_rules,
            context,
            &validation_config,
        )
        .unwrap();
    let total = pending.unresolved_witnesses().count();
    assert!(
        total > 1,
        "fixture must have more than one witness to prove we stopped early"
    );

    let checked = pending.check_resolver(&resolver).unwrap();
    let mut resolved = 0;
    let mut stopped = false;
    loop {
        let next = pending.unresolved_witnesses().next();
        let Some(task) = next else { break };
        let res = task.resolve(&checked).unwrap();
        resolved += 1;
        if pending.resolve_witness(res).unwrap() == WitnessSafety::Unsafe {
            stopped = true;
            break;
        }
    }
    assert!(
        stopped,
        "the fixture's witnesses should be above the safe height"
    );
    assert_eq!(resolved, 1, "stopped on the first one");
    assert_eq!(pending.unresolved_witnesses().count(), total - 1);
    assert!(!pending.is_resolved());

    // and had we not stopped, consensus would only have warned
    pending.resolve_all(&resolver).unwrap();
    let status = pending.finalize();
    assert_eq!(status.validity(), Validity::Warnings);
    assert!(matches!(status.warnings[0], Warning::UnsafeHistory(_)));
}

/// Phase 1's output is plain data: it can be serialised, shipped to whatever
/// resolves the witnesses, and finalised somewhere else entirely.
///
/// This is what the `Validator` itself cannot do - it holds an `Rc<RefCell<S>>`
/// and borrows both the consignment and the schema rules.
#[test]
fn pending_validation_survives_a_round_trip_through_disk() {
    let scenario = Scenario::A;
    let resolver = scenario.resolver();
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        build_opouts_dag: true,
        ..Default::default()
    };
    let context = (asset_schema_rules.schema(), consignment.contract_id());

    let pending =
        Validator::<FilteredContractState<UnfilteredContractState>, _>::validate_deterministic(
            &consignment,
            &asset_schema_rules,
            context,
            &validation_config,
        )
        .unwrap();
    let outstanding = pending.unresolved_witnesses().count();
    assert!(outstanding > 0);
    // the DAG rides along, so the far side can inspect it before resolving
    assert!(pending.dag_data_opt.is_some());

    // it is also `Send`: the whole point of it owning its data rather than
    // borrowing the consignment the way `Validator` does
    fn assert_send<T: Send>() {}
    assert_send::<PendingValidation>();

    let json = serde_json::to_string(&pending).unwrap();
    drop(pending);
    drop(consignment);

    // ...somewhere else, with no consignment and no validator in sight
    let mut shipped: PendingValidation = serde_json::from_str(&json).unwrap();
    assert_eq!(shipped.unresolved_witnesses().count(), outstanding);
    assert!(shipped.dag_data_opt.is_some());

    shipped.resolve_all(&resolver).unwrap();
    assert!(shipped.is_resolved());
    assert_eq!(shipped.finalize().validity(), Validity::Valid);
}

#[test]
fn validate_consignment_opout_dag() {
    let scenario = Scenario::C;
    let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(consignment.schema_id());
    let asset_schema_rules = asset_schema.schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        build_opouts_dag: true,
        ..Default::default()
    };
    let res = consignment
        .clone()
        .validate(
            &asset_schema_rules,
            &scenario.resolver(),
            &validation_config,
        )
        .unwrap();
    let (opouts_dag, opouts_map) = res.validation_status().dag_data_opt.clone().unwrap();
    dbg!(&opouts_dag);
    // 8 nodes:
    // - 2 genesis opout
    // - 2 opouts after first extra transition
    // - 2 opouts after transition that isolates inflation right
    // - 2 opouts after inflation operation
    // - 1 opout after burn transition (change)
    assert_eq!(opouts_map.len(), 9);
    assert_eq!(opouts_dag.node_count(), 9);
    // 8 edges:
    // - 4 extra transition (2 in, 2 out)
    // - 2 inflation right isolation (2 x 1 in, 1 out)
    // - 2 inflation transition (1 in, 2 out)
    // - 3 burn transition (3 in, 1 out)
    assert_eq!(opouts_dag.edge_count(), 11);

    // genesis opouts don't have parents
    let genesis_opid: OpId = (*consignment.contract_id()).into();
    let genesis_asset_opout = Opout::new(genesis_opid, OS_ASSET, 0);
    let genesis_infl_opout = Opout::new(genesis_opid, OS_INFLATION, 0);
    let genesis_asset_idx = *opouts_map.get(&genesis_asset_opout).unwrap();
    let mut parents = opouts_dag.parents(genesis_asset_idx);
    assert!(parents.walk_next(&opouts_dag).is_none());
    let genesis_infl_idx = *opouts_map.get(&genesis_infl_opout).unwrap();
    let mut parents = opouts_dag.parents(genesis_infl_idx);
    assert!(parents.walk_next(&opouts_dag).is_none());

    let mut inflation_opid = None;
    let mut burn_opid = None;
    for KnownTransition { opid, transition } in consignment
        .bundles
        .iter()
        .map(|wb| wb.bundle.known_transitions.first().unwrap())
    {
        match transition.transition_type {
            TS_INFLATION => {
                inflation_opid = Some(*opid);
            }
            TS_BURN => {
                burn_opid = Some(*opid);
            }
            _ => {}
        };
    }
    let infl_ass_opout = Opout::new(inflation_opid.unwrap(), OS_ASSET, 0);
    let infl_chg_opout = Opout::new(inflation_opid.unwrap(), OS_INFLATION, 0);
    let burn_chg_opout = Opout::new(burn_opid.unwrap(), OS_ASSET, 0);
    assert_eq!(
        opouts_map[&burn_chg_opout],
        opouts_dag
            .children(*opouts_map.get(&infl_ass_opout).unwrap())
            .walk_next(&opouts_dag)
            .unwrap()
            .1
    );
    assert_eq!(
        opouts_map[&burn_chg_opout],
        opouts_dag
            .children(*opouts_map.get(&infl_chg_opout).unwrap())
            .walk_next(&opouts_dag)
            .unwrap()
            .1
    );

    // each opout is spent by a single transition
    for node_idx in opouts_map.values() {
        let children_opids = opouts_dag
            .children(*node_idx)
            .iter(&opouts_dag)
            .map(|child1| {
                opouts_map
                    .iter()
                    .find(|(_, idx)| **idx == child1.1)
                    .unwrap()
                    .0
                    .op
            })
            .collect::<HashSet<_>>();
        assert!(children_opids.len() <= 1);
    }
}

#[test]
fn validate_consignment_unknown_rgbisa_opcode() {
    let scenario = Scenario::B;
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let asset_schema = AssetSchema::from(base_consignment.schema_id());
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    let mut consignment = base_consignment.clone();
    // add unknown opcode to script
    let (_, mut script) = asset_schema.scripts().into_iter().next().unwrap();
    let ret = script.code.pop().unwrap();
    script.code.push(0b11_010_101).unwrap(); // unknown opcode
    script.code.push(ret).unwrap();
    let lib_id = script.id();
    let tampered_scripts = Scripts::from_checked(bmap![lib_id => script]);
    // update schema and genesis
    let mut schema = asset_schema.schema();
    let mut validator = schema.genesis.validator.unwrap();
    validator.lib = lib_id;
    schema.genesis.validator = Some(validator);
    schema.transitions.values_mut().for_each(|t| {
        // update transitions validator otherwise validation fails before running aluvm
        let mut validator = t.transition_schema.validator.unwrap();
        validator.lib = lib_id;
        t.transition_schema.validator = Some(validator);
    });
    let old_genesis_opid = consignment.genesis.id();
    consignment.genesis.schema_id = schema.schema_id();
    let genesis_opid = consignment.genesis.id();
    let contract_id = consignment.contract_id();
    let mut bundles = consignment.bundles.release();
    update_transition_children(
        &mut bundles,
        map! {old_genesis_opid => genesis_opid},
        map! {},
        Some(contract_id),
    );
    consignment.bundles = LargeVec::from_checked(bundles);

    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(
            &SchemaDefinition::new(
                schema.clone(),
                asset_schema.libs(),
                tampered_scripts.clone(),
            )
            .verify()
            .unwrap(),
            &resolver,
            &validation_config,
        )
        .unwrap_err();
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ScriptFailure(
            genesis_opid,
            Some(ERRNO_ISSUED_MISMATCH),
            None
        ))
    );
}

#[test]
fn from_str_enforces_confinement() {
    let scenario = Scenario::A;
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let base_consignment: Value = serde_json::from_reader(file).unwrap();

    serde_json::from_str::<UncheckedTransfer>(&serde_json::to_string(&base_consignment).unwrap())
        .unwrap()
        .into_checked()
        .unwrap();

    // SecretSeals wraps a NonEmptyOrdSet, so an empty terminal breaks its lower bound
    let mut consignment = base_consignment.clone();
    let terminals = consignment
        .get_mut("terminals")
        .unwrap()
        .as_object_mut()
        .unwrap();
    let bundle_id = terminals.keys().next().unwrap().clone();
    terminals.insert(bundle_id, Value::Array(vec![]));
    let err =
        serde_json::from_str::<UncheckedTransfer>(&serde_json::to_string(&consignment).unwrap())
            .unwrap_err();
    assert!(
        err.to_string()
            .contains("collection size 0 less than lower boundary"),
        "expected a confinement bound error, got: {err}"
    );
}

#[test]
fn validate_consignment_mpc_proof_depth_overflow() {
    let scenario = Scenario::B;
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let mut consignment: Value = serde_json::from_reader(file).unwrap();

    // MerkleProof::path is a Confined<_, 0, 31>, so a 32-hash path overflows the confinement bound.
    let hash = format!("{:064x}", 0);
    let overlong_path = Value::Array((0..32).map(|_| Value::String(hash.clone())).collect());
    *consignment
        .get_mut("bundles")
        .unwrap()
        .as_array_mut()
        .unwrap()
        .last_mut()
        .unwrap()
        .get_mut("anchor")
        .unwrap()
        .get_mut("mpcProof")
        .unwrap()
        .get_mut("path")
        .unwrap() = overlong_path;
    let malicious_json = serde_json::to_string(&consignment).unwrap();

    let err = serde_json::from_str::<UncheckedTransfer>(&malicious_json).unwrap_err();
    assert!(
        err.to_string()
            .contains("operation results in collection size 32 exceeding 31"),
        "expected a confinement bound error, got: {err}"
    );
}

#[test]
fn into_checked_enforces_custom_strict_bounds() {
    let scenario = Scenario::A;
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let base_consignment: Value = serde_json::from_reader(file).unwrap();

    serde_json::from_str::<UncheckedTransfer>(&serde_json::to_string(&base_consignment).unwrap())
        .unwrap()
        .into_checked()
        .unwrap();

    // Ffv is validated only by its hand-written StrictDecode, which rejects any non-zero
    // fast-forward version; serde parses it as a plain u16, so into_checked is what catches it
    let mut consignment = base_consignment.clone();
    *consignment
        .get_mut("genesis")
        .unwrap()
        .get_mut("ffv")
        .unwrap() = Value::from(1u16);
    let unchecked =
        serde_json::from_str::<UncheckedTransfer>(&serde_json::to_string(&consignment).unwrap())
            .unwrap();
    assert!(matches!(
        unchecked.into_checked(),
        Err(ConsignmentConstraintError::Deserialize(_))
    ));
}

#[test]
fn evolve_state_on_operations_without_validator() {
    let global_type = GlobalStateType::with(42);
    let code = rgbasm! {
        put     a32[0],0;
        ldc     global_type,a32[0],s16[0];
        test;
        ret;
    };
    let lib = Lib::assemble::<Instr<RgbIsa<FilteredContractState>>>(&code)
        .expect("wrong BFA transfer validation script");

    let types = StandardTypes::with(rgb_contract_stl());
    let schema = Schema {
        ffv: zero!(),
        name: tn!("BridgedFungibleAsset"),
        meta_types: none!(),
        global_types: tiny_bmap! {
             global_type => GlobalDetails {
                global_state_schema: GlobalStateSchema::once(types.get("RGBContract.Amount")),
                name: fname!("someGlobal"),
            },
        },
        owned_types: tiny_bmap! {
            OS_ASSET => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
                name: fname!("assetOwner"),
                default_transition: TS_TRANSFER,
            },
        },
        // NOTE: Genesis updates global state and has no validator
        genesis: GenesisSchema {
            metadata: none!(),
            globals: tiny_bmap! {
                global_type => Occurrences::NoneOrOnce,
            },

            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            validator: None,
        },
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: none!(),
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::NoneOrMore,
                    },
                    assignments: none!(),
                    validator: Some(LibSite::with(0, lib.id())),
                },
                name: fname!("transfer"),
            },
        },
        default_assignment: Some(OS_ASSET),
    };
    let scripts = Confined::from_checked(bmap! {lib.id() => lib});
    let schema_scripts = scripts.clone();
    let chain_net = ChainNet::BitcoinRegtest;
    let outpoint =
        Outpoint::from_str("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc:0")
            .unwrap();
    let seal = BuilderSeal::Revealed(GenesisSeal::rand_from(outpoint));
    let contract_consignment = ContractBuilder::with(
        strict_dumb!(),
        SchemaDefinition::new(schema.clone(), types.libs(), scripts)
            .verify()
            .unwrap(),
        chain_net,
    )
    .add_global_state("someGlobal", Amount::from(12u64))
    .unwrap()
    .add_fungible_state("assetOwner", seal, 14u64)
    .unwrap()
    .issue_contract_raw(42)
    .unwrap()
    .into_consignment();
    let contract_id = contract_consignment.contract_id();
    let opout = Opout::new(contract_consignment.genesis().id(), OS_ASSET, 0);
    let transition = Transition {
        contract_id,
        transition_type: TS_TRANSFER,
        inputs: NonEmptyOrdSet::with(opout).into(),
        ..strict_dumb!()
    };
    let opid = transition.id();
    let bundle = TransitionBundle {
        input_map: NonEmptyOrdMap::with_key_value(opout, opid),
        known_transitions: NonEmptyVec::with(KnownTransition::new(opid, transition)),
    };
    let mut psbt = Psbt::from_unsigned_tx(Transaction {
        version: Version::ONE,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0),
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: bitcoin::Amount::ZERO,
            script_pubkey: ScriptBuf::new_op_return([]),
        }],
    })
    .unwrap();
    psbt.inputs.get_mut(0).unwrap().witness_utxo = Some(TxOut {
        value: bitcoin::Amount::from_sat(1000),
        script_pubkey: ScriptBuf::new_p2a(),
    });
    let protocol_id = mpc::ProtocolId::from(contract_id);
    psbt.outputs.get_mut(0).unwrap().set_opret_host();
    psbt.outputs
        .get_mut(0)
        .unwrap()
        .set_mpc_message(protocol_id, mpc::Message::from(bundle.bundle_id()))
        .unwrap();
    let (commitment, proof) = psbt.outputs.get_mut(0).unwrap().mpc_commit().unwrap();
    psbt.outputs
        .get_mut(0)
        .unwrap()
        .opret_commit(commitment)
        .unwrap();
    psbt.set_opret_commitment(0);
    let tx = psbt.extract_tx().unwrap();
    let anchor = Anchor::new(
        proof.to_merkle_proof(protocol_id).unwrap(),
        DbcProof::Opret(OpretProof::strict_dumb()),
    );
    let wbundle = WitnessBundle::with(tx, anchor, bundle);
    let consignment = Consignment::<true> {
        transfer: true,
        bundles: Confined::from_checked(vec![wbundle]),
        genesis: contract_consignment.genesis,
        ..strict_dumb!()
    };
    let validation_config = ValidationConfig {
        chain_net,
        ..Default::default()
    };
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    consignment
        .clone()
        .validate(
            &SchemaDefinition::new(schema.clone(), types.libs(), schema_scripts.clone())
                .verify()
                .unwrap(),
            &resolver,
            &validation_config,
        )
        .unwrap();
}

/// Rebuild the LNPBP-81 recursion from public constructors, to reach the depth-1 subtree roots
fn merklize_subtree(
    leaves: &[MerkleHash],
    depth: u8,
    branch_width: u32,
    base_width: u32,
) -> MerkleHash {
    if branch_width <= 2 {
        return match (leaves.first(), leaves.get(1)) {
            (None, None) => MerkleHash::void(depth, base_width),
            (Some(branch), None) => MerkleHash::single(depth, base_width, *branch),
            (Some(branch1), Some(branch2)) => {
                MerkleHash::branches(depth, base_width, *branch1, *branch2)
            }
            (None, Some(_)) => unreachable!(),
        };
    }
    let div = (branch_width / 2 + branch_width % 2) as usize;
    let (left, right) = leaves.split_at(div.min(leaves.len()));
    MerkleHash::branches(
        depth,
        base_width,
        merklize_subtree(left, depth + 1, div as u32, base_width),
        merklize_subtree(right, depth + 1, branch_width - div as u32, base_width),
    )
}

/// Build a transfer whose transition carries `globals` under global state type 2
fn transfer_with_globals(
    global_type: GlobalStateType,
    blob_sem_id: SemId,
    libs: TypeLibs,
    globals: rgb::GlobalState,
) -> (Transfer, SchemaRules) {
    let schema = Schema {
        ffv: zero!(),
        name: tn!("BlobAsset"),
        meta_types: none!(),
        global_types: tiny_bmap! {
             global_type => GlobalDetails {
                global_state_schema: GlobalStateSchema::many(blob_sem_id),
                name: fname!("someGlobal"),
            },
        },
        owned_types: tiny_bmap! {
            OS_ASSET => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
                name: fname!("assetOwner"),
                default_transition: TS_TRANSFER,
            },
        },
        genesis: GenesisSchema {
            metadata: none!(),
            globals: none!(),
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            validator: None,
        },
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: tiny_bmap! {
                        global_type => Occurrences::NoneOrMore,
                    },
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::NoneOrMore,
                    },
                    assignments: none!(),
                    validator: None,
                },
                name: fname!("transfer"),
            },
        },
        default_assignment: Some(OS_ASSET),
    };

    let chain_net = ChainNet::BitcoinRegtest;
    let outpoint =
        Outpoint::from_str("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc:0")
            .unwrap();
    let seal = BuilderSeal::Revealed(GenesisSeal::rand_from(outpoint));
    let rules = SchemaDefinition::new(schema, libs, Confined::from_checked(bmap! {}))
        .verify()
        .unwrap();
    let contract_consignment = ContractBuilder::with(strict_dumb!(), rules.clone(), chain_net)
        .add_fungible_state("assetOwner", seal, 14u64)
        .unwrap()
        .issue_contract_raw(42)
        .unwrap()
        .into_consignment();
    let contract_id = contract_consignment.contract_id();
    let opout = Opout::new(contract_consignment.genesis().id(), OS_ASSET, 0);

    let transition = Transition {
        contract_id,
        transition_type: TS_TRANSFER,
        globals,
        inputs: NonEmptyOrdSet::with(opout).into(),
        ..strict_dumb!()
    };
    let opid = transition.id();
    let bundle = TransitionBundle {
        input_map: NonEmptyOrdMap::with_key_value(opout, opid),
        known_transitions: NonEmptyVec::with(KnownTransition::new(opid, transition)),
    };
    let mut psbt = Psbt::from_unsigned_tx(Transaction {
        version: Version::ONE,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0),
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: bitcoin::Amount::ZERO,
            script_pubkey: ScriptBuf::new_op_return([]),
        }],
    })
    .unwrap();
    psbt.inputs.get_mut(0).unwrap().witness_utxo = Some(TxOut {
        value: bitcoin::Amount::from_sat(1000),
        script_pubkey: ScriptBuf::new_p2a(),
    });
    let protocol_id = mpc::ProtocolId::from(contract_id);
    psbt.outputs.get_mut(0).unwrap().set_opret_host();
    psbt.outputs
        .get_mut(0)
        .unwrap()
        .set_mpc_message(protocol_id, mpc::Message::from(bundle.bundle_id()))
        .unwrap();
    let (commitment, proof) = psbt.outputs.get_mut(0).unwrap().mpc_commit().unwrap();
    psbt.outputs
        .get_mut(0)
        .unwrap()
        .opret_commit(commitment)
        .unwrap();
    psbt.set_opret_commitment(0);
    let tx = psbt.extract_tx().unwrap();
    let anchor = Anchor::new(
        proof.to_merkle_proof(protocol_id).unwrap(),
        DbcProof::Opret(OpretProof::strict_dumb()),
    );
    let wbundle = WitnessBundle::with(tx, anchor, bundle);
    let transfer = Consignment::<true> {
        transfer: true,
        bundles: Confined::from_checked(vec![wbundle]),
        genesis: contract_consignment.genesis,
        ..strict_dumb!()
    };
    (transfer, rules)
}

#[test]
#[ignore = "merkle leaf and node preimages are not domain-separated"]
fn validate_consignment_substituted_operation() {
    #[derive(Clone, Debug, StrictType, StrictEncode, StrictDecode)]
    #[strict_type(lib = "BlobTest")]
    struct Blob94([u8; 94]);
    impl StrictDumb for Blob94 {
        fn strict_dumb() -> Self {
            Self([0; 94])
        }
    }

    // state type 2 supplies NodeBranching::Branch and a zero depth, a 94-byte blob supplies a
    // base width of 94 and, after 30 zero bytes, both child hashes
    let global_type = GlobalStateType::with(2);
    let blob_lib = LibBuilder::with(libname!("BlobTest"), [std_stl().to_dependency_types()])
        .transpile::<Blob94>()
        .compile()
        .unwrap();
    let types = StandardTypes::with(blob_lib);
    let blob_sem_id = types.get("Blob94");

    let mut many = rgb::GlobalState::default();
    let mut leaves = vec![];
    for i in 0u16..94 {
        let mut blob = vec![0u8; 94];
        blob[..2].copy_from_slice(&i.to_le_bytes());
        let item = RevealedData::new(SmallBlob::from_checked(blob));
        leaves.push(
            GlobalCommitment {
                ty: global_type,
                state: item.clone(),
            }
            .commit_id(),
        );
        many.add_state(global_type, item).unwrap();
    }

    let (consignment, rules) =
        transfer_with_globals(global_type, blob_sem_id, types.libs(), many.clone());
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    consignment
        .clone()
        .validate(&rules, &resolver, &validation_config)
        .unwrap();

    // one item whose blob reproduces the node preimage of the 94-item tree
    let (left, right) = leaves.split_at(47);
    let mut blob = vec![0u8; 30];
    blob.extend_from_slice(merklize_subtree(left, 1, 47, 94).as_slice());
    blob.extend_from_slice(merklize_subtree(right, 1, 47, 94).as_slice());
    let mut one = rgb::GlobalState::default();
    one.add_state(
        global_type,
        RevealedData::new(SmallBlob::from_checked(blob)),
    )
    .unwrap();
    assert_ne!(many, one);

    // swap in the substituted operation, leaving the bundle, anchor and witness untouched
    let mut substituted = consignment.clone();
    let mut bundles = substituted.bundles.release();
    let known = bundles[0].bundle.known_transitions.first().unwrap().clone();
    let mut transition = known.transition.clone();
    transition.globals = one;
    bundles[0].bundle.known_transitions =
        NonEmptyVec::with(KnownTransition::new(known.opid, transition));
    substituted.bundles = LargeVec::from_checked(bundles);

    let resolver = OfflineResolver {
        consignment: &substituted,
    };
    let res = substituted
        .clone()
        .validate(&rules, &resolver, &validation_config);
    assert!(
        res.is_err(),
        "witness commitment does not bind the operation content"
    );
}

/// `cng 42,a8[0]; ret;` as raw bytecode, so the test does not depend on the assembler
const CNG_A8: &[u8] = &[0xc2, 0x2a, 0x00, 0x00, 0x07];

const CHILD_CASE: &str = "RGB_TESTS_CHILD_CASE";

/// Re-run `case` in a child process, so that a hard crash cannot take the test suite down.
/// Returns `None` in the child, which then has to run the case itself.
fn child_case(case: &str) -> Option<bool> {
    if std::env::var(CHILD_CASE).as_deref() == Ok(case) {
        return None;
    }
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "--include-ignored", "--test-threads", "1", case])
        .env(CHILD_CASE, case)
        .stdout(Stdio::null())
        .status()
        .unwrap();
    Some(status.success())
}

/// Build a transfer under a schema whose transition validator is the given bytecode
fn scripted_transfer(code: Vec<u8>, global_items: u16) -> (Transfer, SchemaRules) {
    let global_type = GlobalStateType::with(42);
    let lib = Lib::with("ALU", code, vec![], none!()).unwrap();

    let types = StandardTypes::with(rgb_contract_stl());
    let schema = Schema {
        ffv: zero!(),
        name: tn!("ScriptedAsset"),
        meta_types: none!(),
        global_types: tiny_bmap! {
             global_type => GlobalDetails {
                global_state_schema: GlobalStateSchema::many(types.get("RGBContract.Amount")),
                name: fname!("someGlobal"),
            },
        },
        owned_types: tiny_bmap! {
            OS_ASSET => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
                name: fname!("assetOwner"),
                default_transition: TS_TRANSFER,
            },
        },
        genesis: GenesisSchema {
            metadata: none!(),
            globals: none!(),
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            validator: None,
        },
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: tiny_bmap! {
                        global_type => Occurrences::NoneOrMore,
                    },
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::NoneOrMore,
                    },
                    assignments: none!(),
                    validator: Some(LibSite::with(0, lib.id())),
                },
                name: fname!("transfer"),
            },
        },
        default_assignment: Some(OS_ASSET),
    };
    let scripts = Confined::from_checked(bmap! {lib.id() => lib});
    let rules = SchemaDefinition::new(schema, types.libs(), scripts)
        .verify()
        .unwrap();
    let chain_net = ChainNet::BitcoinRegtest;
    let outpoint =
        Outpoint::from_str("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc:0")
            .unwrap();
    let seal = BuilderSeal::Revealed(GenesisSeal::rand_from(outpoint));
    let contract_consignment = ContractBuilder::with(strict_dumb!(), rules.clone(), chain_net)
        .add_fungible_state("assetOwner", seal, 14u64)
        .unwrap()
        .issue_contract_raw(42)
        .unwrap()
        .into_consignment();
    let contract_id = contract_consignment.contract_id();
    let opout = Opout::new(contract_consignment.genesis().id(), OS_ASSET, 0);

    let mut globals = rgb::GlobalState::default();
    for i in 0..global_items {
        globals
            .add_state(
                global_type,
                RevealedData::new(
                    Amount::from(u64::from(i))
                        .to_strict_serialized::<{ u16::MAX as usize }>()
                        .unwrap(),
                ),
            )
            .unwrap();
    }

    let transition = Transition {
        contract_id,
        transition_type: TS_TRANSFER,
        globals,
        inputs: NonEmptyOrdSet::with(opout).into(),
        ..strict_dumb!()
    };
    let opid = transition.id();
    let bundle = TransitionBundle {
        input_map: NonEmptyOrdMap::with_key_value(opout, opid),
        known_transitions: NonEmptyVec::with(KnownTransition::new(opid, transition)),
    };
    let mut psbt = Psbt::from_unsigned_tx(Transaction {
        version: Version::ONE,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0),
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: bitcoin::Amount::ZERO,
            script_pubkey: ScriptBuf::new_op_return([]),
        }],
    })
    .unwrap();
    psbt.inputs.get_mut(0).unwrap().witness_utxo = Some(TxOut {
        value: bitcoin::Amount::from_sat(1000),
        script_pubkey: ScriptBuf::new_p2a(),
    });
    let protocol_id = mpc::ProtocolId::from(contract_id);
    psbt.outputs.get_mut(0).unwrap().set_opret_host();
    psbt.outputs
        .get_mut(0)
        .unwrap()
        .set_mpc_message(protocol_id, mpc::Message::from(bundle.bundle_id()))
        .unwrap();
    let (commitment, proof) = psbt.outputs.get_mut(0).unwrap().mpc_commit().unwrap();
    psbt.outputs
        .get_mut(0)
        .unwrap()
        .opret_commit(commitment)
        .unwrap();
    psbt.set_opret_commitment(0);
    let tx = psbt.extract_tx().unwrap();
    let anchor = Anchor::new(
        proof.to_merkle_proof(protocol_id).unwrap(),
        DbcProof::Opret(OpretProof::strict_dumb()),
    );
    let wbundle = WitnessBundle::with(tx, anchor, bundle);
    let consignment = Consignment::<true> {
        transfer: true,
        bundles: Confined::from_checked(vec![wbundle]),
        genesis: contract_consignment.genesis,
        ..strict_dumb!()
    };
    (consignment, rules)
}

/// Validate, discarding the verdict: the child process reports a crash through its exit status
fn validate_quietly(consignment: Transfer, rules: SchemaRules) {
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let _ = consignment
        .clone()
        .validate(&rules, &resolver, &validation_config);
}

#[test]
#[ignore = "CnG writes a u16 global state count into RegA::A8"]
fn validate_consignment_global_state_count_overflow() {
    let Some(completed) = child_case("validate_consignment_global_state_count_overflow") else {
        let (consignment, rules) = scripted_transfer(CNG_A8.to_vec(), 256);
        validate_quietly(consignment, rules);
        return;
    };
    assert!(
        completed,
        "validating 256 global state items terminated the process"
    );
}

#[test]
fn validate_consignment_global_state_count_in_range() {
    let Some(completed) = child_case("validate_consignment_global_state_count_in_range") else {
        let (consignment, rules) = scripted_transfer(CNG_A8.to_vec(), 255);
        validate_quietly(consignment, rules);
        return;
    };
    assert!(
        completed,
        "validating 255 global state items terminated the process"
    );
}

#[test]
fn validate_consignment_out_of_bounds_global_state() {
    // AssetSpec: ticker of 255 '@' (declared RString<Alpha, AlphaNum, 1, 8>), empty name
    // (declared min 1), no details, precision centi
    let mut blob_hex = String::from("ff");
    for _ in 0..255 {
        blob_hex.push_str("40");
    }
    blob_hex.push_str("000002");

    let bytes = Vec::<u8>::from_hex(&blob_hex).unwrap();
    assert!(
        AssetSpec::from_strict_serialized::<0xFFFF>(Confined::try_from(bytes).unwrap()).is_err(),
        "blob must violate RGBContract.AssetSpec"
    );

    let scenario = Scenario::B;
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let old_genesis_opid = base_consignment.genesis.id();

    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let mut json_consignment: Value = serde_json::from_reader(file).unwrap();
    *json_consignment
        .get_mut("genesis")
        .unwrap()
        .get_mut("globals")
        .unwrap()
        .get_mut("2000") // GS_NOMINAL
        .unwrap()
        .get_mut(0)
        .unwrap() = Value::String(blob_hex);

    // rewriting genesis re-ids it and the contract, so realign every bundle
    let mut consignment = transfer_from_json_value(&json_consignment);
    let genesis_opid = consignment.genesis.id();
    let contract_id = consignment.contract_id();
    let mut bundles = consignment.bundles.release();
    update_transition_children(
        &mut bundles,
        map! {old_genesis_opid => genesis_opid},
        map! {},
        Some(contract_id),
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    consignment.terminals = empty!(); // terminals are now outdated

    let asset_schema_rules = AssetSchema::from(consignment.schema_id()).schema_rules();
    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let resolver = OfflineResolver {
        consignment: &consignment,
    };
    let res = consignment
        .clone()
        .validate(&asset_schema_rules, &resolver, &validation_config);
    let sem_id = StandardTypes::with(rgb_contract_stl()).get("RGBContract.AssetSpec");
    assert_eq!(
        res.unwrap_err(),
        ValidationError::InvalidConsignment(Failure::SchemaInvalidGlobalValue(
            genesis_opid,
            GS_NOMINAL,
            sem_id
        )),
        "global state outside the declared type accepted"
    );
}

/// `st.a a8[0]; ret;`: a validator body that succeeds
const PASSING_CODE: &[u8] = &[0x1e, 0x01, 0x07];

/// Build a transfer whose transition validator is `validator`, while it carries `shipped`
fn transfer_with_script(
    validator: aluvm::library::LibId,
    shipped: Lib,
) -> Result<(Transfer, SchemaRules), SchemaDefError> {
    let global_type = GlobalStateType::with(42);

    let types = StandardTypes::with(rgb_contract_stl());
    let schema = Schema {
        ffv: zero!(),
        name: tn!("ScriptedAsset"),
        meta_types: none!(),
        global_types: tiny_bmap! {
             global_type => GlobalDetails {
                global_state_schema: GlobalStateSchema::many(types.get("RGBContract.Amount")),
                name: fname!("someGlobal"),
            },
        },
        owned_types: tiny_bmap! {
            OS_ASSET => AssignmentDetails {
                owned_state_schema: OwnedStateSchema::Fungible(FungibleType::Unsigned64Bit),
                name: fname!("assetOwner"),
                default_transition: TS_TRANSFER,
            },
        },
        genesis: GenesisSchema {
            metadata: none!(),
            globals: none!(),
            assignments: tiny_bmap! {
                OS_ASSET => Occurrences::OnceOrMore,
            },
            validator: None,
        },
        transitions: tiny_bmap! {
            TS_TRANSFER => TransitionDetails {
                transition_schema: TransitionSchema {
                    metadata: none!(),
                    globals: tiny_bmap! {
                        global_type => Occurrences::NoneOrMore,
                    },
                    inputs: tiny_bmap! {
                        OS_ASSET => Occurrences::NoneOrMore,
                    },
                    assignments: none!(),
                    validator: Some(LibSite::with(0, validator)),
                },
                name: fname!("transfer"),
            },
        },
        default_assignment: Some(OS_ASSET),
    };
    let scripts = Confined::from_checked(bmap! {validator => shipped});
    let rules = SchemaDefinition::new(schema, types.libs(), scripts).verify()?;
    let chain_net = ChainNet::BitcoinRegtest;
    let outpoint =
        Outpoint::from_str("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc:0")
            .unwrap();
    let seal = BuilderSeal::Revealed(GenesisSeal::rand_from(outpoint));
    let contract_consignment = ContractBuilder::with(strict_dumb!(), rules.clone(), chain_net)
        .add_fungible_state("assetOwner", seal, 14u64)
        .unwrap()
        .issue_contract_raw(42)
        .unwrap()
        .into_consignment();
    let contract_id = contract_consignment.contract_id();
    let opout = Opout::new(contract_consignment.genesis().id(), OS_ASSET, 0);

    let transition = Transition {
        contract_id,
        transition_type: TS_TRANSFER,
        inputs: NonEmptyOrdSet::with(opout).into(),
        ..strict_dumb!()
    };
    let opid = transition.id();
    let bundle = TransitionBundle {
        input_map: NonEmptyOrdMap::with_key_value(opout, opid),
        known_transitions: NonEmptyVec::with(KnownTransition::new(opid, transition)),
    };
    let mut psbt = Psbt::from_unsigned_tx(Transaction {
        version: Version::ONE,
        lock_time: LockTime::ZERO,
        input: vec![TxIn {
            previous_output: outpoint,
            script_sig: ScriptBuf::new(),
            sequence: Sequence(0),
            witness: Witness::new(),
        }],
        output: vec![TxOut {
            value: bitcoin::Amount::ZERO,
            script_pubkey: ScriptBuf::new_op_return([]),
        }],
    })
    .unwrap();
    psbt.inputs.get_mut(0).unwrap().witness_utxo = Some(TxOut {
        value: bitcoin::Amount::from_sat(1000),
        script_pubkey: ScriptBuf::new_p2a(),
    });
    let protocol_id = mpc::ProtocolId::from(contract_id);
    psbt.outputs.get_mut(0).unwrap().set_opret_host();
    psbt.outputs
        .get_mut(0)
        .unwrap()
        .set_mpc_message(protocol_id, mpc::Message::from(bundle.bundle_id()))
        .unwrap();
    let (commitment, proof) = psbt.outputs.get_mut(0).unwrap().mpc_commit().unwrap();
    psbt.outputs
        .get_mut(0)
        .unwrap()
        .opret_commit(commitment)
        .unwrap();
    psbt.set_opret_commitment(0);
    let tx = psbt.extract_tx().unwrap();
    let anchor = Anchor::new(
        proof.to_merkle_proof(protocol_id).unwrap(),
        DbcProof::Opret(OpretProof::strict_dumb()),
    );
    let wbundle = WitnessBundle::with(tx, anchor, bundle);
    let consignment = Consignment::<true> {
        transfer: true,
        bundles: Confined::from_checked(vec![wbundle]),
        genesis: contract_consignment.genesis,
        ..strict_dumb!()
    };
    Ok((consignment, rules))
}

/// Build two libraries whose `LibId` preimages coincide
///
/// The second absorbs the first's code-length prefix and its first 254 code bytes into its own
/// ISAE segment, which the one-byte length prefix then reports as 259 - 256 = 3. The first 254
/// code bytes are therefore laid out as ISAE text: ascending, deduplicated, space separated names
/// of two to eight characters starting with a capital letter, so that `IsaSeg` renders them back
/// unchanged.
fn colliding_libraries(payload: &[u8]) -> (Lib, Lib) {
    // 0x4120 little-endian is " A": a separator followed by the start of a name
    const CODE_A_LEN: usize = 0x4120;
    const CODE_B_LEN: usize = CODE_A_LEN - 0x100;

    let mut isa_tail = String::from("ZAAAAAA "); // completes the name "AZAAAAAA"
    for c in b'A'..=b'Z' {
        isa_tail.push_str(&format!("B{}AAAAAA ", c as char));
    }
    isa_tail.push_str("CAAAAAAA ZZZ");
    assert_eq!(isa_tail.len(), 254);

    let mut code_a = isa_tail.clone().into_bytes();
    code_a.extend_from_slice(&(CODE_B_LEN as u16).to_le_bytes());
    code_a.extend_from_slice(payload);
    code_a.resize(CODE_A_LEN, 0x00);

    let isae_b = format!(
        "ALU{}{isa_tail}",
        String::from_utf8((CODE_A_LEN as u16).to_le_bytes().to_vec()).unwrap()
    );
    let lib_reviewed = Lib::with("ALU", code_a.clone(), vec![], none!()).unwrap();
    let lib_substituted = Lib::with(&isae_b, code_a[0x100..].to_vec(), vec![], none!()).unwrap();
    assert_eq!(
        lib_substituted.isae_segment(),
        isae_b,
        "ISAE segment must round trip"
    );
    (lib_reviewed, lib_substituted)
}

#[test]
#[ignore = "LibId truncates the ISAE segment length to one byte"]
fn lib_id_is_injective() {
    let (lib_reviewed, lib_substituted) = colliding_libraries(PASSING_CODE);

    assert_ne!(lib_reviewed.isae_segment(), lib_substituted.isae_segment());
    assert_ne!(lib_reviewed.code_segment(), lib_substituted.code_segment());
    assert_ne!(lib_reviewed.id(), lib_substituted.id());
}

#[test]
#[ignore = "LibId truncates the ISAE segment length to one byte"]
fn validate_consignment_substituted_script() {
    let (lib_reviewed, lib_substituted) = colliding_libraries(PASSING_CODE);
    // the library is either refused when the rules are verified, or when the consignment is
    let res =
        transfer_with_script(lib_reviewed.id(), lib_substituted).map(|(consignment, rules)| {
            let validation_config = ValidationConfig {
                chain_net: ChainNet::BitcoinRegtest,
                ..Default::default()
            };
            let resolver = OfflineResolver {
                consignment: &consignment,
            };
            consignment
                .clone()
                .validate(&rules, &resolver, &validation_config)
        });
    assert!(
        !matches!(res, Ok(Ok(_))),
        "a library the schema does not commit to was executed"
    );
}

/// `st.a a16[0]; ret;` and `st.a a8[0]; ret;`, as raw bytecode
const ST_A16: &[u8] = &[0x1e, 0x05, 0x07];
const ST_A8: &[u8] = &[0x1e, 0x01, 0x07];

#[test]
#[ignore = "cmp.st merges the 8-bit status flag into a register of any width"]
fn validate_consignment_script_status_merge() {
    let Some(completed) = child_case("validate_consignment_script_status_merge") else {
        // rgb-consensus preloads a16[0] with the transition type, so `st.a a16[0]` alone
        // reaches int_add with mismatched operand layouts
        let (consignment, rules) = scripted_transfer(ST_A16.to_vec(), 0);
        validate_quietly(consignment, rules);
        return;
    };
    assert!(
        completed,
        "a validator library merging into a16 terminated the process"
    );
}

#[test]
fn validate_consignment_script_status_merge_in_range() {
    let Some(completed) = child_case("validate_consignment_script_status_merge_in_range") else {
        let (consignment, rules) = scripted_transfer(ST_A8.to_vec(), 0);
        validate_quietly(consignment, rules);
        return;
    };
    assert!(
        completed,
        "a validator library merging into a8 terminated the process"
    );
}

/// Build a UDA contract whose `tokens` global state holds two attachments, under keys 1 and 2
fn uda_contract_with_two_attachments() -> (Consignment<false>, GlobalStateType, Vec<u8>, usize) {
    let att_a = attachment_from_fpath(MEDIA_FPATH);
    let mut att_b = att_a.clone();
    // same media type, so both encode to the same length and only the digest differs
    att_b.digest = sha256::Hash::hash(b"rgb-tests: second attachment")
        .to_byte_array()
        .into();
    let att_len = att_b.to_strict_serialized::<U16>().unwrap().len();

    let token_data = TokenData {
        index: TokenIndex::from(UDA_FIXED_INDEX),
        attachments: Confined::try_from(bmap! { 1u8 => att_a, 2u8 => att_b }).unwrap(),
        ..Default::default()
    };
    let canonical = token_data.to_strict_serialized::<U16>().unwrap().release();

    let builder = ContractBuilder::with(
        strict_dumb!(),
        UniqueDigitalAsset::schema_rules(),
        ChainNet::BitcoinRegtest,
    );
    let gs_tokens = builder.global_type("tokens");
    let outpoint =
        Outpoint::from_str("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc:0")
            .unwrap();
    let seal = BuilderSeal::Revealed(GenesisSeal::rand_from(outpoint));
    let contract = builder
        .add_global_state(
            "spec",
            AssetSpec::with("TKN", "Token", Precision::try_from(0).unwrap(), None).unwrap(),
        )
        .unwrap()
        .add_global_state(
            "terms",
            ContractTerms {
                text: RicardianContract::from_str("terms").unwrap(),
                media: None,
            },
        )
        .unwrap()
        .add_global_state("tokens", token_data)
        .unwrap()
        .add_data(
            "assetOwner",
            seal,
            Allocation::with(UDA_FIXED_INDEX, OwnedFraction::from(1)),
        )
        .unwrap()
        .issue_contract_raw(0)
        .unwrap()
        .into_consignment();

    (contract, gs_tokens, canonical, att_len)
}

#[test]
fn validate_consignment_noncanonical_global_state_map() {
    use rgbstd::GlobalValues;

    let (contract, gs_tokens, canonical, att_len) = uda_contract_with_two_attachments();

    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };

    // control: the canonically encoded contract is valid
    let resolver = OfflineResolver {
        consignment: &contract,
    };
    contract
        .clone()
        .validate(
            &UniqueDigitalAsset::schema_rules(),
            &resolver,
            &validation_config,
        )
        .unwrap();

    // the second key sits one attachment plus its own key byte from the end, ahead of `reserves`
    let pos = canonical.len() - att_len - 2;
    assert_eq!(canonical[pos], 2, "unexpected TokenData encoding layout");

    for (case, patched_key) in [("duplicate key", 1u8), ("descending keys", 0u8)] {
        let mut blob = canonical.clone();
        blob[pos] = patched_key;
        let blob = SmallBlob::from_checked(blob);

        assert!(
            TokenData::from_strict_serialized::<U16>(blob.clone()).is_err(),
            "{case}: the canonical decoder unexpectedly accepted the blob"
        );

        let mut malicious = contract.clone();
        let _ = malicious
            .genesis
            .globals
            .insert(gs_tokens, GlobalValues::with(RevealedData::new(blob)))
            .unwrap();

        let resolver = OfflineResolver {
            consignment: &malicious,
        };
        let res = malicious.clone().validate(
            &UniqueDigitalAsset::schema_rules(),
            &resolver,
            &validation_config,
        );
        assert!(
            res.is_err(),
            "{case}: validator accepted global state that the canonical strict-encoding decoder \
             rejects"
        );
    }
}

#[test]
#[ignore = "Variant equality matches on tag or name"]
fn validate_consignment_substituted_enum_type() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let mut json_consignment: Value = serde_json::from_reader(file).unwrap();

    // permute two Precision variant names, leaving the tag set intact
    let types = json_consignment
        .get_mut("types")
        .unwrap()
        .as_object_mut()
        .unwrap();
    let mut precision_sem_id = None;
    for (sem_id, ty) in types.iter_mut() {
        let Some(variants) = ty.get_mut("Enum").and_then(Value::as_array_mut) else {
            continue;
        };
        if !variants.iter().any(|v| v == "centi:2") || !variants.iter().any(|v| v == "atto:18") {
            continue;
        }
        for variant in variants.iter_mut() {
            if variant == "centi:2" {
                *variant = json!("atto:2");
            } else if variant == "atto:18" {
                *variant = json!("centi:18");
            }
        }
        precision_sem_id = Some(sem_id.clone());
        break;
    }
    let precision_sem_id = precision_sem_id.expect("Precision enum");
    let precision_sem_id: SemId = serde_json::from_value(Value::String(precision_sem_id)).unwrap();

    let consignment = transfer_from_json_value(&json_consignment);
    let substituted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();

    // 0x02 is the trailing precision byte of the GS_NOMINAL blob the consignment ships
    assert_ne!(
        trusted_typesystem
            .strict_deserialize_type(precision_sem_id, &[2u8])
            .unwrap()
            .unbox(),
        substituted_typesystem
            .strict_deserialize_type(precision_sem_id, &[2u8])
            .unwrap()
            .unbox(),
    );

    let validation_config = ValidationConfig {
        chain_net: ChainNet::BitcoinRegtest,
        ..Default::default()
    };
    let rules = AssetSchema::from(consignment.schema_id()).schema_rules();
    let res = consignment.validate(&rules, &resolver, &validation_config);
    assert!(
        res.is_err(),
        "substituted type definition accepted at a trusted sem id"
    );
}

/// Pad `scripts` with maxed-out libraries until the consignment exceeds the ASCII armor bound
fn oversized_consignment() -> Transfer {
    const PAD_LIBS: u16 = 140;

    let consignment = get_consignment_from_json("consignment_B");
    let base_lib = AssetSchema::from(consignment.schema_id())
        .scripts()
        .release()
        .pop_first()
        .unwrap()
        .1;
    let mut libs = BTreeSet::new();
    for i in 0..PAD_LIBS {
        let mut lib = base_lib.clone();
        let mut data = vec![0u8; u16::MAX as usize];
        data[..2].copy_from_slice(&i.to_le_bytes()); // keep every library distinct
        lib.code = SmallBlob::from_checked(vec![0u8; u16::MAX as usize]);
        lib.data = SmallBlob::from_checked(data);
        libs.insert(lib);
    }
    consignment
}

#[test]
#[ignore = "armor encode cap is below the container decode cap, ~40s"]
fn oversized_consignment_armoring() {
    let Some(completed) = child_case("oversized_consignment_armoring") else {
        let consignment = oversized_consignment();
        let size = consignment
            .to_strict_serialized::<{ usize::MAX }>()
            .unwrap()
            .release()
            .len();
        assert!(size > u24::MAX.to_usize(), "padding insufficient: {size}");

        // refusing the container on save, refusing it on load and rendering it are all
        // acceptable; only terminating the process is not
        let mut container = Vec::<u8>::new();
        if consignment.save(&mut container).is_ok()
            && let Ok(transfer) = Transfer::load(&container[..])
        {
            assert!(!transfer.to_string().is_empty());
        }
        return;
    };
    assert!(
        completed,
        "armoring an oversized consignment terminated the process"
    );
}

#[test]
fn normal_consignment_armoring() {
    let consignment = get_consignment_from_json("consignment_B");
    assert!(
        consignment
            .to_string()
            .starts_with("-----BEGIN RGB CONSIGNMENT-----")
    );
    let mut container = Vec::<u8>::new();
    consignment.save(&mut container).unwrap();
    assert_eq!(Transfer::load(&container[..]).unwrap(), consignment);
}

#[cfg(unix)]
#[test]
fn bundles_size_alloc_bomb() {
    const F1_CHILD_ENV: &str = "RGB_TESTS_F1_CONSIGNMENT_BOMB";

    /// Caps the address space of the current process to model a memory-limited reader.
    fn cap_address_space(cap_mib: u64) {
        unsafe {
            let mut lim = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            libc::getrlimit(libc::RLIMIT_AS, &mut lim);
            let cap = cap_mib << 20;
            lim.rlim_cur = cap.min(lim.rlim_max);
            libc::setrlimit(libc::RLIMIT_AS, &lim);
        }
    }

    /// Takes a valid consignment and inflates its `bundles` length prefix to 0xFFFF_FFFF.
    fn forged_consignment_bytes() -> Vec<u8> {
        let transfer = get_consignment_from_json("consignment_A");
        let mut bytes = transfer
            .to_strict_serialized::<{ usize::MAX }>()
            .unwrap()
            .release();

        let first_bundle = transfer
            .bundles
            .iter()
            .next()
            .expect("consignment_A has at least one witness bundle");
        let bundle_bytes: Vec<u8> = first_bundle
            .strict_encode(StrictWriter::in_memory::<{ usize::MAX }>())
            .unwrap()
            .unbox()
            .unconfine();
        let bundle_start = bytes
            .windows(bundle_bytes.len())
            .position(|w| w == bundle_bytes)
            .expect("first bundle encoding must appear in the consignment");
        let prefix = bundle_start - 4;

        bytes[prefix..prefix + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        bytes
    }

    if std::env::var(F1_CHILD_ENV).is_ok() {
        // this code is executed in a subprocess
        cap_address_space(8192); // 8 GiB
        let bytes = forged_consignment_bytes();
        let confined = Confined::try_from(bytes).unwrap();
        let res = Transfer::from_strict_serialized::<{ usize::MAX }>(confined);
        assert!(res.is_err());
        return;
    }

    // run from_strict_serialized in a dedicated process
    let exe = std::env::current_exe().unwrap();
    let out = std::process::Command::new(exe)
        .args([
            "bundles_size_alloc_bomb",
            "--exact",
            "--nocapture",
            "--include-ignored",
        ])
        .env(F1_CHILD_ENV, "1")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "child process failed ({}):\n{}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
}
