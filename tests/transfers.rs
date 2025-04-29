pub mod utils;

use utils::*;

#[cfg(not(feature = "altered"))]
#[rstest]
// blinded: nia - nia
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Nia)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Nia, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Nia, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Nia, AS::Nia)]
// blinded: nia - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Cfa)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Nia, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Nia, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Nia, AS::Cfa)]
// blinded: nia - uda
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Uda)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Nia, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Nia, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Nia, AS::Uda)]
// blinded: cfa - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Cfa, AS::Cfa)]
// blinded: cfa - nia
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Nia)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Cfa, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Cfa, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Cfa, AS::Nia)]
// blinded: cfa - uda
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Uda)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Cfa, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Cfa, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Cfa, AS::Uda)]
// blinded: uda - uda
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Uda)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Uda)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Uda)]
// blinded: uda - nia
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Nia)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Nia)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Nia)]
// blinded: uda - cfa
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Cfa)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr, AS::Uda, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Wpkh, AS::Uda, AS::Cfa)]
#[case(TT::Blinded, DT::Tr, DT::Tr, AS::Uda, AS::Cfa)]
// witness: nia - nia
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Nia)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Nia, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Nia, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Nia, AS::Nia)]
// witness: nia - cfa
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Cfa)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Nia, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Nia, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Nia, AS::Cfa)]
// witness: nia - uda
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Nia, AS::Uda)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Nia, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Nia, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Nia, AS::Uda)]
// witness: cfa - cfa
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Cfa, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Cfa, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Cfa, AS::Cfa)]
// witness: cfa - nia
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Nia)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Cfa, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Cfa, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Cfa, AS::Nia)]
// witness: cfa - uda
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Cfa, AS::Uda)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Cfa, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Cfa, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Cfa, AS::Uda)]
// witness: uda - uda
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Uda)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Uda)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Uda)]
// witness: uda - nia
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Nia)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Nia)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Nia)]
// witness: uda - cfa
#[case(TT::Witness, DT::Wpkh, DT::Wpkh, AS::Uda, AS::Cfa)]
#[case(TT::Witness, DT::Wpkh, DT::Tr, AS::Uda, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Wpkh, AS::Uda, AS::Cfa)]
#[case(TT::Witness, DT::Tr, DT::Tr, AS::Uda, AS::Cfa)]
fn transfer_loop(
    #[case] transfer_type: TransferType,
    #[case] wlt_1_desc: DescriptorType,
    #[case] wlt_2_desc: DescriptorType,
    #[case] asset_schema_1: AssetSchema,
    #[case] asset_schema_2: AssetSchema,
) {
    println!(
        "transfer_type {transfer_type:?} wlt_1_desc {wlt_1_desc:?} wlt_2_desc {wlt_2_desc:?} \
        asset_schema_1 {asset_schema_1:?} asset_schema_2 {asset_schema_2:?}"
    );

    initialize();

    let mut wlt_1 = get_wallet(&wlt_1_desc);
    let mut wlt_2 = get_wallet(&wlt_2_desc);

    let issued_supply_1 = 999;
    let issued_supply_2 = 666;

    let mut sats = 9000;

    // wlt_1 issues 2 assets on the same UTXO
    let utxo = wlt_1.get_utxo(None);
    let contract_id_1 = match asset_schema_1 {
        AssetSchema::Nia => wlt_1.issue_nia(issued_supply_1, Some(&utxo)),
        AssetSchema::Uda => wlt_1.issue_uda(Some(&utxo)),
        AssetSchema::Cfa => wlt_1.issue_cfa(issued_supply_1, Some(&utxo)),
        _ => unreachable!(),
    };
    let contract_id_2 = match asset_schema_2 {
        AssetSchema::Nia => wlt_1.issue_nia(issued_supply_2, Some(&utxo)),
        AssetSchema::Uda => wlt_1.issue_uda(Some(&utxo)),
        AssetSchema::Cfa => wlt_1.issue_cfa(issued_supply_2, Some(&utxo)),
        _ => unreachable!(),
    };
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![issued_supply_1], true);
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2], true);

    // wlt_1 spends asset 1, automatically moving the others
    let amount_1 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        99
    };
    wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_1,
        amount_1,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1],
        false,
    );
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2], true);
    wlt_2.check_allocations(contract_id_1, asset_schema_1, vec![amount_1], true);

    // wlt_1 spends asset 1 change (only if possible)
    let amount_2 = 33;
    if asset_schema_1 != AssetSchema::Uda {
        wlt_1.send(
            &mut wlt_2,
            transfer_type,
            contract_id_1,
            amount_2,
            sats,
            None,
        );
        wlt_1.check_allocations(
            contract_id_1,
            asset_schema_1,
            vec![issued_supply_1 - amount_1 - amount_2],
            false,
        );
        wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![issued_supply_2], true);
        wlt_2.check_allocations(
            contract_id_1,
            asset_schema_1,
            vec![amount_1, amount_2],
            true,
        );
    }

    // wlt_1 spends asset 2
    let amount_3 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        22
    };
    wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_2,
        amount_3,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2],
        false,
    );
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3],
        false,
    );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1, amount_2],
        true,
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3], true);

    // wlt_2 spends received allocation(s) of asset 1
    let amount_4 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        111
    };
    sats -= 1000;
    wlt_2.send(
        &mut wlt_1,
        transfer_type,
        contract_id_1,
        amount_4,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2, amount_4],
        true,
    );
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3],
        false,
    );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
        false,
    );
    wlt_2.check_allocations(contract_id_2, asset_schema_2, vec![amount_3], true);

    // wlt_2 spends asset 2
    let amount_5 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        11
    };
    sats -= 1000;
    wlt_2.send(
        &mut wlt_1,
        transfer_type,
        contract_id_2,
        amount_5,
        sats,
        None,
    );
    wlt_1.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![issued_supply_1 - amount_1 - amount_2, amount_4],
        true,
    );
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3, amount_5],
        true,
    );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4],
        false,
    );
    wlt_2.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![amount_3 - amount_5],
        false,
    );

    // wlt_1 spends asset 1, received back
    let amount_6 = if asset_schema_1 == AssetSchema::Uda {
        1
    } else {
        issued_supply_1 - amount_1 - amount_2 + amount_4
    };
    sats -= 1000;
    wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_1,
        amount_6,
        sats,
        None,
    );
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![], false);
    wlt_1.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![issued_supply_2 - amount_3, amount_5],
        true,
    );
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4, amount_6],
        true,
    );
    wlt_2.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![amount_3 - amount_5],
        false,
    );

    // wlt_1 spends asset 2, received back
    let amount_7 = if asset_schema_2 == AssetSchema::Uda {
        1
    } else {
        issued_supply_2 - amount_3 + amount_5
    };
    sats -= 1000;
    wlt_1.send(
        &mut wlt_2,
        transfer_type,
        contract_id_2,
        amount_7,
        sats,
        None,
    );
    wlt_1.check_allocations(contract_id_1, asset_schema_1, vec![], false);
    wlt_1.check_allocations(contract_id_2, asset_schema_2, vec![], false);
    wlt_2.check_allocations(
        contract_id_1,
        asset_schema_1,
        vec![amount_1 + amount_2 - amount_4, amount_6],
        true,
    );
    wlt_2.check_allocations(
        contract_id_2,
        asset_schema_2,
        vec![amount_3 - amount_5, amount_7],
        true,
    );
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(AS::Nia)]
#[case(AS::Cfa)]
#[case(AS::Uda)]
#[case(AS::Pfa)]
fn unknown_kit(#[case] asset_schema: AssetSchema) {
    println!("asset_schema {asset_schema:?}");

    initialize();

    let mut wlt_1 = get_wallet_custom(&DescriptorType::Wpkh, None, false);
    let mut wlt_2 = get_wallet_custom(&DescriptorType::Wpkh, None, false);

    let (contract_id, secret_key) = match asset_schema {
        AssetSchema::Nia => (wlt_1.issue_nia(600, None), None),
        AssetSchema::Uda => (wlt_1.issue_uda(None), None),
        AssetSchema::Cfa => (wlt_1.issue_cfa(600, None), None),
        AssetSchema::Pfa => {
            let (secret_key, public_key) =
                Secp256k1::new().generate_keypair(&mut rand::thread_rng());
            let pubkey = CompressedPk::from_byte_array(public_key.serialize()).unwrap();
            (wlt_1.issue_pfa(600, None, pubkey), Some(secret_key))
        }
    };

    if asset_schema == AssetSchema::Pfa {
        wlt_1.send_pfa(
            &mut wlt_2,
            TransferType::Blinded,
            contract_id,
            1,
            secret_key.unwrap(),
        );
    } else {
        wlt_1.send(
            &mut wlt_2,
            TransferType::Blinded,
            contract_id,
            1,
            2000,
            None,
        );
    }
}

