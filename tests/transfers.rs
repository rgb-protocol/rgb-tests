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

    match (wlt_1_desc, wlt_2_desc) {
        (DescriptorType::Wpkh, DescriptorType::Wpkh) => {
            let wlt_1 = BdkTestWallet::with_descriptor(&wlt_1_desc);
            let wlt_2 = BdkTestWallet::with_descriptor(&wlt_2_desc);
            transfer_loop_impl(wlt_1, wlt_2, transfer_type, asset_schema_1, asset_schema_2);
        }
        (DescriptorType::Wpkh, DescriptorType::Tr) => {
            let wlt_1 = BdkTestWallet::with_descriptor(&wlt_1_desc);
            let wlt_2 = BpTestWallet::with_descriptor(&wlt_2_desc);
            transfer_loop_impl(wlt_1, wlt_2, transfer_type, asset_schema_1, asset_schema_2);
        }
        (DescriptorType::Tr, DescriptorType::Wpkh) => {
            let wlt_1 = BpTestWallet::with_descriptor(&wlt_1_desc);
            let wlt_2 = BdkTestWallet::with_descriptor(&wlt_2_desc);
            transfer_loop_impl(wlt_1, wlt_2, transfer_type, asset_schema_1, asset_schema_2);
        }
        (DescriptorType::Tr, DescriptorType::Tr) => {
            let wlt_1 = BpTestWallet::with_descriptor(&wlt_1_desc);
            let wlt_2 = BpTestWallet::with_descriptor(&wlt_2_desc);
            transfer_loop_impl(wlt_1, wlt_2, transfer_type, asset_schema_1, asset_schema_2);
        }
    }
}

