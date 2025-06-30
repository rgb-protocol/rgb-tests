pub mod utils;

use utils::*;

#[derive(Clone)]
enum MockResolvePubWitness {
    Success(Tx),
    Error(WitnessResolverError),
}

#[derive(Clone)]
enum MockResolvePubWitnessOrd {
    Success(WitnessOrd),
    Error(WitnessResolverError),
}

#[derive(Clone)]
struct MockResolver {
    pub_witnesses: HashMap<Txid, MockResolvePubWitness>,
    pub_witness_ords: HashMap<Txid, MockResolvePubWitnessOrd>,
}

impl ResolveWitness for MockResolver {
    fn resolve_pub_witness(&self, witness_id: Txid) -> Result<Tx, WitnessResolverError> {
        if let Some(res) = self.pub_witnesses.get(&witness_id) {
            match res {
                MockResolvePubWitness::Success(tx) => Ok(tx.clone()),
                MockResolvePubWitness::Error(err) => Err(err.clone()),
            }
        } else {
            Err(WitnessResolverError::Unknown(witness_id))
        }
    }

    fn resolve_pub_witness_ord(
        &self,
        witness_id: Txid,
    ) -> Result<WitnessOrd, WitnessResolverError> {
        if let Some(res) = self.pub_witness_ords.get(&witness_id) {
            match res {
                MockResolvePubWitnessOrd::Success(witness_ord) => Ok(*witness_ord),
                MockResolvePubWitnessOrd::Error(err) => Err(err.clone()),
            }
        } else {
            Err(WitnessResolverError::Unknown(witness_id))
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
        resolver
            .pub_witnesses
            .insert(witness_id, MockResolvePubWitness::Success(witness));
        resolver.pub_witness_ords.insert(
            witness_id,
            MockResolvePubWitnessOrd::Success(WitnessOrd::Tentative),
        );
        resolver
    }
}

#[derive(Debug, EnumIter, Copy, Clone, PartialEq)]
enum Scenario {
    A,
    B,
}

impl fmt::Display for Scenario {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Scenario {
    fn resolver(&self) -> MockResolver {
        match self {
            Self::A => {
                let (tx_1, witness_id_1) =
                    get_tx("b8880c28cf9163673b7e39f2af6b6fec952425354c17c74b0d5e69d3c467142b");
                let (tx_2, witness_id_2) =
                    get_tx("b411d8dd37353d243a527739fdc39cca22dbfe4fe92517ce16a33563803c5ad2");
                let (tx_3, witness_id_3) =
                    get_tx("b243f251cd06181c5568e041ed19106512886bf2c8617dfd3bf06c2321c5f8f4");
                MockResolver {
                    pub_witnesses: map![
                        witness_id_1 => MockResolvePubWitness::Success(tx_1),
                        witness_id_2 => MockResolvePubWitness::Success(tx_2),
                        witness_id_3 => MockResolvePubWitness::Success(tx_3),
                    ],
                    pub_witness_ords: map![
                        witness_id_1 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(106).unwrap(), 1726062111).unwrap())),
                        witness_id_2 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(108).unwrap(), 1726062111).unwrap())),
                        witness_id_3 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(110).unwrap(), 1726062112).unwrap())),
                    ],
                }
            }
            Self::B => {
                let (tx_1, witness_id_1) =
                    get_tx("143b34678a7e3e0d2dbbfbd14c6a163aed89d96e4992374154d3c1b5973a93cd");
                let (tx_2, witness_id_2) =
                    get_tx("84e3ac658455e8969e03ac02dc487c9ccd2fcb10314f9d19b0b223cfb85e7ed3");
                let (tx_3, witness_id_3) =
                    get_tx("333b0aea5cbf230791c57814de6dd86340e2626c1c1e8ac462f4f73c2645682c");
                MockResolver {
                    pub_witnesses: map![
                        witness_id_1 => MockResolvePubWitness::Success(tx_1),
                        witness_id_2 => MockResolvePubWitness::Success(tx_2),
                        witness_id_3 => MockResolvePubWitness::Success(tx_3),
                    ],
                    pub_witness_ords: map![
                        witness_id_1 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(105).unwrap(), 1726062423).unwrap())),
                        witness_id_2 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(106).unwrap(), 1726062423).unwrap())),
                        witness_id_3 => MockResolvePubWitnessOrd::Success(WitnessOrd::Mined(WitnessPos::bitcoin(NonZeroU32::new(106).unwrap(), 1726062423).unwrap())),
                    ],
                }
            }
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
    update_witness_and_anchor(witness_bundle, None)
}