#[cfg(not(feature = "altered"))]
#[test]
fn rbf_transfer() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 600;
    let contract_id = wlt_1.issue_nia(issue_supply, None);
    let schema_id = wlt_1.schema_id(contract_id);

    stop_mining();
    let initial_height = get_height();

    let amount = 400;
    let invoice = wlt_2.invoice(contract_id, schema_id, amount, InvoiceType::Witness);
    let (consignment, _, _, _) = wlt_1.pay_full(invoice.clone(), None, Some(500), true, None);

    wlt_2.accept_transfer(consignment.clone(), None);

    // retry with higher fees, TX hasn't been mined
    let mid_height = get_height();
    assert_eq!(initial_height, mid_height);

    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, Some(1000), true, None);

    let final_height = get_height();
    assert_eq!(initial_height, final_height);

    wlt_1.mine_tx(&tx.txid(), true);
    wlt_2.accept_transfer(consignment.clone(), None);
    wlt_1.sync_and_update_witnesses(None);
    wlt_2.sync_and_update_witnesses(None);

    wlt_1.check_allocations(contract_id, schema_id, vec![issue_supply - amount], false);
    wlt_2.check_allocations(contract_id, schema_id, vec![amount], false);

    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id,
        amount,
        1000,
        None,
    );
}

#[cfg(feature = "altered")]
#[rstest]
#[should_panic(expected = "DoubleSpend")]
#[case(TransferType::Blinded)]
#[should_panic(expected = "Composition(InsufficientState)")]
#[case(TransferType::Witness)]
fn same_transfer_twice_no_update_witnesses(#[case] transfer_type: TransferType) {
    println!("transfer_type {transfer_type:?}");

    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 2000;
    let contract_id = wlt_1.issue_nia(issue_supply, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let amount = 100;
    let invoice = wlt_2.invoice(contract_id, schema_id, amount, transfer_type.into());
    let _ = wlt_1.pay_full(invoice.clone(), None, Some(500), false, None);

    let (consignment, _, _, _) = wlt_1.pay_full(invoice, None, Some(1000), true, None);

    wlt_2.accept_transfer(consignment, None);

    // with TransferType::Blinded this shows duplicated allocations
    wlt_2.debug_logs(contract_id, AllocationFilter::WalletAll);

    let allocations = match transfer_type {
        TransferType::Blinded => vec![amount, amount],
        TransferType::Witness => vec![amount],
    };
    wlt_2.check_allocations(contract_id, schema_id, allocations, false);

    // with TransferType::Blinded the receiver will detect a double spend, to avoid this the
    // sendert should call update_witnesses when retrying the same transfer twice
    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id,
        amount * 2,
        1000,
        None,
    );

    if transfer_type == TransferType::Blinded {
        unreachable!("should have panicked at previous send");
    }

    // with TransferType::Blinded this shows 1900+200 as owned, but we issued 2000
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);

    // with TransferType::Blinded this works but should fail
    wlt_1.send(
        &mut wlt_3,
        TransferType::Blinded,
        contract_id,
        issue_supply + amount,
        1000,
        None,
    );
    // with TransferType::Blinded this shows 2100 as owned, but we issued 2000
    wlt_3.debug_logs(contract_id, AllocationFilter::WalletAll);
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(TransferType::Blinded)]
#[case(TransferType::Witness)]
fn same_transfer_twice_update_witnesses(#[case] transfer_type: TransferType) {
    println!("transfer_type {transfer_type:?}");

    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 2000;
    let contract_id = wlt_1.issue_nia(issue_supply, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let amount = 100;
    let invoice = wlt_2.invoice(contract_id, schema_id, amount, transfer_type.into());
    let _ = wlt_1.pay_full(invoice.clone(), None, Some(500), false, None);

    wlt_1.sync_and_update_witnesses(None);

    // with TransferType::Blinded this fails with an AbsentValidWitness error
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, Some(1000), true, None);

    wlt_1.mine_tx(&tx.txid(), false);
    wlt_2.accept_transfer(consignment, None);
    wlt_1.sync();

    wlt_1.check_allocations(contract_id, schema_id, vec![issue_supply - amount], false);
    wlt_2.check_allocations(contract_id, schema_id, vec![amount], false);

    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id,
        amount,
        1000,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(TT::Blinded)]
#[case(TT::Witness)]
fn invoice_reuse(#[case] transfer_type: TransferType) {
    println!("transfer_type {transfer_type:?}");

    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let asset_info = AssetInfo::default_nia(vec![500, 400]);
    let contract_id = wlt_1.issue_with_info(asset_info, vec![None, None], None, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let amount = 300;
    let invoice = wlt_2.invoice(contract_id, schema_id, amount, transfer_type.into());
    wlt_1.send_to_invoice(&mut wlt_2, invoice.clone(), Some(500), None, None);
    let (consignment, _) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(600), None, None);

    wlt_1.check_allocations(contract_id, schema_id, vec![200, 100], false);
    wlt_2.check_allocations(contract_id, schema_id, vec![amount, amount], false);

    // with TransferType::Blinded this fails: bundle for 1st transfer is also included
    assert_eq!(consignment.bundles.len(), 1);
}