fn transfer_loop_impl<W1, D1, W2, D2>(
    mut wlt_1: TestWallet<W1, D1>,
    mut wlt_2: TestWallet<W2, D2>,
    transfer_type: TransferType,
    asset_schema_1: AssetSchema,
    asset_schema_2: AssetSchema,
) where
    W1: WalletProvider,
    W2: WalletProvider,
    TestWallet<W1, D1>: TestWalletExt,
    <TestWallet<W1, D1> as TestWalletExt>::Psbt: Serialize,
    TestWallet<W2, D2>: TestWalletExt,
    <TestWallet<W2, D2> as TestWalletExt>::Psbt: Serialize,
{
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
#[case(AS::Ifa)]
fn unknown_kit(#[case] asset_schema: AssetSchema) {
    println!("asset_schema {asset_schema:?}");

    initialize();

    let mut wlt_1 = BpTestWallet::with(&DescriptorType::Wpkh, None, false);
    let mut wlt_2 = BpTestWallet::with(&DescriptorType::Wpkh, None, false);

    let (contract_id, secret_key) = match asset_schema {
        AssetSchema::Nia => (wlt_1.issue_nia(600, None), None),
        AssetSchema::Uda => (wlt_1.issue_uda(None), None),
        AssetSchema::Cfa => (wlt_1.issue_cfa(600, None), None),
        AssetSchema::Pfa => {
            let (secret_key, public_key) =
                Secp256k1::new().generate_keypair(&mut rand::thread_rng());
            let pubkey = CompressedPublicKey::from_slice(&public_key.serialize()).unwrap();
            (wlt_1.issue_pfa(600, None, pubkey), Some(secret_key))
        }
        AssetSchema::Ifa => (wlt_1.issue_ifa(600, None, vec![]), None),
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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

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

    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), true);
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
#[should_panic(expected = "InvalidConsignment")]
#[case(TransferType::Blinded)]
#[should_panic(expected = "Composition(InsufficientState)")]
#[case(TransferType::Witness)]
fn same_transfer_twice_no_update_witnesses(#[case] transfer_type: TransferType) {
    println!("transfer_type {transfer_type:?}");

    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issue_supply = 2000;
    let contract_id = wlt_1.issue_nia(issue_supply, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let amount = 100;
    let invoice = wlt_2.invoice(contract_id, schema_id, amount, transfer_type);
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

    let mut wlt_3 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issue_supply = 2000;
    let contract_id = wlt_1.issue_nia(issue_supply, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let amount = 100;
    let invoice = wlt_2.invoice(contract_id, schema_id, amount, transfer_type);
    let _ = wlt_1.pay_full(invoice.clone(), None, Some(500), false, None);

    wlt_1.sync_and_update_witnesses(None);

    // with TransferType::Blinded this fails with an AbsentValidWitness error
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, Some(1000), true, None);

    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let asset_info = AssetInfo::default_nia(vec![500, 400]);
    let contract_id = wlt_1.issue_with_info(asset_info, vec![None, None], None, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let amount = 300;
    let invoice = wlt_2.invoice(contract_id, schema_id, amount, transfer_type);
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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issue_supply = 600;
    let contract_id = wlt_1.issue_nia(issue_supply, None);
    let schema_id = wlt_1.schema_id(contract_id);

    let amt = 200;
    let invoice = wlt_2.invoice(contract_id, schema_id, amt, InvoiceType::Witness);
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice.clone(), None, None, true, None);
    let txid = txid_bp_to_bitcoin(tx.txid());

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
    assert!(
        allocations
            .iter()
            .any(|fa| fa.state == Amount::from(wlt_1_change_amt))
    );

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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let pre_funding_height = get_height();

    let utxo_1 = wlt_1.get_utxo(Some(10_000));
    let utxo_2 = wlt_1.get_utxo(Some(20_000));
    let amounts = vec![600, 300];
    let outpoints = vec![Some(utxo_1), Some(utxo_2)];
    let asset_info = AssetInfo::default_nia(amounts.clone());
    let contract_id = wlt_1.issue_with_info(asset_info, outpoints, None, None);

    struct LNFasciaResolver {}
    impl WitnessOrdProvider for LNFasciaResolver {
        fn witness_ord(&self, _: Txid) -> Result<WitnessOrd, WitnessResolverError> {
            Ok(WitnessOrd::Ignored)
        }
    }

    println!("\n1. fake commitment TX (no HTLCs)");
    let witness_info_0 = wlt_2.get_witness_info(Some(2000), None);
    let witness_info_1 = wlt_1.get_witness_info(None, None);
    let beneficiaries = vec![
        (witness_info_0.address(), witness_info_0.amount_sats),
        (witness_info_1.address(), witness_info_1.amount_sats),
    ];
    let (mut psbt, mut meta) = wlt_1.construct_psbt(vec![utxo_1], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                assignments: vec![
                    AssetAssignment {
                        destination: AssetDestination::Witness(witness_info_0.clone()),
                        amount: 100,
                    },
                    AssetAssignment {
                        destination: AssetDestination::Witness(witness_info_1.clone()),
                        amount: 500,
                    },
                ],
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
        close_method: CloseMethod::OpretFirst,
    };
    let (fascia, _asset_beneficiaries, _, _) =
        wlt_1.color_psbt(&mut psbt, &mut meta, coloring_info.clone(), None);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);
    let txid_same_bundle_1 = psbt.txid();
    let witness_info_0_same_bundle = witness_info_0;
    let witness_info_1_same_bundle = witness_info_1;
    let coloring_info_same_bundle = coloring_info;

    let htlc_vout = 2;
    let htlc_rgb_amt = 200;
    let htlc_btc_amt = 4000;
    let htlc_witness_info = wlt_1.get_witness_info(Some(htlc_btc_amt), None);

    // no problem: since there's no htlc for this commitment
    wlt_1.sync_and_update_witnesses(Some(pre_funding_height));

    println!("\n2. fake commitment TX (1 HTLC)");
    let witness_info_0 = wlt_2.get_witness_info(Some(2000), None);
    let witness_info_1 = wlt_1.get_witness_info(None, None);
    let beneficiaries = vec![
        (witness_info_0.address(), witness_info_0.amount_sats),
        (witness_info_1.address(), witness_info_1.amount_sats),
        (htlc_witness_info.address(), htlc_witness_info.amount_sats),
    ];
    let (mut psbt, mut meta) = wlt_1.construct_psbt(vec![utxo_1], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                assignments: vec![
                    AssetAssignment {
                        destination: AssetDestination::Witness(witness_info_0),
                        amount: 100,
                    },
                    AssetAssignment {
                        destination: AssetDestination::Witness(witness_info_1),
                        amount: 300,
                    },
                    AssetAssignment {
                        destination: AssetDestination::Witness(htlc_witness_info.clone()),
                        amount: htlc_rgb_amt,
                    },
                ],
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
        close_method: CloseMethod::OpretFirst,
    };
    let (fascia, _asset_beneficiaries, _, _) =
        wlt_1.color_psbt(&mut psbt, &mut meta, coloring_info, None);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    if update_witnesses_before_htlc {
        wlt_1.sync_and_update_witnesses(Some(pre_funding_height));
    }

    println!("\n3. fake HTLC TX");
    let txid = fascia.witness_id();
    let input_outpoint = Outpoint::new(txid, htlc_vout);
    let witness_info_0 = wlt_1.get_witness_info(None, None);
    let beneficiaries = vec![(witness_info_0.address(), witness_info_0.amount_sats)];
    let (mut psbt, mut meta) = wlt_1.construct_psbt_offchain(
        vec![(
            outpoint_bitcoin_to_bp(input_outpoint),
            htlc_btc_amt,
            htlc_witness_info.terminal(),
            htlc_witness_info.script_pubkey(),
        )],
        beneficiaries,
        None,
    );
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![input_outpoint],
                assignments: vec![AssetAssignment {
                    destination: AssetDestination::Witness(witness_info_0),
                    amount: htlc_rgb_amt,
                }],
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX),
        close_method: CloseMethod::OpretFirst,
    };
    let (fascia, _asset_beneficiaries, _, _) =
        wlt_1.color_psbt(&mut psbt, &mut meta, coloring_info, None);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    println!("\n4. fake commitment TX (no HTLCs)");
    let beneficiaries = vec![
        (witness_info_0_same_bundle.address(), Some(3001)),
        (witness_info_1_same_bundle.address(), None),
    ];
    let (mut psbt, mut meta) = wlt_1.construct_psbt(vec![utxo_1], beneficiaries, None);
    let coloring_info = coloring_info_same_bundle;
    let (mut fascia, _asset_beneficiaries, _, _) =
        wlt_1.color_psbt(&mut psbt, &mut meta, coloring_info, None);
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
    fascia.seal_witness.public = PubWitness::with(psbt.unsigned_tx());
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    let mut old_psbt = psbt.clone();

    println!("\n5. fake commitment TX (1 HTLC)");
    let htlc_rgb_amt = 180;
    let witness_info_0 = wlt_2.get_witness_info(Some(2000), None);
    let witness_info_1 = wlt_1.get_witness_info(None, None);
    let beneficiaries = vec![
        (witness_info_0.address(), witness_info_0.amount_sats),
        (witness_info_1.address(), witness_info_1.amount_sats),
        (htlc_witness_info.address(), htlc_witness_info.amount_sats),
    ];
    let (mut psbt, mut meta) = wlt_1.construct_psbt(vec![utxo_1], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                assignments: vec![
                    AssetAssignment {
                        destination: AssetDestination::Witness(witness_info_0),
                        amount: 122,
                    },
                    AssetAssignment {
                        destination: AssetDestination::Witness(witness_info_1),
                        amount: 298,
                    },
                    AssetAssignment {
                        destination: AssetDestination::Witness(htlc_witness_info.clone()),
                        amount: htlc_rgb_amt,
                    },
                ],
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
        close_method: CloseMethod::OpretFirst,
    };
    let (fascia, _asset_beneficiaries, _, _) =
        wlt_1.color_psbt(&mut psbt, &mut meta, coloring_info.clone(), None);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    if update_witnesses_before_htlc {
        wlt_1.sync_and_update_witnesses(Some(pre_funding_height));
    }

    println!("\n6. fake HTLC TX");
    let txid = fascia.witness_id();
    let input_outpoint = Outpoint::new(txid, htlc_vout);
    let witness_info_0 = wlt_1.get_witness_info(None, None);
    let beneficiaries = vec![(witness_info_0.address(), witness_info_0.amount_sats)];
    let (mut psbt, mut meta) = wlt_1.construct_psbt_offchain(
        vec![(
            outpoint_bitcoin_to_bp(input_outpoint),
            htlc_btc_amt,
            htlc_witness_info.terminal(),
            htlc_witness_info.script_pubkey(),
        )],
        beneficiaries,
        None,
    );
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![input_outpoint],
                assignments: vec![AssetAssignment {
                    destination: AssetDestination::Witness(witness_info_0),
                    amount: htlc_rgb_amt,
                }],
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX),
        close_method: CloseMethod::OpretFirst,
    };
    let (fascia, _asset_beneficiaries, _, _) =
        wlt_1.color_psbt(&mut psbt, &mut meta, coloring_info, None);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    // no problem: since the force-close tx will be updated to mined soon
    wlt_1.sync_and_update_witnesses(Some(pre_funding_height));

    println!("\n7. fake commitment TX (1 HTLC) on 2nd channel");
    let htlc_rgb_amt_2nd_chan = 10;
    let witness_info_0 = wlt_2.get_witness_info(Some(2000), None);
    let witness_info_1 = wlt_1.get_witness_info(None, None);
    let beneficiaries = vec![
        (witness_info_0.address(), witness_info_0.amount_sats),
        (witness_info_1.address(), witness_info_1.amount_sats),
        (htlc_witness_info.address(), htlc_witness_info.amount_sats),
    ];
    let (mut psbt, mut meta) = wlt_1.construct_psbt(vec![utxo_2], beneficiaries, None);
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_2],
                assignments: vec![
                    AssetAssignment {
                        destination: AssetDestination::Witness(witness_info_0),
                        amount: 20,
                    },
                    AssetAssignment {
                        destination: AssetDestination::Witness(witness_info_1),
                        amount: 270,
                    },
                    AssetAssignment {
                        destination: AssetDestination::Witness(htlc_witness_info.clone()),
                        amount: htlc_rgb_amt_2nd_chan,
                    },
                ],
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX - 1),
        close_method: CloseMethod::OpretFirst,
    };
    let (fascia, _asset_beneficiaries, _, _) =
        wlt_1.color_psbt(&mut psbt, &mut meta, coloring_info, None);
    wlt_1.consume_fascia_custom_resolver(fascia.clone(), LNFasciaResolver {});
    wlt_1.debug_logs(contract_id, AllocationFilter::WalletAll);

    println!("\n8. broadcast old PSBT");
    let tx = wlt_1.sign_finalize_extract(&mut old_psbt);
    wlt_1.broadcast_tx(&tx);
    let txid = txid_bp_to_bitcoin(tx.txid());
    wlt_1.mine_tx(&txid, false);
    wlt_1.sync();
    wlt_1.update_witnesses(pre_funding_height, vec![txid]);
    let mut wlt_3 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
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
    let witness_info_0 = wlt_1.get_witness_info(None, None);
    let beneficiaries = vec![(witness_info_0.address(), witness_info_0.amount_sats)];
    let (mut psbt, mut meta) = wlt_1.construct_psbt_offchain(
        vec![(
            outpoint_bitcoin_to_bp(input_outpoint),
            htlc_btc_amt,
            htlc_witness_info.terminal(),
            htlc_witness_info.script_pubkey(),
        )],
        beneficiaries,
        None,
    );
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![input_outpoint],
                assignments: vec![AssetAssignment {
                    destination: AssetDestination::Witness(witness_info_0),
                    amount: htlc_rgb_amt_2nd_chan,
                }],
            },
        )]),
        static_blinding: Some(666),
        nonce: Some(u64::MAX),
        close_method: CloseMethod::OpretFirst,
    };
    let (fascia, _asset_beneficiaries, _, _) =
        wlt_1.color_psbt(&mut psbt, &mut meta, coloring_info, None);
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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::new_mainnet();

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

    let mut wlt_1 = BpTestWallet::new_mainnet();

    // sometimes this fails with a 'Too many requests' error when using esplora
    wlt_1.sync();
}

