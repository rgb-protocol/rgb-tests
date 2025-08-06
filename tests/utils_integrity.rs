//! Tests in this module are about the integrity of some utility methods in this test suite.
//! RGB functionality tests shouldn't be added here.

pub mod utils;

use utils::*;

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(TT::Witness, TT::Witness, DT::Wpkh)]
#[case(TT::Witness, TT::Blinded, DT::Wpkh)]
#[case(TT::Blinded, TT::Witness, DT::Wpkh)]
#[case(TT::Blinded, TT::Blinded, DT::Wpkh)]
#[case(TT::Witness, TT::Witness, DT::Tr)]
#[case(TT::Witness, TT::Blinded, DT::Tr)]
#[case(TT::Blinded, TT::Witness, DT::Tr)]
#[case(TT::Blinded, TT::Blinded, DT::Tr)]
fn flexible_change_and_extra(
    #[case] transfer_type: TransferType,
    #[case] change_type: TransferType,
    #[case] wlt_desc: DescriptorType,
) {
    println!("transfer_type {transfer_type:?} change_type {change_type:?} wlt_desc {wlt_desc:?}");

    initialize();

    let mut wlt_1 = get_wallet(&wlt_desc);
    let mut wlt_2 = get_wallet(&wlt_desc);

    let utxo = wlt_1.get_utxo(Some(30_000));
    let contract_id_1 = wlt_1.issue_nia(600, Some(&utxo));
    let schema_id_1 = wlt_1.schema_id(contract_id_1);
    let contract_id_2 = wlt_1.issue_nia(100, Some(&utxo));
    let schema_id_2 = wlt_1.schema_id(contract_id_2);

    let mut sats = 3000;
    let destination = match transfer_type {
        TransferType::Witness => wlt_2.get_witness_info(Some(sats), None).into(),
        TransferType::Blinded => wlt_2.get_secret_seal(None, None).into(),
    };
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id_1,
            AssetColoringInfo {
                input_outpoints: vec![utxo],
                assignments: vec![AssetAssignment {
                    destination,
                    amount: 400,
                }],
            },
        )]),
        static_blinding: None,
        nonce: None,
        close_method: wlt_1.close_method(),
    };

    let change_utxo = match change_type {
        TransferType::Blinded => Some(wlt_1.get_utxo(None)),
        TransferType::Witness => None,
    };
    let (consignments, tx, _, tweak_info) =
        wlt_1.pay_full_flexible(coloring_info, None, change_utxo);
    wlt_1.mine_tx(&tx.txid(), false);
    for consignment in consignments.into_values() {
        wlt_2.accept_transfer(consignment.clone(), None);
    }
    if let Some((witness_info, tapret_commitment)) = tweak_info {
        wlt_2.add_tapret_tweak(witness_info.terminal(), tapret_commitment);
    }
    wlt_2.sync();
    wlt_1.sync();

    wlt_2.check_allocations(contract_id_1, schema_id_1, vec![400], false);
    wlt_1.check_allocations(contract_id_1, schema_id_1, vec![200], false);
    wlt_1.check_allocations(contract_id_2, schema_id_2, vec![100], false);

    sats -= 1000;
    // spend change
    wlt_1.send(&mut wlt_2, transfer_type, contract_id_1, 200, sats, None);
    // spend extra
    wlt_1.send(&mut wlt_2, transfer_type, contract_id_2, 100, sats, None);

    sats -= 1000;
    // send everything back to ensure it's spendable
    wlt_2.send(&mut wlt_1, transfer_type, contract_id_1, 600, sats, None);
    wlt_2.send(&mut wlt_1, transfer_type, contract_id_2, 100, sats, None);
    wlt_1.check_allocations(contract_id_1, schema_id_1, vec![600], false);
    wlt_1.check_allocations(contract_id_2, schema_id_2, vec![100], false);
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(DescriptorType::Tr, DescriptorType::Tr)]
#[case(DescriptorType::Wpkh, DescriptorType::Tr)]
#[case(DescriptorType::Tr, DescriptorType::Wpkh)]
#[case(DescriptorType::Wpkh, DescriptorType::Wpkh)]
fn flexible_wlt_descriptor_compatibility(
    #[case] descriptor_1: DescriptorType,
    #[case] descriptor_2: DescriptorType,
) {
    println!("descriptor_1 {descriptor_1:?} descriptor_2 {descriptor_2:?}");

    initialize();

    let mut wlt_1 = get_wallet(&descriptor_1);
    let mut wlt_2 = get_wallet(&descriptor_2);

    let utxo_1 = wlt_1.get_utxo(None);
    let contract_id = wlt_1.issue_nia(600, Some(&utxo_1));
    let schema_id = wlt_1.schema_id(contract_id);

    let utxo_2 = wlt_2.get_utxo(None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                assignments: vec![AssetAssignment {
                    destination: wlt_2.get_secret_seal(Some(utxo_2), None).into(),
                    amount: 400,
                }],
            },
        )]),
        static_blinding: None,
        nonce: None,
        close_method: wlt_1.close_method(),
    };
    let (consignments, tx, psbt_meta, tweak_info) =
        wlt_1.pay_full_flexible(coloring_info, None, None);
    wlt_1.mine_tx(&tx.txid(), false);
    for consignment in consignments.into_values() {
        wlt_2.accept_transfer(consignment.clone(), None);
    }
    if let Some((witness_info, tapret_commitment)) = tweak_info {
        wlt_2.add_tapret_tweak(witness_info.terminal(), tapret_commitment);
    }
    wlt_2.sync();
    let chg_utxo_1 = Outpoint::new(tx.txid(), psbt_meta.change_vout.unwrap());
    wlt_1.sync();

    let winfo_3 = wlt_1.get_witness_info(None, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_2],
                assignments: vec![AssetAssignment {
                    destination: AssetDestination::Witness(winfo_3.clone()),
                    amount: 100,
                }],
            },
        )]),
        static_blinding: None,
        nonce: None,
        close_method: CloseMethod::OpretFirst,
    };
    let (consignments, tx, _, tweak_info) = wlt_2.pay_full_flexible(coloring_info, None, None);
    wlt_2.mine_tx(&tx.txid(), false);
    for consignment in consignments.into_values() {
        wlt_1.accept_transfer(consignment.clone(), None);
    }
    if let Some((witness_info, tapret_commitment)) = tweak_info {
        wlt_1.add_tapret_tweak(witness_info.terminal(), tapret_commitment);
    }
    wlt_2.sync();
    let utxo_3 = Outpoint::new(tx.txid(), 1);

    let utxo_4 = wlt_2.get_utxo(None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![chg_utxo_1, utxo_3],
                assignments: vec![AssetAssignment {
                    destination: wlt_2.get_secret_seal(Some(utxo_4), None).into(),
                    amount: 300,
                }],
            },
        )]),
        static_blinding: None,
        nonce: None,
        close_method: wlt_1.close_method(),
    };
    let (consignments, tx, _, tweak_info) = wlt_1.pay_full_flexible(coloring_info, None, None);
    wlt_1.mine_tx(&tx.txid(), false);
    for consignment in consignments.into_values() {
        wlt_2.accept_transfer(consignment.clone(), None);
    }
    if let Some((witness_info, tapret_commitment)) = tweak_info {
        wlt_2.add_tapret_tweak(witness_info.terminal(), tapret_commitment);
    }
    wlt_2.sync();
    wlt_1.sync();

    wlt_1.check_allocations(contract_id, schema_id, vec![], false);
    wlt_2.check_allocations(contract_id, schema_id, vec![300, 300], false);
}