#[cfg(not(feature = "altered"))]
#[test]
fn accept_0conf() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 600;
    let contract_id = wlt_1.issue_nia(issue_supply, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let amt = 200;
    let invoice = wlt_2.invoice(contract_id, schema_id, amt, InvoiceType::Witness);
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice.clone(), None, None, true, None);
    let txid = tx.txid();

    wlt_2.accept_transfer(consignment.clone(), None);

    // wlt_2 sees the allocation even if TX has not been mined
    wlt_2.check_allocations(contract_id, schema_id, vec![amt], false);

    wlt_1.sync();

    let wlt_1_change_amt = issue_supply - amt;

    // wlt_1 needs to get tentative allocations to see its change from the unmined TX
    let allocations: Vec<FungibleAllocation> = wlt_1
        .contract_fungible_allocations(contract_id, true)
        .into_iter()
        .filter(|fa| fa.seal.txid() == Some(txid))
        .collect();
    assert_eq!(allocations.len(), 1);
    assert!(allocations
        .iter()
        .any(|fa| fa.state == Amount::from(wlt_1_change_amt)));

    // after mining, wlt_1 doesn't need to get tentative allocations to see the change
    mine(false);
    wlt_1.sync();
    wlt_1.check_allocations(contract_id, schema_id, vec![wlt_1_change_amt], false);
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(false)]
#[case(true)]
fn ln_transfers(#[case] update_witnesses_before_htlc: bool) {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let pre_funding_height = get_height();

    let utxo_1 = wlt_1.get_utxo(Some(10_000));
    let utxo_2 = wlt_1.get_utxo(Some(20_000));
    let amounts = vec![600, 300];
    let outpoints = vec![Some(utxo_1), Some(utxo_2)];
    let asset_info = AssetInfo::default_nia(amounts.clone());
    let contract_id = wlt_1.issue_with_info(asset_info, outpoints, None, None);

    struct LNFasciaResolver {}
    impl ResolveWitness for LNFasciaResolver {
        fn resolve_pub_witness(&self, _: Txid) -> Result<Tx, WitnessResolverError> {
            unreachable!()
        }
        fn resolve_pub_witness_ord(&self, _: Txid) -> Result<WitnessOrd, WitnessResolverError> {
            Ok(WitnessOrd::Ignored)
        }
        fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
            unreachable!()
        }
    }

    println!("\n1. fake commitment TX (no HTLCs)");
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(2000)),
        (wlt_1.get_address(), None),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo_1], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                output_map: HashMap::from([(0, 100), (1, 500)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info.clone());
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);
    let txid_same_bundle_1 = psbt.txid();
    let coloring_info_same_bundle = coloring_info;

    let htlc_vout = 2;
    let htlc_rgb_amt = 200;
    let htlc_btc_amt = 4000;
    let htlc_derived_addr = wlt_1.get_derived_address(true);

    // no problem: since there's no htlc for this commitment
    wlt_1.sync_and_update_witnesses(Some(pre_funding_height));

    println!("\n2. fake commitment TX (1 HTLC)");
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(2000)),
        (wlt_1.get_address(), None),
        (htlc_derived_addr.addr, Some(htlc_btc_amt)),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo_1], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                output_map: HashMap::from([(0, 100), (1, 300), (htlc_vout, htlc_rgb_amt)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    if update_witnesses_before_htlc {
        wlt_1.sync_and_update_witnesses(Some(pre_funding_height));
    }

    println!("\n3. fake HTLC TX");
    let txid = fascia.witness_id();
    let input_outpoint = Outpoint::new(txid, htlc_vout);
    let beneficiaries = vec![(wlt_1.get_address(), None)];
    let (mut psbt, _meta) = wlt_1.construct_psbt_offchain(
        vec![(
            input_outpoint,
            htlc_btc_amt,
            htlc_derived_addr.terminal,
            htlc_derived_addr.addr.script_pubkey(),
        )],
        beneficiaries,
        None,
    );
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![input_outpoint],
                output_map: HashMap::from([(0, htlc_rgb_amt)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    println!("\n4. fake commitment TX (no HTLCs)");
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(3000)),
        (wlt_1.get_address(), None),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo_1], beneficiaries, None);
    let coloring_info = coloring_info_same_bundle;
    let (mut fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);
    let mut txid_same_bundle_2 = psbt.txid();
    let mut offset = 0;
    // this will make sure that in select_valid_witness the first TXID will be the one with
    // WitnessOrd::Ignored, when we want the one with WitnessOrd::Mined to be selected instead
    while txid_same_bundle_1 > txid_same_bundle_2 {
        psbt.fallback_locktime = LockTime::from_height(offset);
        txid_same_bundle_2 = psbt.txid();
        offset += 1;
    }
    fascia.seal_witness.public = PubWitness::with(psbt.to_unsigned_tx().into());
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    let mut old_psbt = psbt.clone();

    println!("\n5. fake commitment TX (1 HTLC)");
    let htlc_rgb_amt = 180;
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(2000)),
        (wlt_1.get_address(), None),
        (htlc_derived_addr.addr, Some(htlc_btc_amt)),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo_1], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                output_map: HashMap::from([(0, 122), (1, 298), (htlc_vout, htlc_rgb_amt)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info.clone());
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    if update_witnesses_before_htlc {
        wlt_1.sync_and_update_witnesses(Some(pre_funding_height));
    }

    println!("\n6. fake HTLC TX");
    let txid = fascia.witness_id();
    let input_outpoint = Outpoint::new(txid, htlc_vout);
    let beneficiaries = vec![(wlt_1.get_address(), None)];
    let (mut psbt, _meta) = wlt_1.construct_psbt_offchain(
        vec![(
            input_outpoint,
            htlc_btc_amt,
            htlc_derived_addr.terminal,
            htlc_derived_addr.addr.script_pubkey(),
        )],
        beneficiaries,
        None,
    );
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![input_outpoint],
                output_map: HashMap::from([(0, htlc_rgb_amt)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    // no problem: since the force-close tx will be updated to mined soon
    wlt_1.sync_and_update_witnesses(Some(pre_funding_height));

    println!("\n7. fake commitment TX (1 HTLC) on 2nd channel");
    let htlc_rgb_amt_2nd_chan = 10;
    let beneficiaries = vec![
        (wlt_2.get_address(), Some(2000)),
        (wlt_1.get_address(), None),
        (htlc_derived_addr.addr, Some(htlc_btc_amt)),
    ];
    let (mut psbt, _meta) = wlt_1.construct_psbt(vec![utxo_2], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_2],
                output_map: HashMap::from([(0, 20), (1, 270), (htlc_vout, htlc_rgb_amt_2nd_chan)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    println!("\n8. broadcast old PSBT");
    let tx = wlt_1.sign_finalize_extract(&mut old_psbt);
    wlt_1.broadcast_tx(&tx);
    let txid = tx.txid();
    wlt_1.mine_tx(&txid, false);
    wlt_1.sync();
    wlt_1.update_witnesses(pre_funding_height, vec![txid]);
    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);
    wlt_1.send(
        &mut wlt_3,
        TransferType::Blinded,
        contract_id,
        500,
        1000,
        None,
    );

    println!("\n9. fake HTLC TX on 2nd channel");
    let txid = fascia.witness_id();
    let input_outpoint = Outpoint::new(txid, htlc_vout);
    let beneficiaries = vec![(wlt_1.get_address(), None)];
    let (mut psbt, _meta) = wlt_1.construct_psbt_offchain(
        vec![(
            input_outpoint,
            htlc_btc_amt,
            htlc_derived_addr.terminal,
            htlc_derived_addr.addr.script_pubkey(),
        )],
        beneficiaries,
        None,
    );
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![input_outpoint],
                output_map: HashMap::from([(0, htlc_rgb_amt_2nd_chan)]),
                static_blinding: Some(666),
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX),
    };
    let (fascia, _asset_beneficiaries) = wlt_1.color_psbt(&mut psbt, coloring_info);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[should_panic(expected = "InvoiceBeneficiaryWrongChainNet(BitcoinMainnet, BitcoinRegtest)")]
#[case(false)]
#[should_panic(expected = "ContractChainNetMismatch(BitcoinMainnet)")]
#[case(true)]
fn mainnet_wlt_receiving_test_asset(#[case] custom_invoice: bool) {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_mainnet_wallet();

    let contract_id = wlt_1.issue_nia(700, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let utxo =
        Outpoint::from_str("bebcfcb200a17763f6932a6d6fca9448a4b46c5b737cc3810769a7403ef79ce6:0")
            .unwrap();
    let mut invoice = wlt_2.invoice(
        contract_id,
        schema_id,
        150,
        InvoiceType::Blinded(Some(utxo)),
    );
    if custom_invoice {
        invoice.beneficiary = XChainNet::BitcoinRegtest(invoice.beneficiary.into_inner());
    }
    wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);
}

#[cfg(not(feature = "altered"))]
#[test]
fn sync_mainnet_wlt() {
    initialize();

    let mut wlt_1 = get_mainnet_wallet();

    // sometimes this fails with a 'Too many requests' error when using esplora
    wlt_1.sync();
}

#[cfg(not(feature = "altered"))]
#[test]
fn collaborative_transfer() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);

    let sats = 30_000;

    let utxo_0 = wlt_1.get_utxo(Some(sats));
    let contract_id = wlt_1.issue_nia(600, Some(&utxo_0));
    let (_, tx) = wlt_1.send(
        &mut wlt_2,
        TransferType::Witness,
        contract_id,
        200,
        18_000,
        None,
    );
    let utxo_1 = Outpoint::new(tx.txid(), 2); // change: 11_600 sat
    let utxo_2 = Outpoint::new(tx.txid(), 1); // transfered: 18_000 sat

    let mut psbt = Psbt::default();

    wlt_1.psbt_add_input(&mut psbt, utxo_1);
    wlt_2.psbt_add_input(&mut psbt, utxo_2);

    psbt.construct_output_expect(
        wlt_3.get_address().script_pubkey(),
        Sats::from_sats(sats - 2 * DEFAULT_FEE_ABS),
    );

    let coloring_info_1 = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                output_map: HashMap::from([(0, 400)]),
                static_blinding: None,
            },
        )]),
        static_blinding: None,
        nonce: None,
    };
    let coloring_info_2 = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_2],
                output_map: HashMap::from([(0, 200)]),
                static_blinding: None,
            },
        )]),
        static_blinding: None,
        nonce: None,
    };
    let beneficiaries_1 = wlt_1.color_psbt_init(&mut psbt, coloring_info_1);

    let (fascia, beneficiaries_2) = wlt_2.color_psbt(&mut psbt, coloring_info_2);

    wlt_1.sign_finalize(&mut psbt);
    let tx = wlt_2.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);

    wlt_1.consume_fascia(fascia.clone(), tx.txid());
    wlt_2.consume_fascia(fascia, tx.txid());

    let consignments_1 = wlt_1.create_consignments(beneficiaries_1, tx.txid());
    let consignments_2 = wlt_2.create_consignments(beneficiaries_2, tx.txid());

    println!("Send the whole asset amount back to wlt_1 to check new allocations are spendable");
    for consignment in vec![consignments_1, consignments_2].into_iter().flatten() {
        wlt_3.accept_transfer(consignment, None);
    }
    wlt_3.send(
        &mut wlt_1,
        TransferType::Witness,
        contract_id,
        600,
        sats - 4 * DEFAULT_FEE_ABS,
        None,
    );
    wlt_1.send(
        &mut wlt_2,
        TransferType::Witness,
        contract_id,
        600,
        sats - 6 * DEFAULT_FEE_ABS,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn receive_from_unbroadcasted_transfer_to_blinded() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);

    let contract_id = wlt_1.issue_nia(600, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let utxo = wlt_2.get_utxo(None);
    mine(false);
    let invoice = wlt_2.invoice(
        contract_id,
        schema_id,
        100,
        InvoiceType::Blinded(Some(utxo)),
    );
    // create transfer but do not broadcast its TX
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice.clone(), None, Some(500), false, None);
    let witness_id = tx.txid();

    struct OffchainResolver<'a, 'cons, const TRANSFER: bool> {
        witness_id: Txid,
        consignment: &'cons IndexedConsignment<'cons, TRANSFER>,
        fallback: &'a AnyResolver,
    }
    impl<const TRANSFER: bool> ResolveWitness for OffchainResolver<'_, '_, TRANSFER> {
        fn resolve_pub_witness(&self, witness_id: Txid) -> Result<Tx, WitnessResolverError> {
            self.consignment
                .pub_witness(witness_id)
                .and_then(|p| p.tx().cloned())
                .ok_or(WitnessResolverError::Unknown(witness_id))
                .or_else(|_| self.fallback.resolve_pub_witness(witness_id))
        }
        fn resolve_pub_witness_ord(
            &self,
            witness_id: Txid,
        ) -> Result<WitnessOrd, WitnessResolverError> {
            if witness_id != self.witness_id {
                return self.fallback.resolve_pub_witness_ord(witness_id);
            }
            Ok(WitnessOrd::Tentative)
        }
        fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
            Ok(())
        }
    }

    let resolver = OffchainResolver {
        witness_id,
        consignment: &IndexedConsignment::new(&consignment),
        fallback: &wlt_2.get_resolver(),
    };

    // wlt_2 use custom resolver to be able to send the assets even if transfer TX sending to
    // blinded UTXO has not been broadcasted
    wlt_2.accept_transfer_custom_resolver(consignment.clone(), None, &resolver);

    let invoice = wlt_3.invoice(contract_id, schema_id, 50, InvoiceType::Witness);
    let (consignment, tx, _, _) = wlt_2.pay_full(invoice, Some(2000), None, true, None);
    wlt_2.mine_tx(&tx.txid(), false);

    // consignment validation fails because it notices an unbroadcasted TX in the history
    let res = consignment.validate(&wlt_3.get_resolver(), wlt_3.chain_net(), None);
    assert!(res.is_err());
    let validation_status = match res {
        Ok(validated_consignment) => validated_consignment.validation_status().clone(),
        Err((status, _consignment)) => status,
    };
    assert_eq!(validation_status.failures.len(), 1);
    assert!(matches!(
        validation_status.failures[0],
        Failure::SealNoPubWitness(_, _, _)
    ));
}