#[cfg(not(feature = "altered"))]
#[test]
fn collaborative_transfer() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_3 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let sats = 30_000;

    let utxo_0 = wlt_1.get_utxo(Some(sats));
    let contract_id = wlt_1.issue_nia(600, Some(&utxo_0));
    let schema_id = wlt_1.schema_id(contract_id);
    let (_, tx) = wlt_1.send(
        &mut wlt_2,
        TransferType::Witness,
        contract_id,
        200,
        18_000,
        None,
    );
    let utxo_1 = Outpoint::new(txid_bp_to_bitcoin(tx.txid()), 2); // change: 11_600 sat
    let utxo_2 = Outpoint::new(txid_bp_to_bitcoin(tx.txid()), 1); // transfered: 18_000 sat

    let mut psbt = BpPsbt::default();

    wlt_1.psbt_add_input(&mut psbt, outpoint_bitcoin_to_bp(utxo_1));
    wlt_2.psbt_add_input(&mut psbt, outpoint_bitcoin_to_bp(utxo_2));

    let witness_info = wlt_3.get_witness_info(Some(sats - 2 * DEFAULT_FEE_ABS), None);
    psbt.construct_output_expect(
        witness_info.script_pubkey(),
        Sats::from_sats(witness_info.amount_sats.unwrap()),
    );

    let coloring_info_1 = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_1],
                assignments: vec![AssetAssignment {
                    destination: AssetDestination::Witness(witness_info.clone()),
                    amount: 400,
                }],
            },
        )]),
        static_blinding: None,
        nonce: None,
        close_method: CloseMethod::OpretFirst,
    };
    let coloring_info_2 = ColoringInfo {
        asset_info_map: HashMap::from([(
            contract_id,
            AssetColoringInfo {
                input_outpoints: vec![utxo_2],
                assignments: vec![AssetAssignment {
                    destination: AssetDestination::Witness(witness_info),
                    amount: 200,
                }],
            },
        )]),
        static_blinding: None,
        nonce: None,
        close_method: CloseMethod::OpretFirst,
    };
    let mut meta = BpPsbtMeta {
        change_vout: None,
        change_terminal: None,
    };
    wlt_1.color_psbt_init(&mut psbt, &mut meta, coloring_info_1, None);
    let (fascia, beneficiaries, _, _) =
        wlt_2.color_psbt(&mut psbt, &mut meta, coloring_info_2, None);
    wlt_1.sign_finalize(&mut psbt);
    let tx = wlt_2.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);

    wlt_1.consume_fascia(fascia.clone(), tx.txid());
    wlt_2.consume_fascia(fascia, tx.txid());

    let consignments = wlt_1.create_consignments(beneficiaries.clone(), tx.txid());
    assert_eq!(
        consignments,
        wlt_2.create_consignments(beneficiaries, tx.txid())
    );

    for consignment in consignments.into_values() {
        wlt_3.accept_transfer(consignment, None);
    }
    wlt_3.check_allocations(contract_id, schema_id, vec![200, 400], true);
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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_3 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

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
        consignment: &'cons Consignment<TRANSFER>,
        fallback: &'a AnyResolver,
    }
    impl<const TRANSFER: bool> ResolveWitness for OffchainResolver<'_, '_, TRANSFER> {
        fn resolve_witness(&self, witness_id: Txid) -> Result<WitnessStatus, WitnessResolverError> {
            if witness_id != self.witness_id {
                return self.fallback.resolve_witness(witness_id);
            }
            self.consignment
                .bundled_witnesses()
                .find(|bw| bw.witness_id() == witness_id)
                .and_then(|p| p.pub_witness.tx().cloned())
                .map_or_else(
                    || self.fallback.resolve_witness(witness_id),
                    |tx| Ok(WitnessStatus::Resolved(tx, WitnessOrd::Tentative)),
                )
        }
        fn check_chain_net(&self, _: ChainNet) -> Result<(), WitnessResolverError> {
            Ok(())
        }
    }

    let resolver = OffchainResolver {
        witness_id: txid_bp_to_bitcoin(witness_id),
        consignment: &consignment,
        fallback: &wlt_2.get_resolver(),
    };

    // wlt_2 use custom resolver to be able to send the assets even if transfer TX sending to
    // blinded UTXO has not been broadcasted
    wlt_2.accept_transfer_custom(consignment.clone(), None, &resolver);

    let invoice = wlt_3.invoice(contract_id, schema_id, 50, InvoiceType::Witness);
    let (consignment, tx, _, _) = wlt_2.pay_full(invoice, Some(2000), None, true, None);
    wlt_2.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);

    // consignment validation fails because it notices an unbroadcasted TX in the history
    let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
    let validation_config = ValidationConfig {
        chain_net: wlt_3.chain_net(),
        trusted_typesystem,
        ..Default::default()
    };
    let res = consignment
        .validate(&wlt_3.get_resolver(), &validation_config)
        .unwrap_err();
    assert!(matches!(
        res,
        ValidationError::InvalidConsignment(Failure::SealNoPubWitness(_, _))
    ));
}

