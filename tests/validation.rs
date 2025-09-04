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
        Ok(())
    }
}
impl MockResolver {
    pub fn with_new_transaction(&self, witness: Tx) -> Self {
        let mut resolver = self.clone();
        let witness_id = witness.txid();
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

    fn resolver(&self) -> MockResolver {
        let mut txs = map![];
        for entry in std::fs::read_dir(self.txs_folder()).unwrap() {
            let file = std::fs::File::open(entry.unwrap().path()).unwrap();
            let tx: Tx = serde_json::from_reader(file).unwrap();
            txs.insert(tx.txid(), tx);
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

fn update_anchor(witness_bundle: &mut WitnessBundle, contract_id: Option<ContractId>) {
    let mut witness_psbt = Psbt::from_tx(witness_bundle.pub_witness.tx().unwrap().clone());
    let idx = witness_psbt
        .outputs()
        .find(|o| o.script.is_op_return())
        .unwrap()
        .index();
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
    witness_psbt.output_mut(idx).unwrap().script = ScriptPubkey::op_return(&[]);
    witness_psbt
        .output_mut(idx)
        .unwrap()
        .set_opret_host()
        .unwrap();
    witness_psbt
        .output_mut(idx)
        .unwrap()
        .set_mpc_message(protocol_id, message)
        .unwrap();
    let (commitment, proof) = witness_psbt.output_mut(idx).unwrap().mpc_commit().unwrap();
    witness_psbt
        .output_mut(idx)
        .unwrap()
        .opret_commit(commitment)
        .unwrap();
    let witness: Tx = witness_psbt.to_unsigned_tx().into();

    let mut anchor = witness_bundle.anchor.clone();
    anchor.mpc_proof = proof.to_merkle_proof(protocol_id).unwrap();
    witness_bundle.pub_witness = PubWitness::Tx(witness.clone());
    witness_bundle.anchor = anchor;
}

/// Update children bundles to keep consistency with some modified transitions/transactions
fn update_transition_children(
    witness_bundles: &mut Vec<WitnessBundle>,
    changed_opids: HashMap<OpId, OpId>,
    changed_txids: HashMap<Txid, Txid>,
) {
    let mut changed_opids = changed_opids;
    let mut changed_txids = changed_txids;
    let mut something_changed = false;
    for wbundle in witness_bundles.iter_mut() {
        let old_txid = wbundle.witness_id();
        if !changed_txids.contains_key(&old_txid)
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
                    let new_t = Transition {
                        inputs: NonEmptyOrdSet::from_iter_checked(
                            transition.inputs.iter().map(|i| *opout_map.get(i).unwrap()),
                        )
                        .into(),
                        ..transition.clone()
                    };
                    let new_opid = new_t.id();
                    if *opid != new_opid {
                        changed_opids.insert(*opid, new_opid);
                        something_changed = true;
                    }
                    KnownTransition {
                        opid: new_opid,
                        transition: new_t,
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
        let mut witness = wbundle.pub_witness.tx().unwrap().clone();
        witness.inputs.iter_mut().for_each(|i| {
            let txid = &i.prev_output.txid;
            i.prev_output.txid = *changed_txids.get(txid).unwrap_or(txid);
        });
        wbundle.pub_witness = PubWitness::Tx(witness);
        update_anchor(wbundle, None);
        if old_txid != wbundle.witness_id() {
            changed_txids.insert(old_txid, wbundle.witness_id());
            something_changed = true;
        }
    }
    if something_changed {
        update_transition_children(witness_bundles, changed_opids, changed_txids)
    }
}

/// Remove bundles that depend on some opids (optionally only ones spending a given allocation type)
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
            .filter(|o| assignment_type.map_or(true, |t| t == o.ty))
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

    let transfer_type = match scenario {
        Scenario::A => TransferType::Blinded,
        Scenario::B => TransferType::Witness,
        Scenario::C => TransferType::Witness,
    };

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issued_supply_1 = 999;
    let issued_supply_2 = 666;

    let sats = 9000;

    let utxo = wlt_1.get_utxo(None);
    let contract_id_1 = wlt_1.issue_nia(issued_supply_1, Some(&utxo));
    let contract_id_2 = match scenario {
        Scenario::C => wlt_1.issue_ifa(issued_supply_2, Some(&utxo), vec![utxo], vec![(utxo, 100)]),
        _ => wlt_1.issue_nia(issued_supply_2, Some(&utxo)),
    };

    let mut txes = vec![];

    let (_consignment, tx) = wlt_1.send(&mut wlt_2, transfer_type, contract_id_1, 66, sats, None);
    txes.push(tx);

    if scenario == Scenario::C {
        // wlt_1 can't replace assets since its inflation right would also be included

        // get inflation right out of the way
        let schema_id_2 = wlt_1.schema_id(contract_id_2);
        let mut invoice =
            wlt_1.invoice(contract_id_2, schema_id_2, 100, InvoiceType::Blinded(None));
        invoice.assignment_name = Some(FieldName::from_str("inflationAllowance").unwrap());
        let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
        wlt_1.mine_tx(&tx.txid(), false);
        wlt_1.accept_transfer(consignment, None);
        wlt_1.sync();
        txes.push(tx);

        // send assets to be replaced to wlt_2
        let amt = 666;
        let (_, tx) = wlt_1.send(
            &mut wlt_2,
            transfer_type,
            contract_id_2,
            amt,
            sats - 1000,
            None,
        );
        let change_utxo = Outpoint::new(tx.txid(), Vin::from_u32(2));
        txes.push(tx);

        // replace assets
        let tx = wlt_2.replace_ifa(&mut wlt_1, change_utxo, contract_id_2);
        txes.push(tx);
        // send them back to carry on with test
        let (_, tx) = wlt_2.send(
            &mut wlt_1,
            transfer_type,
            contract_id_2,
            amt,
            sats - 2000,
            None,
        );
        txes.push(tx);
    }

    let (tx, next_amt) = if scenario == Scenario::C {
        // inflate asset using right that was moved automatically
        let contract = wlt_1.contract_wrapper::<InflatableFungibleAsset>(contract_id_2);
        let inflation_allocations = contract
            .inflation_allocations(AllocationFilter::Wallet.filter_for(&wlt_1))
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
        let wlt_1_utxos = wlt_1
            .utxos()
            .iter()
            .map(|wu| wu.outpoint)
            .collect::<Vec<_>>();
        wlt_1.burn_ifa(contract_id_2, wlt_1_utxos)
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

    (consignment, txes)
}

struct OfflineResolver<'cons, const TRANSFER: bool> {
    consignment: &'cons IndexedConsignment<'cons, TRANSFER>,
}
impl<const TRANSFER: bool> ResolveWitness for OfflineResolver<'_, TRANSFER> {
    fn resolve_witness(&self, witness_id: Txid) -> Result<WitnessStatus, WitnessResolverError> {
        self.consignment
            .pub_witness(witness_id)
            .and_then(|p| p.tx().cloned())
            .map_or_else(
                || Ok(WitnessStatus::Unresolved),
                |tx| Ok(WitnessStatus::Resolved(tx, WitnessOrd::Tentative)),
            )
    }
    fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
        Ok(())
    }
}

// run once to generate tests/fixtures/consignemnt_<scenario>.json
// for example:
// SCENARIO=B cargo test --test validation validate_consignment_generate -- --ignored --show-output
//
// then copy the generated consignemnt file to tests/fixtures/attack_<n>.json
// manually change tests/fixtures/attack_<n>.json files to simulate attacks
#[cfg(not(feature = "altered"))]
#[test]
#[ignore = "one-shot"]
fn validate_consignment_generate() {
    let scenario = match std::env::var("SCENARIO") {
        Ok(val) if val.to_uppercase() == Scenario::A.to_string() => Scenario::A,
        Ok(val) if val.to_uppercase() == Scenario::B.to_string() => Scenario::B,
        Ok(val) if val.to_uppercase() == Scenario::C.to_string() => Scenario::C,
        Err(VarError::NotPresent) => Scenario::A,
        _ => panic!("invalid scenario"),
    };
    let (consignment, txes) = get_consignment(scenario);
    println!();
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let json = serde_json::to_string_pretty(&consignment).unwrap();
    std::fs::write(&cons_path, json).unwrap();
    println!("written consignment in: {cons_path}");
    let _ = std::fs::remove_dir_all(scenario.txs_folder());
    std::fs::create_dir_all(scenario.txs_folder()).unwrap();
    for tx in txes {
        let txid = tx.txid().to_string();
        let json = serde_json::to_string_pretty(&tx).unwrap();
        let json_path = format!("{}/{txid}.json", scenario.txs_folder());
        std::fs::write(&json_path, json).unwrap();
        println!("written tx: {txid}");
    }
}

fn get_consignment_from_json(fname: &str) -> Transfer {
    let cons_path = format!("tests/fixtures/{fname}.json");
    println!("loading {cons_path}");
    let file = std::fs::File::open(cons_path).unwrap();
    let consignment: Transfer = serde_json::from_reader(file).unwrap();
    consignment
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_success() {
    for scenario in Scenario::iter() {
        let resolver = scenario.resolver();
        let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
        let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
        let res = consignment
            .validate(
                &resolver,
                ChainNet::BitcoinRegtest,
                None,
                trusted_typesystem,
            )
            .unwrap();
        let validation_status = res.validation_status();
        dbg!(&validation_status);
        assert!(validation_status.warnings.is_empty());
        assert!(validation_status.info.is_empty());
        let validity = validation_status.validity();
        assert_eq!(validity, Validity::Valid);
    }
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_chain_fail() {
    let resolver = Scenario::A.resolver();

    // genesis chainNet: change from bitcoinRegtest to liquidTestnet
    let consignment = get_consignment_from_json("attack_chain");
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ContractChainNetMismatch(
            ChainNet::BitcoinRegtest
        ))
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_genesis_fail() {
    let resolver = Scenario::B.resolver();

    // schema ID: change genesis[schemaId] with CFA schema ID
    let consignment = get_consignment_from_json("attack_genesis_schema_id");
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::OperationAbsent(_))
    ));

    // genesis chainNet: change from bitcoinRegtest to bitcoinMainnet
    let consignment = get_consignment_from_json("attack_genesis_testnet");
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ContractChainNetMismatch(
            ChainNet::BitcoinRegtest
        ))
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_bundles_fail() {
    let resolver = Scenario::A.resolver();

    // bundles first in time pubWitness inputs[0] sequence: change from 0 to 1
    let consignment = get_consignment_from_json("attack_bundles_pubWitness_data_input_sequence");
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::SealNoPubWitness(_, _))
    ));
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_resolver_error() {
    let scenario = Scenario::A;
    let base_resolver = scenario.resolver();
    let consignment = get_consignment_from_json("attack_resolver_error");
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let txid =
        Txid::from_str("b411d8dd37353d243a527739fdc39cca22dbfe4fe92517ce16a33563803c5ad2").unwrap();

    struct ConsignmentResolver<'a, 'cons, const TRANSFER: bool> {
        consignment: &'cons IndexedConsignment<'cons, TRANSFER>,
        fallback: &'a MockResolver,
    }
    impl<const TRANSFER: bool> ResolveWitness for ConsignmentResolver<'_, '_, TRANSFER> {
        fn resolve_witness(&self, witness_id: Txid) -> Result<WitnessStatus, WitnessResolverError> {
            self.consignment
                .pub_witness(witness_id)
                .and_then(|p| p.tx().cloned())
                .map_or_else(
                    || self.fallback.resolve_witness(witness_id),
                    |tx| Ok(WitnessStatus::Resolved(tx, WitnessOrd::Tentative)),
                )
        }
        fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
            Ok(())
        }
    }

    // resolve_pub_witness: ResolverIssue
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::ResolverIssue(Some(txid), s!("connection error"));
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(resolver_error.clone());
    let consignment_resolver = ConsignmentResolver {
        consignment: &IndexedConsignment::new(&consignment),
        fallback: &resolver,
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    let consignment_resolver = ConsignmentResolver {
        consignment: &IndexedConsignment::new(&consignment),
        fallback: &resolver,
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // resolve_pub_witness: InvalidResolverData
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::InvalidResolverData;
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(resolver_error.clone());
    let consignment_resolver = ConsignmentResolver {
        consignment: &IndexedConsignment::new(&consignment),
        fallback: &resolver,
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));

    // resolve_pub_witness: WrongChainNet
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::WrongChainNet;
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(resolver_error.clone());
    let consignment_resolver = ConsignmentResolver {
        consignment: &IndexedConsignment::new(&consignment),
        fallback: &resolver,
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(res, ValidationError::ResolverError(resolver_error));
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_unknown_tx() {
    let scenario = Scenario::A;
    let base_resolver = scenario.resolver();
    let consignment = get_consignment_from_json("attack_resolver_error");
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let txid =
        Txid::from_str("b411d8dd37353d243a527739fdc39cca22dbfe4fe92517ce16a33563803c5ad2").unwrap();
    let wbundle = consignment
        .bundles
        .iter()
        .find(|wb| wb.witness_id() == txid)
        .unwrap();
    let bundle_id = wbundle.bundle.bundle_id();

    let mut resolver = base_resolver.clone();
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Success(WitnessStatus::Unresolved);
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SealNoPubWitness(bundle_id, txid))
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_schema_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let trusted_typesystem = AssetSchema::from(base_consignment.schema_id()).types();
    let transition_type = base_consignment.schema.transitions.keys().last().unwrap();

    // SchemaOpMetaTypeUnknown: schema transition has unknown metatype
    let mut consignment = base_consignment.clone();
    let meta_type = MetaType::with(42);
    consignment
        .schema
        .transitions
        .get_mut(transition_type)
        .unwrap()
        .transition_schema
        .metadata = TinyOrdSet::from_checked(bset![meta_type]);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaOpMetaTypeUnknown(
            OpFullType::StateTransition(*transition_type),
            meta_type
        ))
    );

    // SchemaOpEmptyInputs: schema transition has no inputs
    let mut consignment = base_consignment.clone();
    consignment
        .schema
        .transitions
        .get_mut(transition_type)
        .unwrap()
        .transition_schema
        .inputs = TinyOrdMap::new();
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaOpEmptyInputs(
            OpFullType::StateTransition(*transition_type)
        ))
    );

    // SchemaOpGlobalTypeUnknown: schema transition has unknown global type
    let mut consignment = base_consignment.clone();
    let global_state_type = GlobalStateType::with(42);
    consignment
        .schema
        .transitions
        .get_mut(transition_type)
        .unwrap()
        .transition_schema
        .globals = TinyOrdMap::from_checked(bmap! {
        global_state_type => Occurrences::Once
    });
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaOpGlobalTypeUnknown(
            OpFullType::StateTransition(*transition_type),
            global_state_type
        ))
    );

    // SchemaOpAssignmentTypeUnknown: schema transition has unknown assignment type
    let mut consignment = base_consignment.clone();
    let assignment_type = AssignmentType::with(42);
    consignment
        .schema
        .transitions
        .get_mut(transition_type)
        .unwrap()
        .transition_schema
        .assignments = TinyOrdMap::from_checked(bmap! {
        assignment_type => Occurrences::Once
    });
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaOpAssignmentTypeUnknown(
            OpFullType::StateTransition(*transition_type),
            assignment_type
        ))
    );

    // SchemaMetaSemIdUnknown: schema meta type has unknown sem id
    let mut consignment = base_consignment.clone();
    let meta_type = MetaType::with(42);
    let sem_id = SemId::from([42u8; 32]);
    consignment.schema.meta_types = TinyOrdMap::from_checked(bmap! {meta_type => MetaDetails {
        sem_id,
        name: fname!("foo")
    }});
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaMetaSemIdUnknown(meta_type, sem_id))
    );

    // SchemaGlobalSemIdUnknown: schema global type has unknown sem id
    let mut consignment = base_consignment.clone();
    let mut global_types = consignment.schema.global_types.release();
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
    consignment.schema.global_types = TinyOrdMap::from_checked(global_types);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaGlobalSemIdUnknown(
            global_state_type,
            sem_id
        ))
    );

    // SchemaOwnedSemIdUnknown: schema owned type has unknown sem id
    let mut consignment = base_consignment.clone();
    let mut owned_types = consignment.schema.owned_types.release();
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
    consignment.schema.owned_types = TinyOrdMap::from_checked(owned_types);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaOwnedSemIdUnknown(
            assignment_type,
            sem_id
        ))
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_commitments_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let trusted_typesystem = AssetSchema::from(base_consignment.schema_id()).types();

    // CyclicGraph: duplicate the same transition within a bundle to create a cycle
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
    witness_bundle
        .bundle
        .known_transitions
        .push(existing_transition)
        .unwrap();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::CyclicGraph(opid))
    );

    // DoubleSpend: add different transition that spends the same opouts
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

    new_bundle
        .bundle
        .known_transitions
        .push(KnownTransition::new(transition.id(), transition))
        .unwrap();
    let bundle_id = new_bundle.bundle().bundle_id();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ExtraKnownTransition(bundle_id))
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
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::OperationAbsent(missing_opid))
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
    let mut fst_input = transition_inputs.pop_first().unwrap();
    let state_type = AssignmentType::with(42);
    fst_input.ty = state_type;
    transition_inputs.insert(fst_input);
    transition.inputs = NonEmptyOrdSet::from_checked(transition_inputs).into();
    replace_transition_in_bundle(witness_bundle, old_opid, transition.clone());
    let opid = transition.id();
    let mut input_map = witness_bundle.bundle.input_map.clone().release();
    let prev_id = fst_input.op;
    input_map.insert(fst_input, transition.id());
    witness_bundle.bundle.input_map = NonEmptyOrdMap::from_checked(input_map);
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoPrevState {
            opid,
            prev_id,
            state_type
        })
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
    let opid = transition.id();
    let mut input_map = witness_bundle.bundle.input_map.clone().release();
    input_map.insert(fst_input, transition.id());
    witness_bundle.bundle.input_map = NonEmptyOrdMap::from_checked(input_map);
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::NoPrevOut(opid, fst_input))
    );

    // ConfidentialSeal: one of the transitions includes blinded assignments
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
            rgb::Assign::ConfidentialSeal {
                seal: seal.to_secret_seal(),
                state,
            }
        })
        .collect::<Vec<_>>();
    let assignments =
        TypedAssigns::Fungible(AssignVec::with(NonEmptyVec::from_checked(assignments)));
    transition
        .assignments
        .insert(AssignmentType::ASSET, assignments)
        .unwrap();
    new_bundle.bundle.known_transitions = transitions;
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ConfidentialSeal(opout))
    );

    // ExtraKnownTransition: replace known_transition referenced in input map
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
    new_bundle.bundle.known_transitions =
        NonEmptyVec::from_checked(vec![KnownTransition::new(transition.id(), transition)]);
    let bundle_id = new_bundle.bundle().bundle_id();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ExtraKnownTransition(bundle_id))
    );

    // MissingInputMapTransition
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
    let fst_input = *transition.inputs.as_unconfined().first().unwrap();
    transition
        .inputs
        .push(Opout { no: 9, ..fst_input })
        .unwrap();
    replace_transition_in_bundle(witness_bundle, old_opid, transition.clone());
    let bundle_id = witness_bundle.bundle.bundle_id();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::MissingInputMapTransition(
            bundle_id,
            fst_input.op
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
    let consignment: Transfer =
        serde_json::from_str(&serde_json::to_string(&consignment).unwrap()).unwrap();
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    let mut witness_tx = new_bundle.pub_witness.tx().unwrap().clone();
    let mut outputs = witness_tx.outputs.release().clone();
    outputs.retain(|o| !o.script_pubkey.is_op_return());
    witness_tx.outputs = LargeVec::from_checked(outputs);
    let witness_id = witness_tx.txid();
    new_bundle.pub_witness = PubWitness::Tx(witness_tx);
    //update_witness_and_anchor(witness_bundle, contract_id);
    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
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
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::InvalidProofType(
            witness_id,
            bp::dbc::Method::TapretFirst
        ))
    );

    // SealsInvalid
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let new_bundle = bundles.last_mut().unwrap();
    let bundle_id = new_bundle.bundle.bundle_id();
    let mut witness_tx = new_bundle.pub_witness.tx().unwrap().clone();
    let mut inputs = witness_tx.inputs.release().clone();
    inputs.pop();
    witness_tx.inputs = LargeVec::from_checked(inputs);
    let witness_id = witness_tx.txid();
    new_bundle.pub_witness = PubWitness::Tx(witness_tx);
    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::SealsInvalid(bid, wid, _)) if bid == bundle_id && wid == witness_id
    ));

    // UnorderedTransition: put last bundle as first
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let opid = bundles[0].bundle.known_transitions[0].opid;
    bundles.swap(0, 1);
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::UnorderedTransition(opid))
    );

    // DBC-related error cases
    //  EmbedVerifyError::CommitmentMismatch
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles.last_mut().unwrap();
    let bundle_id = wbundle.bundle().bundle_id();
    let mut witness_tx = wbundle.pub_witness.tx().unwrap().clone();
    let mut outputs = witness_tx.outputs.release().clone();
    let output = outputs
        .iter_mut()
        .find(|o| o.script_pubkey.is_op_return())
        .unwrap();
    let mut script_pubkey = output.script_pubkey.as_unconfined().clone();
    script_pubkey.swap(1, 2);
    output.script_pubkey = ScriptPubkey::from_unsafe(script_pubkey);
    witness_tx.outputs = LargeVec::from_checked(outputs);
    let witness_id = witness_tx.txid();
    wbundle.pub_witness = PubWitness::Tx(witness_tx);

    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
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
    let mut witness_tx = wbundle.pub_witness.tx().unwrap().clone();
    let mut outputs = witness_tx.outputs.release().clone();
    outputs
        .iter_mut()
        .find(|o| o.script_pubkey.is_op_return())
        .unwrap()
        .script_pubkey
        .push_slice(&[42]);
    witness_tx.outputs = LargeVec::from_checked(outputs);
    let witness_id = witness_tx.txid();
    wbundle.pub_witness = PubWitness::Tx(witness_tx);

    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
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
    let mut witness_tx = wbundle.pub_witness.tx().unwrap().clone();
    let mut outputs = witness_tx.outputs.release().clone();
    outputs
        .iter_mut()
        .find(|o| o.script_pubkey.is_op_return())
        .unwrap()
        .script_pubkey
        .push_slice(&[42]);
    witness_tx.outputs = LargeVec::from_checked(outputs);
    let witness_id = witness_tx.txid();
    wbundle.pub_witness = PubWitness::Tx(witness_tx);

    consignment.bundles = LargeVec::from_checked(bundles);
    let consignment_resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &consignment_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
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

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_logic_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let trusted_typesystem = AssetSchema::from(base_consignment.schema_id()).types();

    // SchemaMismatch: replace consignment.schema with a compatible schema with different id
    let mut consignment = base_consignment.clone();
    let schema_id = consignment.schema_id();
    let mut alt_schema = NonInflatableAsset::schema();
    alt_schema.name = tn!("NonInflatableAsset2");
    let alt_schema_id = alt_schema.schema_id();
    consignment.schema = alt_schema;
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
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
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
                NonEmptyVec::with(Assign::ConfidentialSeal {
                    seal: SecretSeal::strict_dumb(),
                    state: VoidState::strict_dumb(),
                })
                .into(),
            ),
        )
        .unwrap();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
        .map(|a| a.as_revealed_state().as_u64())
        .sum::<u64>();
    transition
        .assignments
        .insert(
            assignment_type,
            TypedAssigns::Fungible(
                NonEmptyVec::with(Assign::ConfidentialSeal {
                    seal: SecretSeal::strict_dumb(),
                    state: RevealedValue::new(output_sum + 1),
                })
                .into(),
            ),
        )
        .unwrap();
    let transition_id = transition.id();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
            .push(Assign::ConfidentialSeal {
                seal: SecretSeal::strict_dumb(),
                state: RevealedValue::new(Amount::ZERO),
            })
            .unwrap();
    } else {
        panic!("unexpected asssignment type")
    };
    let opid = transition.id();
    assert_ne!(opid, old_opid);
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    let witness_tx = consignment
        .bundles
        .last()
        .unwrap()
        .pub_witness
        .tx()
        .unwrap()
        .clone();
    let witness_id = witness_tx.txid();
    // transaction is added as tentative
    let alt_resolver = resolver.with_new_transaction(witness_tx);
    let res = consignment.validate(
        &alt_resolver,
        ChainNet::BitcoinRegtest,
        Some(NonZeroU32::new(1000).unwrap()),
        trusted_typesystem.clone(),
    );
    let warnings = res.unwrap().validation_status().warnings.clone();
    assert_eq!(warnings.len(), 1);
    assert_eq!(
        warnings[0],
        Warning::UnsafeHistory(map! {0 => set![ witness_id ]})
    );

    // the following test cases require a more complex schema (IFA)
    let scenario = Scenario::C;
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let trusted_typesystem = AssetSchema::from(base_consignment.schema_id()).types();

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
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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
                    .is_superset(&set![OS_ASSET, OS_REPLACE, OS_INFLATION])
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
    transition.transition_type = TS_REPLACE;
    transition.assignments.remove(&OS_INFLATION).unwrap();
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
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::SchemaUnknownAssignmentType(
            opid,
            OS_INFLATION
        ))
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_remove_scripts_code() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let trusted_typesystem = AssetSchema::from(base_consignment.schema_id()).types();

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
        .map(|a| a.as_revealed_state().as_u64())
        .sum::<u64>();
    transition
        .assignments
        .insert(
            assignment_type,
            TypedAssigns::Fungible(
                NonEmptyVec::with(Assign::ConfidentialSeal {
                    seal: SecretSeal::strict_dumb(),
                    state: RevealedValue::new(output_sum + 1),
                })
                .into(),
            ),
        )
        .unwrap();
    replace_transition_in_bundle(witness_bundle, old_opid, transition);
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let mut scripts = base_consignment.scripts.clone().release();
    let mut lib = scripts.pop_last().unwrap().clone();
    let lib_id = lib.id();
    lib.code = none!();
    consignment.scripts = Confined::<BTreeSet<_>, 0, 1024>::from_checked(bset![lib]);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::MissingScript(_, lid)) if lid == lib_id
    ));
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_unmatching_transition_id() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let trusted_typesystem = AssetSchema::from(base_consignment.schema_id()).types();

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

    let alt_resolver =
        resolver.with_new_transaction(other_wbundle.pub_witness.tx().unwrap().clone());
    bundles.push(other_wbundle);
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(
            &alt_resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::TransitionIdMismatch(opid, transition.id()))
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_ifa() {
    let scenario = Scenario::C;
    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let trusted_typesystem = AssetSchema::from(base_consignment.schema_id()).types();

    // find a "transfer" transition that moves a "replace right" allocation
    let spent_replace_opouts = base_consignment
        .bundles
        .iter()
        .flat_map(|b| b.bundle.known_transitions.iter())
        .filter(|kt| kt.transition.transition_type == TS_TRANSFER)
        .flat_map(|kt| kt.transition.inputs.iter().map(|ti| (ti.op, ti.ty)))
        .collect::<HashSet<_>>();
    let mut witness_id = None;
    let mut base_transition = None;
    for wbun in base_consignment.bundled_witnesses() {
        for KnownTransition { opid, transition } in wbun.bundle.known_transitions.iter() {
            if transition.inputs.len() > 1
                && transition.inputs.iter().any(|i| i.ty == OS_REPLACE)
                && transition.transition_type == TS_TRANSFER
                // need its child later in the test
                && spent_replace_opouts.contains(&(*opid, OS_REPLACE))
            {
                witness_id = Some(wbun.witness_id());
                base_transition = Some(transition.clone());
                break;
            }
        }
    }
    let old_txid = witness_id.unwrap();
    let base_transition = base_transition.unwrap();
    let old_opid = base_transition.id();

    // Success: replace right can be shared with others without losing it
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_txid)
        .unwrap();
    let mut transition = base_transition.clone();
    let TypedAssigns::Declarative(replace_assignments) =
        transition.assignments.get_mut(&OS_REPLACE).unwrap()
    else {
        panic!("unexpected assignment type")
    };
    let replace_assignment = replace_assignments[0].clone();
    replace_assignments.push(replace_assignment).unwrap();
    let opid = transition.id();
    replace_transition_in_bundle(wbundle, old_opid, transition);
    let txid = wbundle.witness_id();
    update_transition_children(
        &mut bundles,
        HashMap::from([(old_opid, opid)]),
        HashMap::from([(old_txid, txid)]),
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment.clone().validate(
        &resolver,
        ChainNet::BitcoinRegtest,
        None,
        trusted_typesystem.clone(),
    );
    assert!(res.is_ok());

    // Error: replace rights cannot be burned via transfer operation (2 in, 1 out)
    // NOTE: reusing previous consignment to exploit duplicated allocation
    let mut consignment = consignment.clone();
    let mut bundles = consignment.bundles.release();
    let mut child_witness_id = None;
    let mut child_transition = None;
    for wbun in bundles.iter() {
        for KnownTransition { transition, .. } in wbun.bundle.known_transitions.iter() {
            if transition
                .inputs
                .iter()
                .any(|i| i.op == opid && i.ty == OS_REPLACE)
            {
                child_witness_id = Some(wbun.witness_id());
                child_transition = Some(transition);
                break;
            }
        }
    }
    let old_child_txid = child_witness_id.unwrap();
    let child_transition = child_transition.unwrap();
    let mut transition = child_transition.clone();
    let old_child_opid = transition.id();
    // add another replace right in input
    let mut replace_input = *transition
        .inputs
        .iter()
        .find(|i| i.ty == OS_REPLACE)
        .unwrap();
    replace_input.no = 1;
    transition.inputs.push(replace_input).unwrap();
    let child_opid = transition.id();
    assert!(
        transition
            .inputs
            .iter()
            .filter(|i| i.ty == OS_REPLACE)
            .count()
            == 2
    );
    assert!(transition.assignments.get(&OS_REPLACE).iter().len() == 1);
    assert_ne!(child_opid, old_child_opid);
    let child_wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_child_txid)
        .unwrap();
    child_wbundle
        .bundle
        .input_map
        .insert(replace_input, old_child_opid)
        .unwrap();
    replace_transition_in_bundle(child_wbundle, old_child_opid, transition);
    let child_txid = child_wbundle.witness_id();
    update_transition_children(
        &mut bundles,
        HashMap::from([(old_child_opid, child_opid)]),
        HashMap::from([(old_child_txid, child_txid)]),
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ScriptFailure(
            child_opid,
            Some(ERRNO_REPLACE_HIDDEN_BURN),
            None
        ))
    );

    // Error: replace rights cannot be created from thin air
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_txid)
        .unwrap();
    let mut transition = base_transition.clone();
    let old_opid = transition.id();
    let TypedAssigns::Declarative(replace_assignments) =
        transition.assignments.get_mut(&OS_REPLACE).unwrap()
    else {
        panic!("unexpected assignment type")
    };
    replace_assignments
        .push(Assign::revealed(
            BlindSeal::strict_dumb(),
            VoidState::strict_dumb(),
        ))
        .unwrap();
    transition.inputs = NonEmptyOrdSet::from_iter_checked(
        transition.inputs.into_iter().filter(|i| i.ty != OS_REPLACE),
    )
    .into();
    let opid = transition.id();
    replace_transition_in_bundle(wbundle, old_opid, transition);
    let txid = wbundle.witness_id();
    update_transition_children(
        &mut bundles,
        HashMap::from([(old_opid, opid)]),
        HashMap::from([(old_txid, txid)]),
    );
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ScriptFailure(
            opid,
            Some(ERRNO_REPLACE_NO_INPUT),
            None
        ))
    );

    // Error: replace rights cannot be burned via transfer operation (1 in, 0 out)
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_txid)
        .unwrap();
    let mut transition = base_transition.clone();
    let old_opid = transition.id();
    transition.assignments.remove(&OS_REPLACE).unwrap();
    let opid = transition.id();
    assert_ne!(opid, old_opid);
    replace_transition_in_bundle(wbundle, old_opid, transition);
    let txid = wbundle.witness_id();
    update_transition_children(
        &mut bundles,
        HashMap::from([(old_opid, opid)]),
        HashMap::from([(old_txid, txid)]),
    );
    remove_transition_children(&mut bundles, bset![opid], Some(OS_REPLACE));
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
        .unwrap_err();
    dbg!(&res);
    assert_eq!(
        res,
        ValidationError::InvalidConsignment(Failure::ScriptFailure(
            opid,
            Some(ERRNO_REPLACE_HIDDEN_BURN),
            None
        ))
    );

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
            .push(Assign::ConfidentialSeal {
                seal: SecretSeal::strict_dumb(),
                state: RevealedValue::new(Amount::ZERO),
            })
            .unwrap();
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        let txid = wbundle.witness_id();
        update_transition_children(
            &mut bundles,
            HashMap::from([(old_opid, opid)]),
            HashMap::from([(old_txid, txid)]),
        );
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &IndexedConsignment::new(&consignment),
        };
        let res = consignment
            .clone()
            .validate(
                &resolver,
                ChainNet::BitcoinRegtest,
                None,
                trusted_typesystem.clone(),
            )
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
        let value = assign.iter_mut().last().unwrap().as_revealed_state_mut();
        *value = RevealedValue::new(value.as_u64() + 1);
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &IndexedConsignment::new(&consignment),
        };
        let res = consignment
            .clone()
            .validate(
                &resolver,
                ChainNet::BitcoinRegtest,
                None,
                trusted_typesystem.clone(),
            )
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

    // test replace transition
    let mut witness_id = None;
    let mut base_transition = None;
    for wbun in base_consignment.bundled_witnesses() {
        for KnownTransition { transition, .. } in wbun.bundle.known_transitions.iter() {
            if transition.inputs.len() > 1 && transition.transition_type == TS_REPLACE {
                witness_id = Some(wbun.witness_id());
                base_transition = Some(transition.clone());
                break;
            }
        }
    }
    let old_txid = witness_id.unwrap();
    let base_transition = base_transition.unwrap();
    let old_opid = base_transition.id();

    // Error: replace transfers require all inputs
    for input in base_transition.inputs.iter() {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        transition.inputs = NonEmptyOrdSet::from_iter_checked(
            base_transition.inputs.into_iter().filter(|ti| ti != input),
        )
        .into();
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &IndexedConsignment::new(&consignment),
        };
        let res = consignment
            .clone()
            .validate(
                &resolver,
                ChainNet::BitcoinRegtest,
                None,
                trusted_typesystem.clone(),
            )
            .unwrap_err();
        dbg!(&res);
        let mismatch = OccurrencesMismatch {
            min: 1,
            max: 65535,
            found: 0,
        };
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::SchemaInputOccurrences(
                opid, input.ty, mismatch
            ))
        );
    }

    // Error: replace transitions can't inflate
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let wbundle = bundles
        .iter_mut()
        .find(|wb| wb.witness_id() == old_txid)
        .unwrap();
    let mut transition = base_transition.clone();
    let TypedAssigns::Fungible(assign) = transition.assignments.get_mut(&OS_ASSET).unwrap() else {
        panic!("unexpected asssignment type")
    };
    let value = assign.iter_mut().last().unwrap().as_revealed_state_mut();
    *value = RevealedValue::new(value.as_u64() + 1);
    let opid = transition.id();
    assert_ne!(opid, old_opid);
    replace_transition_in_bundle(wbundle, old_opid, transition);
    remove_transition_children(&mut bundles, bset![old_opid], None);
    consignment.bundles = LargeVec::from_checked(bundles);
    let resolver = OfflineResolver {
        consignment: &IndexedConsignment::new(&consignment),
    };
    let res = consignment
        .clone()
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem.clone(),
        )
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

    // Error: replace transitions can't burn allocations
    for assignment_type in [OS_ASSET, OS_REPLACE] {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        transition.assignments.remove(&assignment_type).unwrap();
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &IndexedConsignment::new(&consignment),
        };
        let res = consignment
            .clone()
            .validate(
                &resolver,
                ChainNet::BitcoinRegtest,
                None,
                trusted_typesystem.clone(),
            )
            .unwrap_err();
        dbg!(&res);
        let mismatch = OccurrencesMismatch {
            min: 1,
            max: 65535,
            found: 0,
        };
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::SchemaAssignmentOccurrences(
                opid,
                assignment_type,
                mismatch
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
        let value = assign.iter_mut().last().unwrap().as_revealed_state_mut();
        *value = RevealedValue::new(value.as_u64() + 1);
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &IndexedConsignment::new(&consignment),
        };
        let res = consignment
            .clone()
            .validate(
                &resolver,
                ChainNet::BitcoinRegtest,
                None,
                trusted_typesystem.clone(),
            )
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
    assert_eq!(
        input_assignment_types,
        set![OS_ASSET, OS_INFLATION, OS_REPLACE]
    );

    // Error: burn transitions can't have assignments
    for assignment_type in [OS_ASSET, OS_INFLATION, OS_REPLACE] {
        let mut consignment = base_consignment.clone();
        let mut bundles = consignment.bundles.release();
        let wbundle = bundles
            .iter_mut()
            .find(|wb| wb.witness_id() == old_txid)
            .unwrap();
        let mut transition = base_transition.clone();
        transition
            .assignments
            .insert(assignment_type, TypedAssigns::strict_dumb())
            .unwrap();
        let opid = transition.id();
        assert_ne!(opid, old_opid);
        replace_transition_in_bundle(wbundle, old_opid, transition);
        remove_transition_children(&mut bundles, bset![old_opid], None);
        consignment.bundles = LargeVec::from_checked(bundles);
        let resolver = OfflineResolver {
            consignment: &IndexedConsignment::new(&consignment),
        };
        let res = consignment
            .clone()
            .validate(
                &resolver,
                ChainNet::BitcoinRegtest,
                None,
                trusted_typesystem.clone(),
            )
            .unwrap_err();
        dbg!(&res);
        assert_eq!(
            res,
            ValidationError::InvalidConsignment(Failure::SchemaUnknownAssignmentType(
                opid,
                assignment_type
            ))
        );
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

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_typesystem_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let file = std::fs::File::open(cons_path).unwrap();
    let base_consignment: Value = serde_json::from_reader(file).unwrap();

    // modified type system will be detected
    let mut json_consignment = base_consignment.clone();
    let path = vec![
        Step::Key(s!("types")),
        Step::Idx(0),
        Step::Key(s!("List")),
        Step::Idx(1),
        Step::Key(s!("max")),
    ];
    let value_mut = get_entry_at_path_mut(&mut json_consignment, &path);
    *value_mut = Value::Number(u32::MAX.into());

    let consignment =
        serde_json::from_str::<Transfer>(&serde_json::to_string(&json_consignment).unwrap())
            .unwrap();
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let res = consignment
        .validate(
            &resolver,
            ChainNet::BitcoinRegtest,
            None,
            trusted_typesystem,
        )
        .unwrap_err();
    dbg!(&res);
    assert!(matches!(res, ValidationError::InvalidConsignment(_)));
}