#[cfg(not(feature = "altered"))]
#[test]
fn check_fungible_history() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 600;

    let contract_id = wlt_1.issue_nia(issue_supply, None);

    wlt_1.debug_contracts();
    wlt_1.debug_history(contract_id, false);

    wlt_1.check_history_operation(&contract_id, None, OpDirection::Issued, issue_supply);

    let amt = 200;

    let (_, tx) = wlt_1.send(
        &mut wlt_2,
        TransferType::Witness,
        contract_id,
        amt,
        1000,
        None,
    );
    let txid = tx.txid();

    wlt_1.debug_history(contract_id, false);
    wlt_2.debug_history(contract_id, false);

    wlt_1.check_history_operation(&contract_id, Some(&txid), OpDirection::Sent, amt);

    wlt_2.check_history_operation(&contract_id, Some(&txid), OpDirection::Received, amt);
}

#[cfg(not(feature = "altered"))]
#[test]
fn send_to_oneself() {
    initialize();

    let mut wlt = get_wallet(&DescriptorType::Wpkh);

    let issue_supply = 600;

    let contract_id = wlt.issue_nia(issue_supply, None);
    let schema_id = wlt.schema_id(contract_id);

    let amt = 200;

    let invoice = wlt.invoice(contract_id, schema_id, amt, InvoiceType::Witness);

    let (consignment, tx, _, _) = wlt.pay_full(invoice.clone(), None, None, true, None);
    wlt.mine_tx(&tx.txid(), false);
    wlt.accept_transfer(consignment, None);
    wlt.sync();

    wlt.debug_history(contract_id, false);
    let history = wlt.history(contract_id);
    // only issue operation is found, because self-transfers should not appear in history
    assert_eq!(history.len(), 1);

    wlt.debug_logs(contract_id, AllocationFilter::WalletAll);
    wlt.check_allocations(contract_id, schema_id, vec![amt, issue_supply - amt], true);
}