#[cfg(not(feature = "altered"))]
#[test]
fn check_fungible_history() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

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

    let mut wlt = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issue_supply = 600;

    let contract_id = wlt.issue_nia(issue_supply, None);
    let schema_id = wlt.schema_id(contract_id);

    let amt = 200;

    let invoice = wlt.invoice(contract_id, schema_id, amt, InvoiceType::Witness);

    let (consignment, tx, _, _) = wlt.pay_full(invoice.clone(), None, None, true, None);
    wlt.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Tr);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_3 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

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

    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id_1,
        250,
        1000,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn tapret_commitments_on_beneficiary_output() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Tr);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Tr);

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
    let mut beneficiary_script_1 = None;
    let mut beneficiary_address_1 = None;
    if let Beneficiary::WitnessVout(pay2vout, _) = invoice_1.beneficiary.into_inner() {
        beneficiary_script_1 = Some(script_buf_to_script_pubkey(pay2vout.to_script()));
        beneficiary_address_1 = Some(pay2vout.into_address(wlt_2.network()));
    }
    if tx.outputs().last().unwrap().script_pubkey != beneficiary_script_1.unwrap() {
        wlt_2.try_add_tapret_tweak(consignment.clone(), &txid_bp_to_bitcoin(tx.txid()));
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
    let mut beneficiary_script = None;
    if let Beneficiary::WitnessVout(pay2vout, _) = invoice_2.beneficiary.into_inner() {
        beneficiary_script = Some(script_buf_to_script_pubkey(pay2vout.to_script()));
    }
    assert_eq!(
        tx.outputs().last().unwrap().script_pubkey,
        beneficiary_script.unwrap()
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
    let mut beneficiary_script_2 = None;
    if let Beneficiary::WitnessVout(pay2vout, _) = invoice_1.beneficiary.into_inner() {
        beneficiary_script_2 = Some(script_buf_to_script_pubkey(pay2vout.to_script()));
    }
    if tx.outputs().last().unwrap().script_pubkey != beneficiary_script_2.unwrap() {
        wlt_2.try_add_tapret_tweak(consignment.clone(), &txid_bp_to_bitcoin(tx.txid()));
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

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let (secret_key, public_key) = Secp256k1::new().generate_keypair(&mut rand::thread_rng());
    let pubkey = CompressedPublicKey::from_slice(&public_key.serialize()).unwrap();

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
#[test]
fn ifa_inflation() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_3 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_supply = 999;
    let inflation_supply = 555;
    let inflation_outpoint = wlt_1.get_utxo(None);
    let contract_id = wlt_1.issue_ifa(
        issued_supply,
        None,
        vec![(inflation_outpoint, inflation_supply)],
    );

    wlt_1.send_ifa(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id,
        issued_supply,
    );

    // first inflation
    let inflation_1_amt_1 = 300;
    let inflation_1_amt_2 = 200;
    wlt_1.inflate_ifa(
        contract_id,
        vec![inflation_outpoint],
        vec![inflation_1_amt_1, inflation_1_amt_2],
    );

    // send inflated asset
    wlt_1.check_allocations(
        contract_id,
        AssetSchema::Ifa,
        vec![inflation_1_amt_1, inflation_1_amt_2],
        false,
    );
    wlt_1.send_ifa(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id,
        inflation_1_amt_1 + inflation_1_amt_2,
    );
    wlt_2.check_allocations(
        contract_id,
        AssetSchema::Ifa,
        vec![issued_supply, inflation_1_amt_1 + inflation_1_amt_2],
        false,
    );

    // second inflation
    let contract = wlt_1.contract_wrapper::<InflatableFungibleAsset>(contract_id);
    let inflation_allocations = contract
        .inflation_allocations(AllocationFilter::Wallet.filter_for(&wlt_1))
        .collect::<Vec<_>>();
    let inflation_outpoints = inflation_allocations
        .iter()
        .map(|oa| oa.seal.outpoint().unwrap())
        .collect::<Vec<_>>();
    let inflation_2_amt: u64 = inflation_allocations
        .iter()
        .map(|oa| oa.state.value())
        .sum();
    wlt_1.inflate_ifa(contract_id, inflation_outpoints, vec![inflation_2_amt]);

    // send inflated asset
    let total_circulating = issued_supply + inflation_1_amt_1 + inflation_1_amt_2 + inflation_2_amt;
    wlt_1.check_allocations(contract_id, AssetSchema::Ifa, vec![inflation_2_amt], false);
    wlt_1.send_ifa(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id,
        inflation_2_amt,
    );
    wlt_2.check_allocations(
        contract_id,
        AssetSchema::Ifa,
        vec![
            issued_supply,
            inflation_1_amt_1 + inflation_1_amt_2,
            inflation_2_amt,
        ],
        false,
    );
    wlt_2.send_ifa(
        &mut wlt_3,
        TransferType::Blinded,
        contract_id,
        total_circulating,
    );
    wlt_1.check_allocations(contract_id, AssetSchema::Ifa, vec![], false);
    wlt_2.check_allocations(contract_id, AssetSchema::Ifa, vec![], false);
    wlt_3.check_allocations(
        contract_id,
        AssetSchema::Ifa,
        vec![total_circulating],
        false,
    );

    // check max supply has been reached, no more inflation allowed
    let contract = wlt_1.contract_wrapper::<InflatableFungibleAsset>(contract_id);
    let max_supply = contract.max_supply().value();
    let total_issued_supply = contract.total_issued_supply().value();
    assert_eq!(max_supply, total_issued_supply);
    assert_eq!(max_supply, total_circulating);
    let inflation_allocations = contract
        .inflation_allocations(AllocationFilter::Wallet.filter_for(&wlt_1))
        .collect::<Vec<_>>();
    let inflatable: u64 = inflation_allocations
        .iter()
        .map(|oa| oa.state.value())
        .sum();
    assert_eq!(inflatable, 0);
}

#[cfg(not(feature = "altered"))]
#[test]
fn ifa_zero_issuance_with_inflation() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    // issue zero assets
    let issued_supply = 0;
    let inflation_supply = 1000;
    let inflation_outpoint = wlt_1.get_utxo(None);
    let contract_id = wlt_1.issue_ifa(
        issued_supply,
        None,
        vec![(inflation_outpoint, inflation_supply)],
    );
    wlt_1.check_allocations(contract_id, AssetSchema::Ifa, vec![], false);

    // inflate the asset
    wlt_1.inflate_ifa(
        contract_id,
        vec![inflation_outpoint],
        vec![inflation_supply],
    );
    wlt_1.check_allocations(contract_id, AssetSchema::Ifa, vec![inflation_supply], false);

    // send inflated asset
    wlt_1.send_ifa(
        &mut wlt_2,
        TransferType::Blinded,
        contract_id,
        inflation_supply,
    );
    wlt_2.check_allocations(contract_id, AssetSchema::Ifa, vec![inflation_supply], false);
    wlt_1.check_allocations(contract_id, AssetSchema::Ifa, vec![], false);
}

#[cfg(not(feature = "altered"))]
#[test]
fn ifa_move_inflation_right() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_supply = 999;
    let inflation_supply = 555;
    let inflation_outpoint = wlt_1.get_utxo(None);
    let contract_id = wlt_1.issue_ifa(
        issued_supply,
        None,
        vec![(inflation_outpoint, inflation_supply)],
    );
    let schema_id = wlt_1.schema_id(contract_id);

    // partially move inflation right from wlt_1 to wlt_2
    let inflation_moved = 55;
    let inflation_moved_utxo = wlt_2.get_utxo(None);
    let mut invoice = wlt_2.invoice(
        contract_id,
        schema_id,
        inflation_moved,
        InvoiceType::Blinded(Some(inflation_moved_utxo)),
    );
    invoice.assignment_name = Some(FieldName::from_str("inflationAllowance").unwrap());
    wlt_1.send_ifa_to_invoice(&mut wlt_2, invoice);
    let contract = wlt_2.contract_wrapper::<InflatableFungibleAsset>(contract_id);
    let inflation_allocations = contract
        .inflation_allocations(AllocationFilter::Wallet.filter_for(&wlt_2))
        .collect::<Vec<_>>();
    let inflation_outpoints = inflation_allocations
        .iter()
        .map(|oa| oa.seal.outpoint().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(inflation_outpoints.len(), 1);
    assert_eq!(inflation_outpoints[0], inflation_moved_utxo);
    assert_eq!(
        inflation_allocations
            .iter()
            .map(|oa| oa.state.value())
            .sum::<u64>(),
        inflation_moved
    );

    // inflate asset with wlt_2
    wlt_2.inflate_ifa(
        contract_id,
        vec![inflation_moved_utxo],
        vec![inflation_moved],
    );
    let contract = wlt_2.contract_wrapper::<InflatableFungibleAsset>(contract_id);
    assert_eq!(
        contract.total_issued_supply().value(),
        issued_supply + inflation_moved
    );

    // inflate asset with wlt_1
    let contract = wlt_1.contract_wrapper::<InflatableFungibleAsset>(contract_id);
    let inflation_change_utxo = contract
        .inflation_allocations(AllocationFilter::Wallet.filter_for(&wlt_1))
        .map(|oa| oa.seal.outpoint().unwrap())
        .collect::<Vec<_>>()[0];
    let inflation_change = inflation_supply - inflation_moved;
    wlt_1.inflate_ifa(
        contract_id,
        vec![inflation_change_utxo],
        vec![inflation_change],
    );
    let contract = wlt_1.contract_wrapper::<InflatableFungibleAsset>(contract_id);
    let wlt_1_amt = issued_supply + inflation_change;
    assert_eq!(contract.total_issued_supply().value(), wlt_1_amt);

    // wlt_1 sends all to wlt_2
    wlt_1.send_ifa(&mut wlt_2, TransferType::Blinded, contract_id, wlt_1_amt);
    wlt_1.check_allocations(contract_id, AssetSchema::Ifa, vec![], false);
    wlt_2.check_allocations(
        contract_id,
        AssetSchema::Ifa,
        vec![wlt_1_amt, inflation_moved],
        false,
    );

    // check max supply has been reached, no more inflation allowed
    let contract = wlt_2.contract_wrapper::<InflatableFungibleAsset>(contract_id);
    let max_supply = contract.max_supply().value();
    let total_issued_supply = contract.total_issued_supply().value();
    assert_eq!(max_supply, total_issued_supply);
    assert_eq!(max_supply, wlt_1_amt + inflation_moved);
    let inflation_allocations = contract
        .inflation_allocations(AllocationFilter::Wallet.filter_for(&wlt_2))
        .collect::<Vec<_>>();
    assert_eq!(
        inflation_allocations
            .iter()
            .map(|oa| oa.state.value())
            .sum::<u64>(),
        0
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn ifa_burn() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let contract_id = wlt_1.issue_ifa(999, None, vec![]);

    let amt = 300;
    let utxo = wlt_2.get_utxo(None);
    wlt_1.send_ifa(
        &mut wlt_2,
        InvoiceType::Blinded(Some(utxo)),
        contract_id,
        amt,
    );

    // burn assets
    wlt_2.check_allocations(contract_id, AssetSchema::Ifa, vec![amt], false);
    let (consignment, _) = wlt_2.burn_ifa(contract_id, vec![utxo]);
    let last_transition = consignment
        .bundles
        .iter()
        .last()
        .unwrap()
        .bundle
        .known_transitions
        .iter()
        .last()
        .unwrap()
        .transition
        .clone();
    assert!(last_transition.transition_type == TS_BURN);
    wlt_2.check_allocations(contract_id, AssetSchema::Ifa, vec![], false);
}

#[cfg(not(feature = "altered"))]
#[should_panic(expected = "InputMapTransitionMismatch")]
#[test]
fn extra_known_transition() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_amt = 900;
    let contract_id = wlt_1.issue_nia(issued_amt, None);
    let asset_schema = wlt_1.asset_schema(contract_id);
    let schema_id = wlt_1.schema_id(contract_id);
    let contract = wlt_1.stock().contract_data(contract_id).unwrap();
    let assignment_type = contract
        .schema
        .assignment_types_for_state(asset_schema.default_state_type())[0];
    let transition_type = contract
        .schema
        .default_transition_for_assignment(assignment_type);

    let utxo_1 = wlt_1.get_utxo(Some(8000));
    let amt_0 = 500;
    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        amt_0,
        InvoiceType::Blinded(Some(utxo_1)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment, None);
    wlt_1.sync();

    let (base_consignment, tx) = wlt_1.send(
        &mut wlt_2,
        InvoiceType::Blinded(None),
        contract_id,
        amt_0,
        1000,
        None,
    );
    let base_txid = txid_bp_to_bitcoin(tx.txid());

    // allocate additional assets on spent utxo_1
    let amt_1 = 400;
    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        amt_1,
        InvoiceType::Blinded(Some(utxo_1)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment, None);
    wlt_1.sync();

    let (opout, _) = wlt_1
        .contract_assignments_for(contract_id, vec![utxo_1])
        .into_values()
        .flat_map(|s| s.into_iter())
        .find(|(_, s)| match s {
            AllocatedState::Amount(a) => a.as_u64() == amt_1,
            _ => {
                panic!("unexpected allocatedState");
            }
        })
        .unwrap();

    let mut new_consignment = base_consignment.clone();
    let mut bundles = base_consignment.bundles.release().clone();

    let mut transition_builder = wlt_1
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(amt_1);
    transition_builder = transition_builder.add_input(opout, state.clone()).unwrap();
    let secret_seal = wlt_2.get_secret_seal(None, None);
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, BuilderSeal::Concealed(secret_seal), state)
        .unwrap();
    let new_transition = transition_builder.complete_transition().unwrap();
    let new_opid = new_transition.id();

    let new_bundle = bundles
        .iter_mut()
        .find(|b| b.pub_witness.txid() == base_txid)
        .unwrap();
    let bundle_id = new_bundle.bundle.bundle_id();
    new_bundle
        .bundle
        .known_transitions
        .push(KnownTransition::new(new_opid, new_transition))
        .unwrap();
    assert_eq!(bundle_id, new_bundle.bundle.bundle_id());
    assert!(new_bundle.bundle.known_transitions_contain_opid(&new_opid));
    new_consignment.bundles = LargeVec::from_checked(bundles);

    wlt_2.accept_transfer(new_consignment, None);
}

