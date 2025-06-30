pub mod utils;

use utils::*;

type TT = TransferType;
type DT = DescriptorType;

#[cfg(not(feature = "altered"))]
#[rstest]
// blinded
#[case(TT::Blinded, DT::Wpkh, DT::Wpkh)]
#[case(TT::Blinded, DT::Wpkh, DT::Tr)]
#[case(TT::Blinded, DT::Tr, DT::Tr)]
// witness
#[case(TT::Witness, DT::Wpkh, DT::Wpkh)]
#[case(TT::Witness, DT::Wpkh, DT::Tr)]
#[case(TT::Witness, DT::Tr, DT::Tr)]
#[ignore = "run a single case if desired"]
fn back_and_forth(
    #[case] transfer_type: TransferType,
    #[case] wlt_1_desc: DescriptorType,
    #[case] wlt_2_desc: DescriptorType,
) {
    println!("transfer_type {transfer_type:?} wlt_1_desc {wlt_1_desc:?} wlt_2_desc {wlt_2_desc:?}");

    initialize();

    let stress_tests_dir = PathBuf::from(TEST_DATA_DIR).join(STRESS_DATA_DIR);
    std::fs::create_dir_all(&stress_tests_dir).unwrap();
    let fname = OffsetDateTime::unix_timestamp(OffsetDateTime::now_utc()).to_string();
    let mut fpath = stress_tests_dir.join(fname);
    fpath.set_extension("csv");
    println!("report path: {}", fpath.to_string_lossy());
    let report = Report { report_path: fpath };
    report.write_header(&[
        "wlt_1_pay",
        "wlt_2_validate",
        "wlt_2_accept",
        "wlt_2_pay",
        "wlt_1_validate",
        "wlt_1_accept",
        "send_1_tot",
        "send_2_tot",
    ]);

    let mut wlt_1 = get_wallet(&wlt_1_desc);
    let mut wlt_2 = get_wallet(&wlt_2_desc);

    let issued_supply = u64::MAX;

    let contract_id = wlt_1.issue_nia(issued_supply, None);

    let loops = match std::env::var("LOOPS") {
        Ok(val) if u16::from_str(&val).is_ok() => u16::from_str(&val).unwrap(),
        Err(VarError::NotPresent) => 50,
        _ => {
            panic!("invalid loops value: must be a u16 number")
        }
    };

    let sats_base = 3000;
    let mut sats_send = sats_base * loops as u64;
    let now = Instant::now();
    for i in 1..=loops {
        println!("loop {i}/{loops}");
        sats_send -= DEFAULT_FEE_ABS * 2;
        let wlt_1_send_start = Instant::now();
        wlt_1.send(
            &mut wlt_2,
            transfer_type,
            contract_id,
            issued_supply - i as u64,
            sats_send,
            Some(&report),
        );
        let wlt_1_send_duration = wlt_1_send_start.elapsed();
        sats_send -= DEFAULT_FEE_ABS * 2;
        let wlt_2_send_start = Instant::now();
        wlt_2.send(
            &mut wlt_1,
            transfer_type,
            contract_id,
            issued_supply - i as u64 - 1,
            sats_send,
            Some(&report),
        );
        let wlt_2_send_duration = wlt_2_send_start.elapsed();

        report.write_duration(wlt_1_send_duration);
        report.write_duration(wlt_2_send_duration);
        report.end_line();
    }
    let elapsed = now.elapsed();
    println!("elapsed: {elapsed:.2?}");
}