#[cfg(not(feature = "altered"))]
#[test]
fn tapret_opret_same_utxo() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Tr);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_3 = get_wallet(&DescriptorType::Wpkh);

    let contract_id_1 = wlt_1.issue_nia(600, None);
    let schema_id_1 = wlt_1.schema_id(contract_id_1);
    let contract_id_2 = wlt_2.issue_nia(800, None);
    let schema_id_2 = wlt_2.schema_id(contract_id_2);

    let utxo = wlt_3.get_utxo(None);
    mine(false);

    let invoice = wlt_3.invoice(
        contract_id_1,
        schema_id_1,
        100,
        InvoiceType::Blinded(Some(utxo)),
    );
    wlt_1.send_to_invoice(&mut wlt_3, invoice, Some(1000), None, None);

    let invoice = wlt_3.invoice(
        contract_id_2,
        schema_id_2,
        550,
        InvoiceType::Blinded(Some(utxo)),
    );
    wlt_2.send_to_invoice(&mut wlt_3, invoice, Some(1000), None, None);

    wlt_3.send(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id_1,
        70,
        1000,
        None,
    );

    wlt_3.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id_2,
        20,
        1000,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn multiple_transitions_per_vin() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let contract_id_1 = wlt_1.issue_nia(600, None);
    let schema_id_1 = wlt_1.schema_id(contract_id_1);
    let contract_id_2 = wlt_1.issue_nia(800, None);
    let schema_id_2 = wlt_1.schema_id(contract_id_2);

    let utxo = wlt_2.get_utxo(None);
    mine(false);
    let invoice = wlt_2.invoice(
        contract_id_1,
        schema_id_1,
        100,
        InvoiceType::Blinded(Some(utxo)),
    );
    wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);
    let invoice = wlt_2.invoice(
        contract_id_1,
        schema_id_1,
        200,
        InvoiceType::Blinded(Some(utxo)),
    );
    wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);
    let invoice = wlt_2.invoice(
        contract_id_2,
        schema_id_2,
        550,
        InvoiceType::Blinded(Some(utxo)),
    );
    wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

    // this will create an input_map with a vin associated to 2 transitions when moving
    // contract_id_1 automatically
    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id_2,
        500,
        1000,
        None,
    );

    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id_1,
        20,
        1000,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn tapret_commitments_on_beneficiary_output() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Tr);
    let mut wlt_2 = get_wallet(&DescriptorType::Tr);

    let sats = 3000;
    let issued_amt = 600;

    let utxo = wlt_1.get_utxo(Some(sats));
    let contract_id = wlt_1.issue_nia(issued_amt, Some(&utxo));
    let schema_id = wlt_1.schema_id(contract_id);

    // put tapret commitment on beneficiary output
    let invoice_1 = wlt_2.invoice(
        contract_id,
        schema_id,
        issued_amt,
        InvoiceType::WitnessTapret,
    );
    let (consignment, tx) = wlt_1.send_to_invoice(
        &mut wlt_2,
        invoice_1.clone(),
        Some(sats - DEFAULT_FEE_ABS),
        None,
        None,
    );
    assert_eq!(tx.outputs.len(), 1);
    let mut beneficiary_address_1 = None;
    if let Beneficiary::WitnessVout(pay2vout, _) = invoice_1.beneficiary.into_inner() {
        beneficiary_address_1 = Some(pay2vout.into_address(wlt_2.network().into()));
    }
    if tx.outputs().last().unwrap().script_pubkey != beneficiary_address_1.unwrap().script_pubkey()
    {
        wlt_2.try_add_tapret_tweak(consignment.clone(), &tx.txid());
        wlt_2.sync();
    } else {
        panic!("unexpected");
    }
    wlt_2.check_allocations(contract_id, schema_id, vec![issued_amt], false);

    // make sure that tapret commitment goes on bitcoin change if it exists
    let change_amt = 1;
    let amt = issued_amt - change_amt;
    let invoice_2 = wlt_1.invoice(contract_id, schema_id, amt, InvoiceType::WitnessTapret);
    let (_, tx) = wlt_2.send_to_invoice(&mut wlt_1, invoice_2.clone(), Some(1000), None, None);
    assert_eq!(tx.outputs.len(), 2);
    let mut beneficiary_address = None;
    if let Beneficiary::WitnessVout(pay2vout, _) = invoice_2.beneficiary.into_inner() {
        beneficiary_address = Some(pay2vout.into_address(wlt_1.network().into()));
    }
    assert_eq!(
        tx.outputs().last().unwrap().script_pubkey,
        beneficiary_address.unwrap().script_pubkey()
    );
    wlt_1.check_allocations(contract_id, schema_id, vec![amt], false);
    wlt_2.check_allocations(contract_id, schema_id, vec![change_amt], false);

    // send back assets to allow invoice reuse at next step
    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id,
        change_amt,
        500,
        None,
    );
    wlt_1.check_allocations(contract_id, schema_id, vec![change_amt, amt], false);

    // invoice reuse to check multiple tweaks work
    let (consignment, tx) =
        wlt_1.send_to_invoice(&mut wlt_2, invoice_1.clone(), Some(100000600), None, None);
    assert_eq!(tx.outputs.len(), 1);
    let mut beneficiary_address_2 = None;
    if let Beneficiary::WitnessVout(pay2vout, _) = invoice_1.beneficiary.into_inner() {
        beneficiary_address_2 = Some(pay2vout.into_address(wlt_2.network().into()));
    }
    if tx.outputs().last().unwrap().script_pubkey != beneficiary_address_2.unwrap().script_pubkey()
    {
        wlt_2.try_add_tapret_tweak(consignment.clone(), &tx.txid());
        wlt_2.sync();
    } else {
        panic!("unexpected");
    }
    if let RgbDescr::TapretKey(tr) = wlt_2.descriptor() {
        assert_eq!(tr.tweaks.len(), 3);
        assert_eq!(tr.tweaks.values().flatten().count(), 4);
        // 2 tweaks on the same terminal
        assert!(tr.tweaks.iter().any(|(_, c)| c.len() == 2));
    } else {
        unreachable!()
    }
    wlt_2.check_allocations(contract_id, schema_id, vec![issued_amt], false);

    // send bitcoins to untweaked address
    let sats_pre = wlt_2.balance();
    fund_wallet(
        beneficiary_address_1.unwrap().to_string(),
        Some(sats),
        INSTANCE_1,
    );
    wlt_2.sync();
    let sats_post = wlt_2.balance();
    assert_eq!(sats_post, sats_pre + sats);
}