#[cfg(not(feature = "altered"))]
#[should_panic(expected = "InputMapTransitionMismatch")]
#[test]
fn uncommitted_input_opout() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_amt = 900;
    let contract_id = wlt_1.issue_nia(issued_amt, None);
    let schema_id = wlt_1.schema_id(contract_id);

    // split issued amount into 2 opouts
    let amt_0 = 500;
    let invoice = wlt_1.invoice(contract_id, schema_id, amt_0, InvoiceType::Witness);
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment, None);
    wlt_1.sync();

    // merge the 2 allocation to send to wlt_2
    let invoice = wlt_2.invoice(contract_id, schema_id, issued_amt, InvoiceType::Witness);
    let (mut psbt, _, mut consignment) = wlt_1.pay_invoice(invoice, None, None);
    let prev_txids = consignment
        .bundles
        .iter()
        .map(|wb| wb.witness_id())
        .collect::<HashSet<_>>();

    // remove commitment to one of the spent opouts
    consignment.modify_bundle(txid_bp_to_bitcoin(psbt.txid()), |witness_bundle| {
        let mut input_map = witness_bundle.bundle.input_map.clone().release();
        input_map.pop_last();
        witness_bundle.bundle.input_map = NonEmptyOrdMap::from_checked(input_map);
        let tx = tx_bitcoin_to_bp(witness_bundle.pub_witness.tx().unwrap().clone());
        let mut witness_psbt = BpPsbt::from_tx(tx);
        let idx = witness_psbt
            .outputs()
            .find(|o| o.script.is_op_return())
            .unwrap()
            .index();
        let contract_id = witness_bundle
            .bundle
            .known_transitions
            .last()
            .unwrap()
            .transition
            .contract_id;
        let protocol_id = mpc::ProtocolId::from(contract_id);
        let message = mpc::Message::from(witness_bundle.bundle.bundle_id());
        witness_psbt.output_mut(idx).unwrap().script = ScriptPubkey::op_return(&[]);
        witness_psbt.output_mut(idx).unwrap().set_opret_host();
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
        witness_bundle.anchor.mpc_proof = proof.to_merkle_proof(protocol_id).unwrap();
        witness_bundle.pub_witness = PubWitness::Tx(tx_bp_to_bitcoin(witness.clone()));
    });
    let tx = consignment
        .bundles
        .iter()
        .find(|wb| !prev_txids.contains(&wb.witness_id()))
        .unwrap()
        .pub_witness
        .tx()
        .unwrap();
    let opret_script = tx
        .output
        .iter()
        .find(|o| o.script_pubkey.is_op_return())
        .unwrap()
        .script_pubkey
        .clone();
    psbt.outputs_mut()
        .find(|o| o.script.is_op_return())
        .unwrap()
        .script = script_buf_to_script_pubkey(opret_script);
    assert_eq!(tx.compute_txid(), txid_bp_to_bitcoin(psbt.txid()));
    let new_tx = wlt_1.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&new_tx);
    wlt_2.accept_transfer(consignment, None);
}

#[cfg(not(feature = "altered"))]
#[test]
fn concealed_known_transition() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_amt = 700;
    let contract_id = wlt_1.issue_nia(issued_amt, None);
    let asset_schema = wlt_1.asset_schema(contract_id);
    let schema_id = wlt_1.schema_id(contract_id);
    let contract = wlt_1.stock().contract_data(contract_id).unwrap();
    let assignment_type = contract
        .schema
        .assignment_types_for_state(asset_schema.default_state_type())[0];
    let transition_type = contract
        .schema
        .default_transition_for_assignment(assignment_type);

    // prepare 2 allocations on utxo
    let utxo = wlt_1.get_utxo(None);

    let amt_1 = 300;
    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        amt_1,
        InvoiceType::Blinded(Some(utxo)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment, None);
    wlt_1.sync();

    let amt_2 = 400;
    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        amt_2,
        InvoiceType::Blinded(Some(utxo)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment, None);
    wlt_1.sync();

    // retrieve the two opouts on utxo
    let allocations = wlt_1
        .contract_assignments_for(contract_id, vec![utxo])
        .into_values()
        .flat_map(|v| v.into_iter())
        .collect::<Vec<_>>();
    assert_eq!(allocations.len(), 2);
    let (opout_1, amt_1) = if let (opout, AllocatedState::Amount(state)) = allocations[0] {
        (opout, state.as_u64())
    } else {
        panic!("unexpected state type");
    };
    let (opout_2, amt_2) = if let (opout, AllocatedState::Amount(state)) = allocations[1] {
        (opout, state.as_u64())
    } else {
        panic!("unexpected state type");
    };

    // construct transaction committing to bundle with missing transition
    let btc_change = wlt_1.get_address();
    let (mut psbt, _) = wlt_1.construct_psbt(vec![utxo], vec![(btc_change, None)], None);
    psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO);
    psbt.output_mut(1).unwrap().set_opret_host();
    psbt.set_rgb_close_method(CloseMethod::OpretFirst);

    // 1st transition
    let mut transition_builder = wlt_1
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(amt_1);
    transition_builder = transition_builder
        .add_input(opout_1, state.clone())
        .unwrap();
    let secret_seal_1 = wlt_2.get_secret_seal(None, None);
    let seal_1 = BuilderSeal::Concealed(secret_seal_1);
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal_1, state)
        .unwrap();
    let transition = transition_builder.complete_transition().unwrap();
    for opout in transition.inputs() {
        // this is not necessary since it's done by push_rgb_transition,
        // but it shows that it's idempotent
        psbt.set_rgb_contract_consumer(contract_id, opout, transition.id())
            .unwrap();
    }
    psbt.push_rgb_transition(transition).unwrap();

    // 2nd transition
    let mut transition_builder = wlt_1
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(amt_2);
    transition_builder = transition_builder
        .add_input(opout_2, state.clone())
        .unwrap();
    let secret_seal_2 = wlt_2.get_secret_seal(None, None);
    let seal_2 = BuilderSeal::Concealed(secret_seal_2);
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal_2, state)
        .unwrap();
    let transition = transition_builder.complete_transition().unwrap();
    let opid_2 = transition.id();
    for opout in transition.inputs() {
        psbt.set_rgb_contract_consumer(contract_id, opout, opid_2)
            .unwrap();
    }
    // we don't push this transition to keep it concealed

    psbt.set_as_unmodifiable();
    let fascia = psbt.rgb_commit().unwrap();
    let witness_id = psbt.txid();
    wlt_1.consume_fascia(fascia, witness_id);
    let tx = wlt_1.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);
    wlt_2.sync();

    let mut beneficiaries = AssetBeneficiariesMap::new();
    beneficiaries.insert(contract_id, vec![seal_1]);
    let consignment = wlt_1
        .create_consignments(beneficiaries, witness_id)
        .into_values()
        .next()
        .unwrap();

    // ensure the consignment contains the bundle with missing transition
    let bundle = consignment
        .bundles
        .iter()
        .find(|wb| wb.bundle.input_map_opids().contains(&opid_2))
        .unwrap();
    assert!(!bundle.bundle.known_transitions_contain_opid(&opid_2));

    wlt_2.accept_transfer(consignment, None);
}