fn update_witness_and_anchor(witness_bundle: &mut WitnessBundle, contract_id: Option<ContractId>) {
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

fn get_consignment(scenario: Scenario) -> (Transfer, Vec<Tx>) {
    initialize();

    let transfer_type = match scenario {
        Scenario::A => TransferType::Blinded,
        Scenario::B => TransferType::Witness,
    };

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issued_supply_1 = 999;
    let issued_supply_2 = 666;

    let sats = 9000;

    let utxo = wlt_1.get_utxo(None);
    let contract_id_1 = wlt_1.issue_nia(issued_supply_1, Some(&utxo));
    let contract_id_2 = wlt_1.issue_nia(issued_supply_2, Some(&utxo));

    let mut txes = vec![];

    let (_consignment, tx) = wlt_1.send(&mut wlt_2, transfer_type, contract_id_1, 66, sats, None);
    txes.push(tx);

    // spend asset that was moved automatically
    let (_consignment, tx) = wlt_1.send(&mut wlt_2, transfer_type, contract_id_2, 50, sats, None);
    txes.push(tx);

    // spend change of previous send
    let (consignment, tx) = wlt_1.send(&mut wlt_2, transfer_type, contract_id_2, 77, sats, None);
    txes.push(tx);

    (consignment, txes)
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
        Err(VarError::NotPresent) => Scenario::A,
        _ => panic!("invalid scenario"),
    };
    let (consignment, txes) = get_consignment(scenario);
    println!();
    let cons_path = format!("tests/fixtures/consignment_{scenario}.json");
    let json = serde_json::to_string_pretty(&consignment).unwrap();
    std::fs::write(&cons_path, json).unwrap();
    println!("written consignment in: {cons_path}");
    for tx in txes {
        let txid = tx.txid().to_string();
        let json = serde_json::to_string_pretty(&tx).unwrap();
        let json_path = format!("tests/fixtures/{txid}.json");
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

fn get_tx(txid: &str) -> (Tx, Txid) {
    let normalized_txid = txid.replace(":", "_");
    let json_path = format!("tests/fixtures/{normalized_txid}.json");
    let file = std::fs::File::open(json_path).unwrap();
    let tx: Tx = serde_json::from_reader(file).unwrap();
    let txid = Txid::from_str(txid).unwrap();
    (tx, txid)
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_success() {
    for scenario in Scenario::iter() {
        let resolver = scenario.resolver();
        let consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
        let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
        assert!(res.is_ok());
        let validation_status = match res {
            Ok(validated_consignment) => validated_consignment.validation_status().clone(),
            Err(status) => status,
        };
        dbg!(&validation_status);
        assert!(validation_status.failures.is_empty());
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
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err(status) => status,
    };
    dbg!(&validation_status);
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    assert_eq!(validation_status.failures.len(), 1);
    assert!(matches!(
        validation_status.failures[0],
        Failure::ContractChainNetMismatch(ChainNet::BitcoinRegtest)
    ));
    assert_eq!(validation_status.validity(), Validity::Invalid);
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_genesis_fail() {
    let resolver = Scenario::B.resolver();

    // schema ID: change genesis[schemaId] with CFA schema ID
    let consignment = get_consignment_from_json("attack_genesis_schema_id");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err(status) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 4);
    assert!(matches!(
        validation_status.failures[0],
        Failure::OperationAbsent(_)
    ));
    assert!(matches!(
        validation_status.failures[1],
        Failure::MpcInvalid(_, _, _)
    ));
    assert!(matches!(
        validation_status.failures[3],
        Failure::MpcInvalid(_, _, _)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);

    // genesis chainNet: change from bitcoinRegtest to bitcoinMainnet
    let consignment = get_consignment_from_json("attack_genesis_testnet");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err(status) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 1);
    assert!(matches!(
        validation_status.failures[0],
        Failure::ContractChainNetMismatch(_)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_bundles_fail() {
    let resolver = Scenario::A.resolver();

    // bundles first in time pubWitness inputs[0] sequence: change from 0 to 1
    let consignment = get_consignment_from_json("attack_bundles_pubWitness_data_input_sequence");
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err(status) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 3);
    assert!(matches!(
        validation_status.failures[0],
        Failure::SealNoPubWitness(_, _, _)
    ));
    assert!(matches!(
        validation_status.failures[1],
        Failure::SealsInvalid(_, _, _)
    ));
    assert!(matches!(
        validation_status.failures[2],
        Failure::WitnessMissingInput(_, _, _)
    ));
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    let validity = validation_status.validity();
    assert_eq!(validity, Validity::Invalid);
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_resolver_error() {
    let scenario = Scenario::A;
    let base_resolver = scenario.resolver();
    let consignment = get_consignment_from_json("attack_resolver_error");
    let txid =
        Txid::from_str("b411d8dd37353d243a527739fdc39cca22dbfe4fe92517ce16a33563803c5ad2").unwrap();
    let wbundle = consignment
        .bundles
        .iter()
        .find(|wb| wb.witness_id() == txid)
        .unwrap();
    let bundle_id = wbundle.bundle.bundle_id();

    struct ConsignmentResolver<'a, 'cons, const TRANSFER: bool> {
        consignment: &'cons IndexedConsignment<'cons, TRANSFER>,
        fallback: &'a MockResolver,
    }
    impl<const TRANSFER: bool> ResolveWitness for ConsignmentResolver<'_, '_, TRANSFER> {
        fn resolve_pub_witness(&self, witness_id: Txid) -> Result<Tx, WitnessResolverError> {
            self.consignment
                .pub_witness(witness_id)
                .and_then(|p| p.tx().cloned())
                .ok_or(WitnessResolverError::Unknown(witness_id))
                .or_else(|_| self.fallback.resolve_pub_witness(witness_id))
        }
        fn resolve_pub_witness_ord(&self, _: Txid) -> Result<WitnessOrd, WitnessResolverError> {
            Ok(WitnessOrd::Tentative)
        }
        fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
            Ok(())
        }
    }

    // resolve_pub_witness error
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::Other(txid, s!("unexpected error"));
    *resolver.pub_witnesses.get_mut(&txid).unwrap() =
        MockResolvePubWitness::Error(resolver_error.clone());
    let consignment_resolver = ConsignmentResolver {
        consignment: &IndexedConsignment::new(&consignment),
        fallback: &resolver,
    };
    let res = consignment
        .clone()
        .validate(&consignment_resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err(status) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 1);
    assert_eq!(
        validation_status.failures[0],
        Failure::SealNoPubWitness(bundle_id, txid, resolver_error)
    );
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    assert_eq!(validation_status.validity(), Validity::Invalid);
    assert_eq!(
        validation_status,
        consignment
            .clone()
            .validate(&resolver, ChainNet::BitcoinRegtest, None)
            .unwrap_err()
    );

    // resolve_pub_witness_ord error
    let mut resolver = base_resolver.clone();
    let resolver_error = WitnessResolverError::Other(txid, s!("another unexpected error"));
    *resolver.pub_witness_ords.get_mut(&txid).unwrap() =
        MockResolvePubWitnessOrd::Error(resolver_error.clone());
    let res = consignment
        .clone()
        .validate(&resolver, ChainNet::BitcoinRegtest, None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err(status) => status,
    };
    dbg!(&validation_status);
    assert_eq!(validation_status.failures.len(), 1);
    assert_eq!(
        validation_status.failures[0],
        Failure::WitnessUnresolved(bundle_id, txid, resolver_error)
    );
    assert!(validation_status.warnings.is_empty());
    assert!(validation_status.info.is_empty());
    assert_eq!(validation_status.validity(), Validity::Invalid);
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_schema_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));
    let transition_type = base_consignment.schema.transitions.keys().last().unwrap();

    // SchemaOpMetaTypeUnknown: schema transition has unknown metatype
    let mut consignment = base_consignment.clone();
    consignment
        .schema
        .transitions
        .get_mut(transition_type)
        .unwrap()
        .transition_schema
        .metadata = TinyOrdSet::from_checked(bset![MetaType::with(42)]);
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 1);
    assert!(matches!(
        failures[0],
        Failure::SchemaOpMetaTypeUnknown(_, _)
    ));

    // SchemaOpEmptyInputs: schema transition has no inputs
    let mut consignment = base_consignment.clone();
    consignment
        .schema
        .transitions
        .get_mut(transition_type)
        .unwrap()
        .transition_schema
        .inputs = TinyOrdMap::new();
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 1);
    assert!(matches!(failures[0], Failure::SchemaOpEmptyInputs(_)));

    // SchemaOpGlobalTypeUnknown: schema transition has unknown global type
    let mut consignment = base_consignment.clone();
    consignment
        .schema
        .transitions
        .get_mut(transition_type)
        .unwrap()
        .transition_schema
        .globals = TinyOrdMap::from_checked(bmap! {
        GlobalStateType::with(42) => Occurrences::Once
    });
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 1);
    assert!(matches!(
        failures[0],
        Failure::SchemaOpGlobalTypeUnknown(_, _)
    ));

    // SchemaOpAssignmentTypeUnknown: schema transition has unknown assignment type
    let mut consignment = base_consignment.clone();
    consignment
        .schema
        .transitions
        .get_mut(transition_type)
        .unwrap()
        .transition_schema
        .assignments = TinyOrdMap::from_checked(bmap! {
        AssignmentType::with(42) => Occurrences::Once
    });
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 1);
    assert!(matches!(
        failures[0],
        Failure::SchemaOpAssignmentTypeUnknown(_, _)
    ));

    // SchemaMetaSemIdUnknown: schema meta type has unknown sem id
    let mut consignment = base_consignment.clone();
    consignment.schema.meta_types =
        TinyOrdMap::from_checked(bmap! {MetaType::with(42) => MetaDetails {
            sem_id: SemId::from([42u8; 32]),
            name: fname!("foo")
        }});
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 1);
    assert!(matches!(failures[0], Failure::SchemaMetaSemIdUnknown(_, _)));

    // SchemaGlobalSemIdUnknown: schema global type has unknown sem id
    let mut consignment = base_consignment.clone();
    let mut global_types = consignment.schema.global_types.release();
    global_types.insert(
        GlobalStateType::with(42),
        GlobalDetails {
            global_state_schema: GlobalStateSchema {
                sem_id: SemId::from([42u8; 32]),
                max_items: u24::from_le_bytes([42u8; 3]),
            },
            name: fname!("foo"),
        },
    );
    consignment.schema.global_types = TinyOrdMap::from_checked(global_types);
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 1);
    assert!(matches!(
        failures[0],
        Failure::SchemaGlobalSemIdUnknown(_, _)
    ));

    // SchemaOwnedSemIdUnknown: schema owned type has unknown sem id
    let mut consignment = base_consignment.clone();
    let mut owned_types = consignment.schema.owned_types.release();
    owned_types.insert(
        AssignmentType::with(56),
        AssignmentDetails {
            owned_state_schema: OwnedStateSchema::Structured(SemId::from([42u8; 32])),
            default_transition: TransitionType::with(42),
            name: fname!("foo"),
        },
    );
    consignment.schema.owned_types = TinyOrdMap::from_checked(owned_types);
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 1);
    dbg!(&failures);
    assert!(matches!(
        failures[0],
        Failure::SchemaOwnedSemIdUnknown(_, _)
    ));
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_commitments_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));

    // CyclicGraph: it's enough to make the same opid appear twice, changing InputMap and
    // PubWitness to avoid automated removal of duplicates
    let mut consignment = base_consignment.clone();
    let bundle = consignment.bundles.iter().last().unwrap();
    let mut new_bundle = bundle.clone();
    new_bundle.pub_witness = PubWitness::new(
        Txid::from_str("a6e7e4775ea6e8b7155f6cfcc8d193df326c468dce99e9c4ee0a26511659feb3").unwrap(),
    );
    new_bundle
        .bundle
        .input_map
        .insert(Opout::strict_dumb(), OpId::strict_dumb())
        .unwrap();

    consignment.bundles = LargeVec::from_checked(vec![bundle.clone(), new_bundle]);
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert!(failures
        .iter()
        .any(|f| matches!(f, Failure::CyclicGraph(_))));

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
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 2);
    assert!(matches!(failures[0], Failure::ExtraKnownTransition(_)));
    assert!(matches!(failures[1], Failure::DoubleSpend(_)));

    // OperationAbsent: remove a bundle that contains spent assignments
    let mut consignment = base_consignment.clone();
    let spent_transitions = consignment
        .bundles
        .iter()
        .flat_map(|b| b.bundle.known_transitions.as_unconfined())
        .flat_map(|kt| kt.transition.inputs.iter())
        .map(|ti| ti.op)
        .collect::<HashSet<_>>();
    let bundle_id_to_remove = consignment
        .bundles
        .iter()
        .map(|wb| wb.clone().bundle)
        .find(|b| {
            spent_transitions
                .iter()
                .any(|st| b.known_transitions_contain_opid(st))
        })
        .unwrap()
        .bundle_id();
    consignment.bundles = LargeVec::from_checked(
        consignment
            .bundles
            .into_iter()
            .filter(|b| b.bundle.bundle_id() != bundle_id_to_remove)
            .collect::<Vec<_>>(),
    );
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert!(failures
        .iter()
        .any(|f| matches!(f, Failure::OperationAbsent(_))));

    // NoPrevState: add input with missing assignment type to a transition
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
    let fst_input = *transition.inputs.as_unconfined().first().unwrap();
    transition
        .inputs
        .push(Opout {
            ty: AssignmentType::with(42),
            ..fst_input
        })
        .unwrap();
    new_bundle
        .bundle
        .known_transitions
        .push(KnownTransition::new(transition.id(), transition))
        .unwrap();

    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert!(failures.iter().any(|f| matches!(
        f,
        Failure::NoPrevState {
            opid: _,
            prev_id: _,
            state_type: _
        }
    )));

    // NoPrevOut: add input with missing assignment number to a transition
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
    let fst_input = *transition.inputs.as_unconfined().first().unwrap();
    transition
        .inputs
        .push(Opout {
            no: 42,
            ..fst_input
        })
        .unwrap();
    new_bundle
        .bundle
        .known_transitions
        .push(KnownTransition::new(transition.id(), transition))
        .unwrap();
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert!(failures
        .iter()
        .any(|f| matches!(f, Failure::NoPrevOut(_, _))));

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
    let transition = transitions
        .iter_mut()
        .find(|kt| spent_transitions.contains(&kt.opid))
        .map(|kt| &mut kt.transition)
        .unwrap();
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
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert!(failures
        .iter()
        .any(|f| matches!(f, Failure::ConfidentialSeal(_))));

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
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 1);
    assert!(matches!(failures[0], Failure::ExtraKnownTransition(_)));
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_logic_fail() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));

    // SchemaMismatch: replace consignment.schema with a compatible schema with different id
    let mut consignment = base_consignment.clone();
    let schema_id = consignment.schema_id();
    let mut alt_schema = NonInflatableAsset::schema();
    alt_schema.name = tn!("NonInflatableAsset2");
    let alt_schema_id = alt_schema.schema_id();
    consignment.schema = alt_schema;
    let res = consignment.validate(&resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0],
        Failure::SchemaMismatch {
            expected: schema_id,
            actual: alt_schema_id,
        }
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
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0],
        Failure::SchemaUnknownTransitionType(transition_id, TransitionType::with(42))
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
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0],
        Failure::SchemaUnknownMetaType(transition_id, MetaType::with(42))
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
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0],
        Failure::SchemaUnknownGlobalStateType(transition_id, GlobalStateType::with(42))
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
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0],
        Failure::SchemaUnknownAssignmentType(transition_id, AssignmentType::with(42))
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
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(
        failures[0],
        Failure::SchemaAssignmentOccurrences(
            transition_id,
            AssignmentType::with(4000),
            OccurrencesMismatch {
                min: 1,
                max: 65535,
                found: 0
            }
        )
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
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(
        failures[0],
        Failure::StateTypeMismatch {
            opid: transition_id,
            state_type: assignment_type,
            expected: StateType::Fungible,
            found: StateType::Void
        }
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
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0],
        Failure::ScriptFailure(transition_id, Some(0), None)
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
    update_witness_and_anchor(witness_bundle, Some(old_contract_id));
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    dbg!(&failures);
    assert_eq!(failures.len(), 1);
    assert_eq!(
        failures[0],
        Failure::ContractMismatch(transition_id, ContractId::strict_dumb())
    );

    // Error: zero-amount allocations are not allowed
    let mut consignment = base_consignment.clone();
    let mut bundles = consignment.bundles.release();
    let mut witness_bundle = bundles.last_mut().unwrap();
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
    replace_transition_in_bundle(&mut witness_bundle, old_opid, transition);
    let alt_resolver =
        resolver.with_new_transaction(witness_bundle.pub_witness.tx().unwrap().clone());
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment
        .validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0], Failure::ScriptFailure(opid, Some(0), None));
}

#[cfg(not(feature = "altered"))]
#[test]
fn validate_consignment_unmatching_transition_id() {
    let scenario = Scenario::B;
    let resolver = scenario.resolver();

    let base_consignment = get_consignment_from_json(&format!("consignment_{scenario}"));

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
    if let Some(existing) = witness_bundle
        .bundle
        .known_transitions
        .iter_mut()
        .find(|kt| kt.opid == opid)
    {
        existing.transition = transition.clone();
    }

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
    update_witness_and_anchor(&mut other_wbundle, Some(contract_id));

    let alt_resolver =
        resolver.with_new_transaction(other_wbundle.pub_witness.tx().unwrap().clone());
    bundles.push(other_wbundle);
    consignment.bundles = LargeVec::from_checked(bundles);
    let res = consignment.validate(&alt_resolver, ChainNet::BitcoinRegtest, None);
    let failures = res.unwrap_err().failures;
    assert_eq!(failures.len(), 2);
    assert_eq!(
        failures[0],
        Failure::TransitionIdMismatch(opid, transition.id())
    );
    assert_eq!(failures[1], Failure::OperationAbsent(OpId::strict_dumb()));
}