#[cfg(not(feature = "altered"))]
#[test]
fn pfa() {
    initialize();

    let mut wlt_1 = get_wallet(&DescriptorType::Wpkh);
    let mut wlt_2 = get_wallet(&DescriptorType::Wpkh);

    let (secret_key, public_key) = Secp256k1::new().generate_keypair(&mut rand::thread_rng());
    let pubkey = CompressedPk::from_byte_array(public_key.serialize()).unwrap();

    let utxo = wlt_1.get_utxo(None);

    let issued_amt_1 = 600;
    let contract_id_1 = wlt_1.issue_pfa(issued_amt_1, Some(&utxo), pubkey);
    let schema_id_1 = wlt_1.schema_id(contract_id_1);

    let issued_amt_2 = 400;
    let contract_id_2 = wlt_1.issue_pfa(issued_amt_2, Some(&utxo), pubkey);
    let schema_id_2 = wlt_1.schema_id(contract_id_2);

    let amt_1 = 42;
    wlt_1.send_pfa(
        &mut wlt_2,
        TransferType::Witness,
        contract_id_1,
        amt_1,
        secret_key,
    );

    let amt_2 = 66;
    wlt_1.send_pfa(
        &mut wlt_2,
        TransferType::Witness,
        contract_id_2,
        amt_2,
        secret_key,
    );

    wlt_1.check_allocations(
        contract_id_1,
        schema_id_1,
        vec![issued_amt_1 - amt_1],
        false,
    );
    wlt_2.check_allocations(contract_id_1, schema_id_1, vec![amt_1], false);
    wlt_1.check_allocations(
        contract_id_2,
        schema_id_2,
        vec![issued_amt_2 - amt_2],
        false,
    );
    wlt_2.check_allocations(contract_id_2, schema_id_2, vec![amt_2], false);
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(HistoryType::Linear, ReorgType::ChangeOrder)]
#[case(HistoryType::Linear, ReorgType::Revert)]
#[case(HistoryType::Branching, ReorgType::ChangeOrder)]
#[case(HistoryType::Branching, ReorgType::Revert)]
#[case(HistoryType::Merging, ReorgType::ChangeOrder)]
#[case(HistoryType::Merging, ReorgType::Revert)]
#[serial]
fn reorg_history(#[case] history_type: HistoryType, #[case] reorg_type: ReorgType) {
    println!("history_type {history_type:?} reorg_type {reorg_type:?}");

    initialize();
    connect_reorg_nodes();

    let mut wlt_1 = get_wallet_custom(&DescriptorType::Wpkh, Some(INSTANCE_2), true);
    let mut wlt_2 = get_wallet_custom(&DescriptorType::Wpkh, Some(INSTANCE_2), true);

    let contract_id = match history_type {
        HistoryType::Linear | HistoryType::Branching => wlt_1.issue_nia(600, None),
        HistoryType::Merging => {
            let asset_info = AssetInfo::default_nia(vec![400, 200]);
            wlt_1.issue_with_info(asset_info, vec![None, None], None, None)
        }
    };
    let schema_id = wlt_1.schema_id(contract_id);

    let utxo_wlt_1_1 = wlt_1.get_utxo(None);
    let utxo_wlt_1_2 = wlt_1.get_utxo(None);
    let utxo_wlt_2_1 = wlt_2.get_utxo(None);
    let utxo_wlt_2_2 = wlt_2.get_utxo(None);
    mine_custom(false, INSTANCE_2, 6);
    disconnect_reorg_nodes();

    let txs = match history_type {
        HistoryType::Linear => {
            let amt_0 = 590;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_0,
                InvoiceType::Blinded(Some(utxo_wlt_2_1)),
            );
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            let amt_1 = 100;
            let invoice = wlt_1.invoice(
                contract_id,
                schema_id,
                amt_1,
                InvoiceType::Blinded(Some(utxo_wlt_1_1)),
            );
            let (_, tx_1) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            let amt_2 = 80;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_2,
                InvoiceType::Blinded(Some(utxo_wlt_2_2)),
            );
            let (_, tx_2) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            vec![tx_0, tx_1, tx_2]
        }
        HistoryType::Branching => {
            let amt_0 = 600;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_0,
                InvoiceType::Blinded(Some(utxo_wlt_2_1)),
            );
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            let amt_1 = 200;
            let invoice = wlt_1.invoice(
                contract_id,
                schema_id,
                amt_1,
                InvoiceType::Blinded(Some(utxo_wlt_1_1)),
            );
            let (_, tx_1) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            let amt_2 = amt_0 - amt_1 - 1;
            let invoice = wlt_1.invoice(
                contract_id,
                schema_id,
                amt_2,
                InvoiceType::Blinded(Some(utxo_wlt_1_2)),
            );
            let (_, tx_2) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            vec![tx_0, tx_1, tx_2]
        }
        HistoryType::Merging => {
            let amt_0 = 400;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_0,
                InvoiceType::Blinded(Some(utxo_wlt_2_1)),
            );
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, None, None, None);

            let amt_1 = 200;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_1,
                InvoiceType::Blinded(Some(utxo_wlt_2_2)),
            );
            let (_, tx_1) = wlt_1.send_to_invoice(&mut wlt_2, invoice, None, None, None);

            let amt_2 = amt_0 + amt_1 - 1;
            let invoice = wlt_1.invoice(
                contract_id,
                schema_id,
                amt_2,
                InvoiceType::Blinded(Some(utxo_wlt_1_1)),
            );
            let (_, tx_2) = wlt_2.send_to_invoice(&mut wlt_1, invoice, None, None, None);

            vec![tx_0, tx_1, tx_2]
        }
    };

    match (history_type, reorg_type) {
        (HistoryType::Linear, ReorgType::ChangeOrder) => {
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[0], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 10;
            let wlt_1_alloc_2 = 20;
            let wlt_2_alloc_1 = 490;
            let wlt_2_alloc_2 = 80;
            wlt_1.check_allocations(
                contract_id,
                schema_id,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
                false,
            );
            wlt_2.check_allocations(
                contract_id,
                schema_id,
                vec![wlt_2_alloc_1, wlt_2_alloc_2],
                false,
            );
        }
        (HistoryType::Linear | HistoryType::Branching, ReorgType::Revert) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 600;
            wlt_1.check_allocations(contract_id, schema_id, vec![wlt_1_alloc_1], false);
            wlt_2.check_allocations(contract_id, schema_id, vec![], false);
        }
        (HistoryType::Branching, ReorgType::ChangeOrder) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            broadcast_tx_and_mine(&txs[0], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 200;
            let wlt_1_alloc_2 = 399;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(
                contract_id,
                schema_id,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
                false,
            );
            wlt_2.check_allocations(contract_id, schema_id, vec![wlt_2_alloc_1], false);
        }
        (HistoryType::Merging, ReorgType::ChangeOrder) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[0], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 599;
            let wlt_2_alloc_1 = 1;
            wlt_1.check_allocations(contract_id, schema_id, vec![wlt_1_alloc_1], false);
            wlt_2.check_allocations(contract_id, schema_id, vec![wlt_2_alloc_1], false);
        }
        (HistoryType::Merging, ReorgType::Revert) => {
            broadcast_tx_and_mine(&txs[1], INSTANCE_3);
            broadcast_tx_and_mine(&txs[2], INSTANCE_3);
            wlt_1.switch_to_instance(INSTANCE_3);
            wlt_2.switch_to_instance(INSTANCE_3);
            let wlt_1_alloc_1 = 400;
            let _wlt_2_alloc_1 = 200;
            wlt_1.check_allocations(contract_id, schema_id, vec![wlt_1_alloc_1], false);
            // this checks 0 allocations instead of vec![_wlt_2_alloc_1]
            // because funds are burnt in this case
            // to avoid this sender & acceptor should check mining depth of history when merging
            wlt_2.check_allocations(contract_id, schema_id, vec![], false);
        }
    }

    mine_custom(false, INSTANCE_3, 3);
    connect_reorg_nodes();
    mine_custom(false, INSTANCE_2, 3);
    wlt_1.switch_to_instance(INSTANCE_2);
    wlt_2.switch_to_instance(INSTANCE_2);

    let mut wlt_3 = get_wallet_custom(&DescriptorType::Wpkh, Some(INSTANCE_2), true);

    match history_type {
        HistoryType::Linear => {
            let wlt_1_alloc_1 = 10;
            let wlt_1_alloc_2 = 20;
            let wlt_1_amt = wlt_1_alloc_1 + wlt_1_alloc_2;
            let wlt_2_alloc_1 = 490;
            let wlt_2_alloc_2 = 80;
            let wlt_2_amt = wlt_2_alloc_1 + wlt_2_alloc_2;
            wlt_1.check_allocations(
                contract_id,
                schema_id,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
                false,
            );
            wlt_2.check_allocations(
                contract_id,
                schema_id,
                vec![wlt_2_alloc_1, wlt_2_alloc_2],
                false,
            );
            wlt_1.send(
                &mut wlt_3,
                TransferType::Witness,
                contract_id,
                wlt_1_amt,
                1000,
                None,
            );
            wlt_2.send(
                &mut wlt_3,
                TransferType::Witness,
                contract_id,
                wlt_2_amt,
                1000,
                None,
            );
            wlt_3.check_allocations(contract_id, schema_id, vec![wlt_1_amt, wlt_2_amt], false);
        }
        HistoryType::Branching => {
            let wlt_1_alloc_1 = 200;
            let wlt_1_alloc_2 = 399;
            let wlt_1_amt = wlt_1_alloc_1 + wlt_1_alloc_2;
            let wlt_2_alloc_1 = 1;
            let wlt_2_amt = wlt_2_alloc_1;
            wlt_1.check_allocations(
                contract_id,
                schema_id,
                vec![wlt_1_alloc_1, wlt_1_alloc_2],
                false,
            );
            wlt_2.check_allocations(contract_id, schema_id, vec![wlt_2_alloc_1], false);
            wlt_1.send(
                &mut wlt_3,
                TransferType::Witness,
                contract_id,
                wlt_1_amt,
                1000,
                None,
            );
            wlt_2.send(
                &mut wlt_3,
                TransferType::Witness,
                contract_id,
                wlt_2_amt,
                1000,
                None,
            );
            wlt_3.check_allocations(contract_id, schema_id, vec![wlt_1_amt, wlt_2_amt], false);
        }
        HistoryType::Merging => {
            let wlt_1_alloc_1 = 599;
            let wlt_1_amt = wlt_1_alloc_1;
            let wlt_2_alloc_1 = 1;
            let wlt_2_amt = wlt_2_alloc_1;
            wlt_1.check_allocations(contract_id, schema_id, vec![wlt_1_alloc_1], false);
            wlt_2.check_allocations(contract_id, schema_id, vec![wlt_2_alloc_1], false);
            wlt_1.send(
                &mut wlt_3,
                TransferType::Witness,
                contract_id,
                wlt_1_amt,
                1000,
                None,
            );
            wlt_2.send(
                &mut wlt_3,
                TransferType::Witness,
                contract_id,
                wlt_2_amt,
                1000,
                None,
            );
            wlt_3.check_allocations(contract_id, schema_id, vec![wlt_1_amt, wlt_2_amt], false);
        }
    }
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(HistoryType::Linear)]
#[case(HistoryType::Branching)]
#[case(HistoryType::Merging)]
#[serial]
fn reorg_revert_multiple(#[case] history_type: HistoryType) {
    println!("history_type {history_type:?}");

    initialize();
    connect_reorg_nodes();

    let mut wlt_1 = get_wallet_custom(&DescriptorType::Wpkh, Some(INSTANCE_2), true);
    let mut wlt_2 = get_wallet_custom(&DescriptorType::Wpkh, Some(INSTANCE_2), true);

    let contract_id = match history_type {
        HistoryType::Linear | HistoryType::Branching => wlt_1.issue_nia(600, None),
        HistoryType::Merging => {
            let asset_info = AssetInfo::default_nia(vec![400, 200]);
            wlt_1.issue_with_info(asset_info, vec![None, None], None, None)
        }
    };
    let schema_id = wlt_1.schema_id(contract_id);

    let utxo_wlt_1_1 = wlt_1.get_utxo(None);
    let utxo_wlt_1_2 = wlt_1.get_utxo(None);
    let utxo_wlt_2_1 = wlt_2.get_utxo(None);
    let utxo_wlt_2_2 = wlt_2.get_utxo(None);
    mine_custom(false, INSTANCE_2, 6);
    disconnect_reorg_nodes();

    let txs = match history_type {
        HistoryType::Linear => {
            let amt_0 = 590;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_0,
                InvoiceType::Blinded(Some(utxo_wlt_2_1)),
            );
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            let amt_1 = 100;
            let invoice = wlt_1.invoice(
                contract_id,
                schema_id,
                amt_1,
                InvoiceType::Blinded(Some(utxo_wlt_1_1)),
            );
            let (_, tx_1) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            let amt_2 = 80;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_2,
                InvoiceType::Blinded(Some(utxo_wlt_2_2)),
            );
            let (_, tx_2) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            vec![tx_0, tx_1, tx_2]
        }
        HistoryType::Branching => {
            let amt_0 = 600;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_0,
                InvoiceType::Blinded(Some(utxo_wlt_2_1)),
            );
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, Some(1000), None, None);

            let amt_1 = 200;
            let invoice = wlt_1.invoice(
                contract_id,
                schema_id,
                amt_1,
                InvoiceType::Blinded(Some(utxo_wlt_1_1)),
            );
            let (_, tx_1) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            let amt_2 = amt_0 - amt_1 - 1;
            let invoice = wlt_1.invoice(
                contract_id,
                schema_id,
                amt_2,
                InvoiceType::Blinded(Some(utxo_wlt_1_2)),
            );
            let (_, tx_2) = wlt_2.send_to_invoice(&mut wlt_1, invoice, Some(1000), None, None);

            vec![tx_0, tx_1, tx_2]
        }
        HistoryType::Merging => {
            let amt_0 = 400;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_0,
                InvoiceType::Blinded(Some(utxo_wlt_2_1)),
            );
            let (_, tx_0) = wlt_1.send_to_invoice(&mut wlt_2, invoice, None, None, None);

            let amt_1 = 200;
            let invoice = wlt_2.invoice(
                contract_id,
                schema_id,
                amt_1,
                InvoiceType::Blinded(Some(utxo_wlt_2_2)),
            );
            let (_, tx_1) = wlt_1.send_to_invoice(&mut wlt_2, invoice, None, None, None);

            let amt_2 = amt_0 + amt_1 - 1;
            let invoice = wlt_1.invoice(
                contract_id,
                schema_id,
                amt_2,
                InvoiceType::Blinded(Some(utxo_wlt_1_1)),
            );
            // sender checks if it's safe to merge allocations
            let height_pre_transfer = get_height_custom(INSTANCE_2);
            let allocations = wlt_2.contract_fungible_allocations(contract_id, false);
            let utxos: Vec<(Outpoint, Txid)> = allocations
                .iter()
                .map(|a| (a.seal.to_outpoint(), a.witness.unwrap()))
                .collect();
            let safe_height = height_pre_transfer - 6; // min 6 confirmations
            for (utxo, txid) in utxos {
                let height = if txid == tx_0.txid() {
                    height_pre_transfer - 1
                } else {
                    height_pre_transfer
                };
                let assets_history =
                    wlt_2.get_outpoint_unsafe_history(utxo, NonZeroU32::new(safe_height).unwrap());
                let expected_history = HashMap::from([(
                    contract_id,
                    HashMap::from([(height, HashSet::from([txid]))]),
                )]);
                assert_eq!(assets_history, expected_history);
            }
            // sender proceeds with the tranfer even if there's unsafe history
            let (consignment, tx_2, _, _) = wlt_2.pay_full(invoice, None, None, true, None);
            wlt_2.mine_tx(&tx_2.txid(), false);
            // receiver checks if it's safe to receive allocations
            let safe_height = height_pre_transfer; // min 1 confirmation
            let validated_consignment = consignment
                .clone()
                .validate(
                    &wlt_1.get_resolver(),
                    wlt_1.chain_net(),
                    Some(NonZeroU32::new(safe_height).unwrap()),
                )
                .map_err(|(status, _)| status)
                .unwrap();
            let validation_status = validated_consignment.clone().into_validation_status();
            assert_eq!(validation_status.warnings.len(), 1);
            let unsafe_height = height_pre_transfer + 1;
            let unsafe_history_map = HashMap::from([(unsafe_height, HashSet::from([tx_2.txid()]))]);
            assert!(
                matches!(&validation_status.warnings[0], Warning::UnsafeHistory(map) if *map == unsafe_history_map)
            );
            // receiver decides to accept the consignment even if there's unsafe history
            wlt_1.accept_transfer(consignment.clone(), None);
            wlt_2.sync();

            vec![tx_0, tx_1, tx_2]
        }
    };

    broadcast_tx_and_mine(&txs[1], INSTANCE_3);
    wlt_1.switch_to_instance(INSTANCE_3);
    wlt_2.switch_to_instance(INSTANCE_3);
    let (wlt_1_allocs, wlt_2_allocs) = match history_type {
        HistoryType::Linear | HistoryType::Branching => (vec![600], vec![]),
        HistoryType::Merging => (vec![400], vec![200]),
    };
    wlt_1.check_allocations(contract_id, schema_id, wlt_1_allocs, false);
    wlt_2.check_allocations(contract_id, schema_id, wlt_2_allocs, false);
    broadcast_tx_and_mine(&txs[2], INSTANCE_3);
    wlt_1.sync_and_update_witnesses(None);
    wlt_2.sync_and_update_witnesses(None);
    let (wlt_1_allocs, wlt_2_allocs) = match history_type {
        HistoryType::Linear | HistoryType::Branching => (vec![600], vec![]),
        HistoryType::Merging => (vec![400], vec![]), // funds are burnt
    };
    wlt_1.check_allocations(contract_id, schema_id, wlt_1_allocs, false);
    wlt_2.check_allocations(contract_id, schema_id, wlt_2_allocs, false);
    broadcast_tx_and_mine(&txs[0], INSTANCE_3);
    wlt_1.sync_and_update_witnesses(None);
    wlt_2.sync_and_update_witnesses(None);
    let (wlt_1_allocs, wlt_2_allocs) = match history_type {
        HistoryType::Linear => (vec![10, 20], vec![490, 80]),
        HistoryType::Branching => (vec![200, 399], vec![1]),
        HistoryType::Merging => (vec![599], vec![1]),
    };
    wlt_1.check_allocations(contract_id, schema_id, wlt_1_allocs, false);
    wlt_2.check_allocations(contract_id, schema_id, wlt_2_allocs, false);
}