#[cfg(not(feature = "altered"))]
#[should_panic(expected = "MissingScript")]
#[test]
fn remove_scripts_code() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_amt = 700;
    let utxo = wlt_1.get_utxo(None);
    let contract_id = wlt_1.issue_nia(issued_amt, Some(&utxo));
    let asset_schema = wlt_1.asset_schema(contract_id);
    let contract = wlt_1.stock().contract_data(contract_id).unwrap();
    let assignment_type = contract
        .schema
        .assignment_types_for_state(asset_schema.default_state_type())[0];
    let transition_type = contract
        .schema
        .default_transition_for_assignment(assignment_type);

    // construct transaction committing to bundle with missing transition
    let btc_change = wlt_1.get_address();
    let (mut psbt, _) = wlt_1.construct_psbt(vec![utxo], vec![(btc_change, None)], None);
    psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO);
    psbt.output_mut(1).unwrap().set_opret_host();
    psbt.set_rgb_close_method(CloseMethod::OpretFirst);

    // 1st transition
    let opout = Opout {
        op: OpId::copy_from_slice(contract_id.as_slice()).unwrap(),
        ty: OS_ASSET,
        no: 0,
    };
    let mut transition_builder = wlt_1
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(issued_amt);
    transition_builder = transition_builder.add_input(opout, state.clone()).unwrap();
    let secret_seal_1 = wlt_2.get_secret_seal(None, None);
    let seal_1 = BuilderSeal::Concealed(secret_seal_1);
    let state = asset_schema.allocated_state(issued_amt + 1);
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal_1, state)
        .unwrap();
    let transition = transition_builder.complete_transition().unwrap();
    for opout in transition.inputs() {
        // this is not necessary since it's done by push_rgb_transition,
        // but it shows that it's idempotent
        psbt.set_rgb_contract_consumer(contract_id, opout, transition.id())
            .unwrap();
    }
    psbt.push_rgb_transition(transition).unwrap();

    psbt.set_as_unmodifiable();
    let fascia = psbt.rgb_commit().unwrap();
    let witness_id = psbt.txid();
    wlt_1.consume_fascia(fascia, witness_id);
    let tx = wlt_1.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);
    wlt_2.sync();

    let mut beneficiaries = AssetBeneficiariesMap::new();
    beneficiaries.insert(contract_id, vec![seal_1]);
    let mut consignment = wlt_1
        .create_consignments(beneficiaries, witness_id)
        .into_values()
        .next()
        .unwrap();
    let mut scripts = consignment.scripts.clone().release();
    let mut lib = scripts.pop_last().unwrap().clone();
    lib.code = none!();
    consignment.scripts = Confined::<BTreeSet<_>, 0, 1024>::from_checked(bset![lib]);

    // should fail here
    wlt_2.accept_transfer(consignment, None);

    // only fails here
    wlt_2.send(
        &mut wlt_1,
        InvoiceType::Witness,
        contract_id,
        issued_amt,
        1000,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn accept_bundle_missing_transitions() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_3 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_amt = 700;
    let contract_id = wlt_1.issue_nia(issued_amt, None);
    let asset_schema = wlt_1.asset_schema(contract_id);
    let schema_id = wlt_1.schema_id(contract_id);
    let contract = wlt_1.stock().contract_data(contract_id).unwrap();
    let assignment_type = contract
        .schema
        .assignment_types_for_state(asset_schema.default_state_type())[0];
    let transition_type = contract
        .schema
        .default_transition_for_assignment(assignment_type);

    // split into 2 allocations
    let utxo_1 = wlt_1.get_utxo(None);
    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        300,
        InvoiceType::Blinded(Some(utxo_1)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment.clone(), None);
    wlt_1.sync();

    // construct bundle that will be omitted in first validation
    let utxo_2 = wlt_1.get_utxo(None);
    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        400,
        InvoiceType::Blinded(Some(utxo_2)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment.clone(), None);
    wlt_1.sync();

    // retrieve the two opouts on utxo
    let allocations = wlt_1
        .contract_assignments_for(contract_id, vec![utxo_1])
        .into_values()
        .flat_map(|v| v.into_iter())
        .collect::<Vec<_>>();
    assert_eq!(allocations.len(), 1);
    let (opout_1, amt_1) = if let (opout, AllocatedState::Amount(state)) = allocations[0] {
        (opout, state.as_u64())
    } else {
        panic!("unexpected state type");
    };
    let allocations = wlt_1
        .contract_assignments_for(contract_id, vec![utxo_2])
        .into_values()
        .flat_map(|v| v.into_iter())
        .collect::<Vec<_>>();
    assert_eq!(allocations.len(), 1);
    let (opout_2, amt_2) = if let (opout, AllocatedState::Amount(state)) = allocations[0] {
        (opout, state.as_u64())
    } else {
        panic!("unexpected state type");
    };

    // construct bundle that will be accepted twice, first with missing transition
    let btc_change = wlt_1.get_address();
    let (mut psbt, _) = wlt_1.construct_psbt(vec![utxo_1, utxo_2], vec![(btc_change, None)], None);
    psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO);
    psbt.output_mut(1).unwrap().set_opret_host();
    psbt.set_rgb_close_method(CloseMethod::OpretFirst);

    // 1st transition (revealed)
    let mut transition_builder = wlt_1
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(amt_1);
    transition_builder = transition_builder
        .add_input(opout_1, state.clone())
        .unwrap();
    let secret_seal_1 = wlt_2.get_secret_seal(None, None);
    let seal_1 = BuilderSeal::Concealed(secret_seal_1);
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal_1, state)
        .unwrap();
    let transition = transition_builder.complete_transition().unwrap();
    let opid_1 = transition.id();
    psbt.push_rgb_transition(transition).unwrap();

    // 2nd transition (concealed at first, revealed later)
    let mut transition_builder = wlt_1
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(amt_2);
    transition_builder = transition_builder
        .add_input(opout_2, state.clone())
        .unwrap();
    let secret_seal_2 = wlt_3.get_secret_seal(None, None);
    let seal_2 = BuilderSeal::Concealed(secret_seal_2);
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal_2, state)
        .unwrap();
    let transition = transition_builder.complete_transition().unwrap();
    let opid_2 = transition.id();
    psbt.push_rgb_transition(transition).unwrap();

    psbt.set_as_unmodifiable();
    let fascia = psbt.rgb_commit().unwrap();
    let witness_id = psbt.txid();
    wlt_1.consume_fascia(fascia, witness_id);
    let tx = wlt_1.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);
    wlt_2.sync();

    let mut beneficiaries = AssetBeneficiariesMap::new();
    beneficiaries.insert(contract_id, vec![seal_1]);
    let consignment_1 = wlt_1
        .create_consignments(beneficiaries, witness_id)
        .into_values()
        .next()
        .unwrap();
    assert!(
        consignment_1
            .bundles
            .iter()
            .all(|wb| !wb.bundle.known_transitions_opids().contains(&opid_2))
    );
    //wlt_2 accepts bundle with opid_1 revealed and opid_2 concealed
    wlt_2.accept_transfer(consignment_1, None);

    let mut beneficiaries = AssetBeneficiariesMap::new();
    beneficiaries.insert(contract_id, vec![seal_2]);
    let consignment_2 = wlt_1
        .create_consignments(beneficiaries, witness_id)
        .into_values()
        .next()
        .unwrap();
    assert!(
        consignment_2
            .bundles
            .iter()
            .all(|wb| !wb.bundle.known_transitions_opids().contains(&opid_1))
    );
    // wlt_3 accepts the same bundle with opid_1 concealed and opid_2 revealed
    wlt_3.accept_transfer(consignment_2, None);

    let _ = wlt_3.send(
        &mut wlt_2,
        InvoiceType::Blinded(None),
        contract_id,
        amt_2,
        0,
        None,
    );

    // wlt_2 can spend allocation from both opid_1 and opid_2
    wlt_2.check_allocations(contract_id, asset_schema, vec![amt_1, amt_2], false);
    wlt_2.send(
        &mut wlt_1,
        InvoiceType::Blinded(None),
        contract_id,
        amt_1 + amt_2,
        0,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[test]
fn unordered_transitions_within_bundle() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let utxo_0 = wlt_1.get_utxo(Some(8000));
    let issued_amt = 666;
    let contract_id = wlt_1.issue_nia(issued_amt, Some(&utxo_0));
    let asset_schema = wlt_1.asset_schema(contract_id);
    let contract = wlt_1.wallet.stock().contract_data(contract_id).unwrap();
    let assignment_type = contract
        .schema
        .assignment_types_for_state(asset_schema.default_state_type())[0];
    let transition_type = contract
        .schema
        .default_transition_for_assignment(assignment_type);

    let utxo_1 = wlt_1.get_utxo(Some(7000));

    let utxo_2 = wlt_2.get_utxo(None);
    let btc_change = wlt_1.get_address();
    let (mut psbt, _) = wlt_1.construct_psbt(vec![utxo_0, utxo_1], vec![(btc_change, None)], None);
    psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO);
    psbt.output_mut(1).unwrap().set_opret_host();
    psbt.set_rgb_close_method(CloseMethod::OpretFirst);

    let mut beneficiaries = AssetBeneficiariesMap::new();
    let mut transition_builder = wlt_1
        .wallet
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let opout = Opout::new(
        OpId::copy_from_slice(contract_id.as_slice()).unwrap(),
        *assignment_type,
        0,
    );
    let state = asset_schema.allocated_state(issued_amt);
    transition_builder = transition_builder.add_input(opout, state.clone()).unwrap();
    let seal = BuilderSeal::Revealed(GraphSeal::rand_from(utxo_1));
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal, state)
        .unwrap();
    beneficiaries.push((contract_id, vec![seal]));
    let transition_1 = transition_builder.complete_transition().unwrap();
    let opid_1 = transition_1.id();
    psbt.push_rgb_transition(transition_1).unwrap();

    let mut transition_builder = wlt_1
        .wallet
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let opout = Opout::new(opid_1, *assignment_type, 0);
    let state = asset_schema.allocated_state(issued_amt);
    transition_builder = transition_builder.add_input(opout, state.clone()).unwrap();
    let seal = BuilderSeal::Concealed(wlt_2.get_secret_seal(Some(utxo_2), None));
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal, state)
        .unwrap();
    beneficiaries.push((contract_id, vec![seal]));
    let mut transition_2 = transition_builder.clone().complete_transition().unwrap();
    // mine transition_2 until its opid is lower than transition_1
    while opid_1 < transition_2.id() {
        transition_2.nonce -= 1;
    }
    psbt.push_rgb_transition(transition_2).unwrap();
    psbt.set_as_unmodifiable();
    let fascia = psbt.rgb_commit().unwrap();
    let witness_id = psbt.txid();
    wlt_1.consume_fascia(fascia, witness_id);

    let tx = wlt_1.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);
    wlt_2.sync();

    let consignments = wlt_1.create_consignments(beneficiaries, witness_id);
    for (_, consignment) in consignments {
        wlt_2.accept_transfer(consignment, None);
    }

    wlt_2.send(
        &mut wlt_1,
        InvoiceType::Witness,
        contract_id,
        issued_amt,
        1000,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[should_panic(expected = "InputMapTransitionMismatch")]
#[test]
fn transition_spending_uncommitted_opout() {
    initialize();

    let mut wlt_1 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_2 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);
    let mut wlt_3 = BpTestWallet::with_descriptor(&DescriptorType::Wpkh);

    let issued_amt = 700;
    let contract_id = wlt_1.issue_nia(issued_amt, None);
    let asset_schema = wlt_1.asset_schema(contract_id);
    let schema_id = wlt_1.schema_id(contract_id);
    let contract = wlt_1.stock().contract_data(contract_id).unwrap();
    let assignment_type = contract
        .schema
        .assignment_types_for_state(asset_schema.default_state_type())[0];
    let transition_type = contract
        .schema
        .default_transition_for_assignment(assignment_type);

    // split into 2 allocations on a single utxo
    let utxo_1 = wlt_1.get_utxo(None);
    let amt_1 = 300;
    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        amt_1,
        InvoiceType::Blinded(Some(utxo_1)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment.clone(), None);
    let amt_2 = 400;
    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        amt_2,
        InvoiceType::Blinded(Some(utxo_1)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice, None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment.clone(), None);
    wlt_1.sync();

    // construct bundle with 2 transitions, one per allocation
    let btc_change = wlt_1.get_address();
    let (mut psbt, _) = wlt_1.construct_psbt(vec![utxo_1], vec![(btc_change, None)], None);
    psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO);
    psbt.output_mut(1).unwrap().set_opret_host();
    psbt.set_rgb_close_method(CloseMethod::OpretFirst);

    let mut beneficiaries = AssetBeneficiariesMap::new();
    // transition 1: spends opid_1 and will be concealed
    let (opout_1, _) = wlt_1
        .contract_assignments_for(contract_id, vec![utxo_1])
        .into_values()
        .flat_map(|s| s.into_iter())
        .find(|(_, s)| match s {
            AllocatedState::Amount(a) => a.as_u64() == amt_1,
            _ => {
                panic!("unexpected allocatedState");
            }
        })
        .unwrap();
    let mut transition_builder = wlt_1
        .wallet
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(amt_1);
    transition_builder = transition_builder
        .add_input(opout_1, state.clone())
        .unwrap();
    let seal = BuilderSeal::Concealed(wlt_2.get_secret_seal(None, None));
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal, state)
        .unwrap();
    let transition_1 = transition_builder.complete_transition().unwrap();
    let opid_1 = transition_1.id();

    // transition 2: spends opid_1+opid_2 and will be revealed
    let (opout_2, _) = wlt_1
        .contract_assignments_for(contract_id, vec![utxo_1])
        .into_values()
        .flat_map(|s| s.into_iter())
        .find(|(_, s)| match s {
            AllocatedState::Amount(a) => a.as_u64() == amt_2,
            _ => {
                panic!("unexpected allocatedState");
            }
        })
        .unwrap();
    let mut transition_builder = wlt_1
        .wallet
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    transition_builder = transition_builder
        .add_input(opout_1, asset_schema.allocated_state(amt_1))
        .unwrap();
    transition_builder = transition_builder
        .add_input(opout_2, asset_schema.allocated_state(amt_2))
        .unwrap();
    let seal = BuilderSeal::Concealed(wlt_3.get_secret_seal(None, None));
    transition_builder = transition_builder
        .add_owned_state_raw(
            *assignment_type,
            seal,
            asset_schema.allocated_state(amt_1 + amt_2),
        )
        .unwrap();
    beneficiaries.push((contract_id, vec![seal]));
    let transition_2 = transition_builder.complete_transition().unwrap();
    let _opid_2 = transition_2.id();

    psbt.push_rgb_transition(transition_2).unwrap();
    // override input map to obtain { opout_1 => opid_1, opout_2 => opid_2 }
    let key = PropKey::rgb_consumed_by(contract_id);
    let existing_data = psbt.proprietary(&key).unwrap();
    let mut items = OpoutAndOpids::deserialize(existing_data).unwrap();
    items.insert(opout_1, opid_1);
    psbt.insert_proprietary(key, items.serialize().into());

    psbt.set_as_unmodifiable();
    let fascia = psbt.rgb_commit().unwrap();
    let witness_id = psbt.txid();
    wlt_1.consume_fascia(fascia, witness_id);
    let tx = wlt_1.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);
    wlt_2.sync();

    let consignments = wlt_1.create_consignments(beneficiaries, witness_id);
    for (_, consignment) in consignments {
        wlt_3.accept_transfer(consignment, None);
    }
}