#[cfg(not(feature = "altered"))]
#[rstest]
fn flexible_multiple_transitions_per_vin() {
    initialize();

    let wlt_desc = DescriptorType::Wpkh;
    let mut wlt_1 = get_wallet(&wlt_desc);
    let mut wlt_2 = get_wallet(&wlt_desc);

    let utxo = wlt_1.get_utxo(Some(30_000));
    let contract_id_1 = wlt_1.issue_nia(600, Some(&utxo));
    let contract_id_2 = wlt_2.issue_nia(100, None);
    let schema_id_2 = wlt_2.schema_id(contract_id_2);
    let invoice = wlt_1.invoice(
        contract_id_2,
        schema_id_2,
        40,
        InvoiceType::Blinded(Some(utxo)),
    );
    wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(2000), None, None);
    let invoice = wlt_1.invoice(
        contract_id_2,
        schema_id_2,
        60,
        InvoiceType::Blinded(Some(utxo)),
    );
    wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(2000), None, None);

    let destination = wlt_2.get_secret_seal(None, None).into();
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id_1,
            AssetColoringInfo {
                input_outpoints: vec![utxo],
                assignments: vec![AssetAssignment {
                    destination,
                    amount: 400,
                }],
            },
        )]),
        static_blinding: None,
        nonce: None,
        close_method: wlt_1.close_method(),
    };
    let (consignments, tx, _, tweak_info) = wlt_1.pay_full_flexible(coloring_info, None, None);
    wlt_1.mine_tx(&tx.txid(), false);
    for consignment in consignments.into_values() {
        wlt_2.accept_transfer(consignment.clone(), None);
    }
    if let Some((witness_info, tapret_commitment)) = tweak_info {
        wlt_2.add_tapret_tweak(witness_info.terminal(), tapret_commitment);
    }
    wlt_2.sync();
    wlt_1.sync();

    wlt_1.check_allocations(contract_id_1, AssetSchema::Nia, vec![200], false);
    wlt_1.check_allocations(contract_id_2, AssetSchema::Nia, vec![40, 60], false);
    wlt_2.check_allocations(contract_id_1, AssetSchema::Nia, vec![400], false);

    let mut sats = 2000;

    // spend change
    wlt_1.send(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id_1,
        200,
        sats,
        None,
    );
    // spend extra
    wlt_1.send(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id_2,
        100,
        sats,
        None,
    );

    sats -= 1000;
    // send everything back to ensure it's spendable
    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id_1,
        600,
        sats,
        None,
    );
    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id_2,
        100,
        sats,
        None,
    );
    wlt_1.check_allocations(contract_id_1, AssetSchema::Nia, vec![600], false);
    wlt_1.check_allocations(contract_id_2, AssetSchema::Nia, vec![100], false);
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(DescriptorType::Wpkh)]
#[case(DescriptorType::Tr)]
fn flexible_tapret_no_change(#[case] recv_descriptor: DescriptorType) {
    println!("recv_descriptor {recv_descriptor:?}");

    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Tr);
    let mut wlt_2 = get_wallet(&recv_descriptor);

    let utxo = wlt_1.get_utxo(Some(800));
    let issued_amt = 666;
    let contract_id = wlt_1.issue_nia(issued_amt, Some(&utxo));
    let schema_id = wlt_1.schema_id(contract_id);

    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo],
                assignments: vec![AssetAssignment {
                    destination: wlt_2.get_witness_info(None, None).into(),
                    amount: issued_amt,
                }],
            },
        )]),
        static_blinding: None,
        nonce: None,
        close_method: CloseMethod::TapretFirst,
    };
    let (consignments, tx, _, tweak_info) = wlt_1.pay_full_flexible(coloring_info, None, None);
    wlt_1.mine_tx(&tx.txid(), false);
    for consignment in consignments.into_values() {
        wlt_2.accept_transfer(consignment.clone(), None);
    }
    if let Some((witness_info, tapret_commitment)) = tweak_info {
        wlt_2.add_tapret_tweak(witness_info.terminal(), tapret_commitment);
    } else if recv_descriptor == DescriptorType::Tr {
        panic!("tweak should go on beneficiary output");
    } else {
        assert!(tx.outputs().any(|o| o.script_pubkey.is_op_return()));
    }
    wlt_2.sync();

    wlt_2.check_allocations(contract_id, schema_id, vec![issued_amt], false);
}