#[cfg(not(feature = "altered"))]
#[rstest]
#[case(false)]
#[case(true)]
#[serial]
fn revert_genesis(#[case] with_transfers: bool) {
    println!("with_transfers {with_transfers}");

    initialize();
    // connecting before disconnecting since disconnect is not idempotent
    connect_reorg_nodes();
    disconnect_reorg_nodes();

    let mut wlt = get_wallet_custom(&DescriptorType::Wpkh, Some(INSTANCE_2), true);

    let issued_supply = 600;
    let utxo = wlt.get_utxo(None);
    let contract_id = wlt.issue_nia(issued_supply, Some(&utxo));
    let schema_id = wlt.schema_id(contract_id);

    wlt.check_allocations(contract_id, schema_id, vec![issued_supply], false);

    if with_transfers {
        let mut recv_wlt = get_wallet_custom(&DescriptorType::Wpkh, Some(INSTANCE_2), true);
        let amt = 200;
        wlt.send(
            &mut recv_wlt,
            TransferType::Blinded,
            contract_id,
            amt,
            1000,
            None,
        );
        wlt.check_allocations(contract_id, schema_id, vec![issued_supply - amt], false);
    }

    assert!(matches!(
        wlt.get_witness_ord(&utxo.txid),
        WitnessOrd::Mined(_)
    ));
    wlt.switch_to_instance(INSTANCE_3);
    assert_eq!(wlt.get_witness_ord(&utxo.txid), WitnessOrd::Archived);

    wlt.sync();
    let utxos = wlt.utxos();
    assert!(utxos.is_empty());

    wlt.check_allocations(contract_id, schema_id, vec![], false);
}