#[cfg(not(feature = "altered"))]
#[rstest]
fn multiasset_transfer() {
    initialize();

    let wlt_desc = DescriptorType::Tr;
    let mut wlt_1 = BpTestWallet::with_descriptor(&wlt_desc);
    let mut wlt_2 = BpTestWallet::with_descriptor(&wlt_desc);

    let utxo = wlt_1.get_utxo(None);

    let amt_1 = 200;
    let contract_id_1 = wlt_1.issue_nia(amt_1, Some(&utxo));

    let amt_2 = 100;
    let contract_id_2 = wlt_1.issue_nia(amt_2, Some(&utxo));

    let amt_chg = 50;
    let coloring_info = ColoringInfo {
        asset_info_map: HashMap::from([
            (
                contract_id_1,
                AssetColoringInfo {
                    input_outpoints: vec![utxo],
                    assignments: vec![AssetAssignment {
                        destination: wlt_2.get_secret_seal(None, None).into(),
                        amount: amt_1 - amt_chg,
                    }],
                },
            ),
            (
                contract_id_2,
                AssetColoringInfo {
                    input_outpoints: vec![utxo],
                    assignments: vec![AssetAssignment {
                        destination: wlt_2.get_secret_seal(None, None).into(),
                        amount: amt_2 - amt_chg,
                    }],
                },
            ),
        ]),
        static_blinding: None,
        nonce: None,
        close_method: wlt_1.close_method(),
    };
    let (consignments, tx, _, tweak_info) = wlt_1.pay_full_flexible(coloring_info, None, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    for consignment in consignments.into_values() {
        wlt_2.accept_transfer(consignment.clone(), None);
    }
    if let Some((witness_info, tapret_commitment)) = tweak_info {
        wlt_2.add_tapret_tweak(witness_info.terminal(), tapret_commitment);
    }
    wlt_2.sync();
    wlt_1.sync();

    wlt_1.check_allocations(contract_id_1, AssetSchema::Nia, vec![amt_chg], false);
    wlt_1.check_allocations(contract_id_2, AssetSchema::Nia, vec![amt_chg], false);

    wlt_2.check_allocations(
        contract_id_1,
        AssetSchema::Nia,
        vec![amt_1 - amt_chg],
        false,
    );
    wlt_2.check_allocations(
        contract_id_2,
        AssetSchema::Nia,
        vec![amt_2 - amt_chg],
        false,
    );
    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id_1,
        amt_1 - amt_chg - 1,
        0,
        None,
    );
    wlt_2.send(
        &mut wlt_1,
        TransferType::Blinded,
        contract_id_2,
        amt_2 - amt_chg - 1,
        0,
        None,
    );
}

#[cfg(not(feature = "altered"))]
#[rstest]
fn extra_after_merge() {
    initialize();

    let wlt_desc = DescriptorType::Wpkh;
    let mut wlt_1 = BpTestWallet::with_descriptor(&wlt_desc);
    let mut wlt_2 = BpTestWallet::with_descriptor(&wlt_desc);

    let utxo_1 = wlt_1.get_utxo(None);
    let utxo_2 = wlt_1.get_utxo(None);

    let amt_1 = 100;
    let amt_2 = 200;

    let contract_id = wlt_1.issue_with_info(
        AssetInfo::default_nia(vec![amt_1, amt_2]),
        vec![Some(utxo_1), Some(utxo_2)],
        None,
        None,
    );
    let asset_schema = wlt_1.asset_schema(contract_id);
    let schema_id = wlt_1.schema_id(contract_id);
    let contract = wlt_1.stock().contract_data(contract_id).unwrap();
    let assignment_type = contract
        .schema
        .assignment_types_for_state(asset_schema.default_state_type())[0];
    let transition_type = contract
        .schema
        .default_transition_for_assignment(assignment_type);

    let invoice = wlt_1.invoice(
        contract_id,
        schema_id,
        amt_2,
        InvoiceType::Blinded(Some(utxo_1)),
    );
    let (consignment, tx, _, _) = wlt_1.pay_full(invoice.clone(), None, None, true, None);
    wlt_1.mine_tx(&txid_bp_to_bitcoin(tx.txid()), false);
    wlt_1.accept_transfer(consignment, None);
    wlt_1.sync();

    // retrieve the two opouts on utxo
    let mut allocations = wlt_1
        .contract_assignments_for(contract_id, vec![utxo_1])
        .into_values()
        .flat_map(|v| v.into_iter())
        .collect::<Vec<_>>();
    allocations.sort_by(|(_, a1), (_, a2)| a1.cmp(a2));
    assert_eq!(allocations.len(), 2);
    let (opout_1, _) = allocations[0];
    let (opout_2, _) = allocations[1];

    let btc_change = wlt_1.get_address();
    let (mut psbt, _) = wlt_1.construct_psbt(vec![utxo_1], vec![(btc_change, None)], None);
    psbt.construct_output_expect(ScriptPubkey::op_return(&[]), Sats::ZERO);
    psbt.output_mut(1).unwrap().set_opret_host();
    psbt.set_rgb_close_method(CloseMethod::OpretFirst);
    // 1st transition
    let mut transition_builder = wlt_1
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(amt_1);
    transition_builder = transition_builder
        .add_input(opout_1, state.clone())
        .unwrap();
    let secret_seal = wlt_2.get_secret_seal(None, None);
    let seal_1 = BuilderSeal::Concealed(secret_seal);
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal_1, state)
        .unwrap();
    let transition = transition_builder.complete_transition().unwrap();
    psbt.push_rgb_transition(transition).unwrap();
    // 2nd transition
    let mut transition_builder = wlt_1
        .stock()
        .transition_builder_raw(contract_id, transition_type)
        .unwrap();
    let state = asset_schema.allocated_state(amt_2);
    transition_builder = transition_builder
        .add_input(opout_2, state.clone())
        .unwrap();
    let secret_seal_extra = wlt_1.get_secret_seal(None, None);
    let seal_2 = BuilderSeal::Concealed(secret_seal_extra);
    transition_builder = transition_builder
        .add_owned_state_raw(*assignment_type, seal_2, state)
        .unwrap();
    let transition = transition_builder.complete_transition().unwrap();
    psbt.push_rgb_transition(transition).unwrap();
    psbt.set_as_unmodifiable();
    let fascia = psbt.rgb_commit().unwrap();
    let witness_id = psbt.txid();
    wlt_1.consume_fascia(fascia, witness_id);
    let tx = wlt_1.sign_finalize_extract(&mut psbt);
    wlt_1.broadcast_tx(&tx);
    wlt_2.sync();

    let mut beneficiaries = AssetBeneficiariesMap::new();
    beneficiaries.insert(contract_id, vec![seal_1]);
    let consignment = wlt_1
        .create_consignments(beneficiaries, witness_id)
        .into_values()
        .next()
        .unwrap();
    wlt_2.accept_transfer(consignment, None);
}

#[cfg(not(feature = "altered"))]
#[rstest]
fn contract_linking() {
    initialize();

    let wlt_desc = DescriptorType::Wpkh;
    let mut wlt_1 = BpTestWallet::with_descriptor(&wlt_desc);
    let mut wlt_2 = BpTestWallet::with_descriptor(&wlt_desc);

    let issuance_utxo_1 = wlt_1.get_utxo(None);
    let amt_1 = 100;
    let mut asset_info_1 = AssetInfo::default_ifa(vec![amt_1], vec![]);
    if let AssetInfo::Ifa {
        ref mut link_info, ..
    } = asset_info_1
    {
        *link_info = (None, Some(issuance_utxo_1));
    }
    let contract_id_1 =
        wlt_1.issue_with_info(asset_info_1, vec![Some(issuance_utxo_1)], None, None);

    let issuance_utxo_2 = wlt_1.get_utxo(None);
    let amt_2 = 200;
    let mut asset_info_2 = AssetInfo::default_ifa(vec![amt_2], vec![]);
    if let AssetInfo::Ifa {
        ref mut link_info, ..
    } = asset_info_2
    {
        *link_info = (Some(contract_id_1), None);
    }
    let contract_id_2 =
        wlt_1.issue_with_info(asset_info_2, vec![Some(issuance_utxo_2)], None, None);

    // sending contract_id_1 allocation causes link right to be moved in extra
    let schema_id = wlt_1.schema_id(contract_id_1);
    let invoice = wlt_2.invoice(contract_id_1, schema_id, amt_1, InvoiceType::Blinded(None));
    let (consignment, tx, _, psbt_meta) = wlt_1.pay_full(invoice, None, None, true, None);
    let txid = txid_bp_to_bitcoin(tx.txid());
    wlt_1.mine_tx(&txid, false);
    wlt_2.accept_transfer(consignment, None);
    wlt_1.sync();
    let link_utxo = Outpoint::new(txid, psbt_meta.change_vout.unwrap());
    let (link_consignment, _) = wlt_1.link_ifa(contract_id_1, contract_id_2, link_utxo);
    wlt_1.send_ifa(&mut wlt_2, InvoiceType::Blinded(None), contract_id_2, amt_2);

    // contract link validation only succeeds after accepting the linking consignment
    assert_eq!(
        contract_id_1,
        wlt_2
            .contract_wrapper::<InflatableFungibleAsset>(contract_id_2)
            .link_from()
            .unwrap()
            .unwrap()
    );
    assert!(
        wlt_2
            .contract_wrapper::<InflatableFungibleAsset>(contract_id_1)
            .link_to()
            .unwrap()
            .is_none()
    );
    wlt_2
        .wallet
        .stock()
        .validate_contracts_link::<InflatableFungibleAsset, InflatableFungibleAsset>(
            contract_id_1,
            contract_id_2,
        )
        .unwrap_err();
    wlt_2.accept_transfer(link_consignment, None);
    wlt_2
        .wallet
        .stock()
        .validate_contracts_link::<InflatableFungibleAsset, InflatableFungibleAsset>(
            contract_id_1,
            contract_id_2,
        )
        .unwrap();
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

    let mut wlt_1 = BpTestWallet::with(&DescriptorType::Wpkh, Some(INSTANCE_2), true);
    let mut wlt_2 = BpTestWallet::with(&DescriptorType::Wpkh, Some(INSTANCE_2), true);

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

    let mut wlt_3 = BpTestWallet::with(&DescriptorType::Wpkh, Some(INSTANCE_2), true);

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

    let mut wlt_1 = BpTestWallet::with(&DescriptorType::Wpkh, Some(INSTANCE_2), true);
    let mut wlt_2 = BpTestWallet::with(&DescriptorType::Wpkh, Some(INSTANCE_2), true);

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
            let tx_0_txid = txid_bp_to_bitcoin(tx_0.txid());

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
                let height = if txid == tx_0_txid {
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
            let txid = txid_bp_to_bitcoin(tx_2.txid());
            wlt_2.mine_tx(&txid, false);
            // receiver checks if it's safe to receive allocations
            let safe_height = height_pre_transfer; // min 1 confirmation
            let trusted_typesystem = AssetSchema::from(consignment.schema_id()).types();
            let validation_config = ValidationConfig {
                chain_net: wlt_1.chain_net(),
                safe_height: Some(NonZeroU32::new(safe_height).unwrap()),
                trusted_typesystem,
                ..Default::default()
            };
            let validated_consignment = consignment
                .clone()
                .validate(&wlt_1.get_resolver(), &validation_config)
                .unwrap();
            let validation_status = validated_consignment.clone().into_validation_status();
            assert_eq!(validation_status.warnings.len(), 1);
            let unsafe_height = height_pre_transfer + 1;
            let unsafe_history_map = HashMap::from([(unsafe_height, HashSet::from([txid]))]);
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

    let mut wlt = BpTestWallet::with(&DescriptorType::Wpkh, Some(INSTANCE_2), true);

    let issued_supply = 600;
    let utxo = wlt.get_utxo(None);
    let contract_id = wlt.issue_nia(issued_supply, Some(&utxo));
    let schema_id = wlt.schema_id(contract_id);

    wlt.check_allocations(contract_id, schema_id, vec![issued_supply], false);

    if with_transfers {
        let mut recv_wlt = BpTestWallet::with(&DescriptorType::Wpkh, Some(INSTANCE_2), true);
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
    let utxos = wlt.list_unspents();
    assert!(utxos.is_empty());

    wlt.check_allocations(contract_id, schema_id, vec![], false);
}
