pub mod utils;

use utils::*;

type TT = TransferType;
type DT = DescriptorType;

#[cfg(feature = "memprof")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

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
    let ts = OffsetDateTime::unix_timestamp(OffsetDateTime::now_utc()).to_string();
    let fname = format!("back_and_forth-{ts}");
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

#[test]
#[ignore = "run if desired"]
fn random_transfers() {
    // atomic bool to gracefully handle termination requestes
    let term = Arc::new(AtomicBool::new(false));
    // register for SIGINT (Ctrl-c) and SIGTERM
    register(SIGINT, Arc::clone(&term)).unwrap();
    register(SIGTERM, Arc::clone(&term)).unwrap();

    let default_load_file = s!("");
    let load_id: String = std::env::var("LOAD_ID").unwrap_or(default_load_file);
    if !load_id.is_empty() {
        std::env::set_var("SKIP_INIT", s!("1"));
    }

    initialize();

    // CSV report helper
    fn write_row(row: &[impl ToString], file: &mut std::fs::File) {
        let row_str: Vec<String> = row.iter().map(ToString::to_string).collect();
        file.write_all(format!("{}\n", row_str.join(";")).as_bytes())
            .unwrap();
    }

    // CPU timer
    // - helpers
    fn get_cpu_time() -> (Duration, Duration) {
        let mut usage = libc::rusage {
            ru_utime: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ru_stime: libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            ..unsafe { std::mem::zeroed() }
        };

        unsafe {
            libc::getrusage(libc::RUSAGE_SELF, &mut usage);
        }

        let user = Duration::new(
            usage.ru_utime.tv_sec as u64,
            (usage.ru_utime.tv_usec * 1000) as u32,
        );
        let sys = Duration::new(
            usage.ru_stime.tv_sec as u64,
            (usage.ru_stime.tv_usec * 1000) as u32,
        );
        (user, sys)
    }
    fn get_cpu_usage(
        cpu_start: (Duration, Duration),
        cpu_end: (Duration, Duration),
        wall_elapsed: Duration,
    ) -> (Duration, Duration, f64) {
        let delta_usr = cpu_end.0 - cpu_start.0;
        let delta_sys = cpu_end.1 - cpu_start.1;
        let delta_tot = delta_usr + delta_sys;
        let percent = (delta_tot.as_secs_f64() / wall_elapsed.as_secs_f64()) * 100.0;
        (delta_usr, delta_sys, percent)
    }

    // memory sampler
    // - data structure
    #[cfg(feature = "memprof")]
    enum SamplerMessage {
        StartSampling,
        NewIteration,
        Stop,
    }
    // - helpers
    #[cfg(feature = "memprof")]
    fn flush_allocator() {
        // try to release free memory pages back to the kernel
        unsafe {
            libc::malloc_trim(0);
        }
    }

    // network stats
    // - data structures
    #[derive(Debug, Clone, Copy)]
    struct NetStat {
        rx_bytes: u64,
        tx_bytes: u64,
    }
    type NetStats = HashMap<String, NetStat>;
    #[derive(Debug, Clone, Copy, Default)]
    pub struct TcpStats {
        pub active_opens: i128,  // connections opened
        pub passive_opens: i128, // connections accepted
        pub attempt_fails: i128, // failed connection attempts
        pub estab_resets: i128,  // established connections reset
        pub in_segs: i128,       // segments received
        pub out_segs: i128,      // segments sent
        pub retrans_segs: i128,  // segments retransmitted
        pub in_errs: i128,       // bad segments received
        pub out_rsts: i128,      // reset segments sent
    }
    #[derive(Debug, Clone, Copy, Default)]
    pub struct UdpStats {
        pub in_datagrams: i128,   // datagrams received
        pub no_ports: i128,       // datagrams to unknown port received
        pub in_errors: i128,      // datagram receive errors
        pub out_datagrams: i128,  // datagrams sent
        pub rcvbuf_errors: i128,  // receive buffer errors
        pub sndbuf_errors: i128,  // send buffer errors
        pub in_csum_errors: i128, // checksum errors
        pub ignored_multi: i128,  // ignored multicast
    }
    // - helpers
    fn get_network_stats() -> NetStats {
        let mut stats = HashMap::new();
        let content = std::fs::read_to_string("/proc/net/dev").unwrap();
        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let interface = parts[0].trim_end_matches(':');
            let (rx_bytes, tx_bytes) = (
                parts[1].parse::<u64>().unwrap(),
                parts[9].parse::<u64>().unwrap(),
            );
            stats.insert(interface.to_string(), NetStat { rx_bytes, tx_bytes });
        }
        stats
    }
    fn process_network_stats(
        start: &NetStats,
        end: &NetStats,
        interface: Option<&String>,
        print: bool,
        mut file: Option<&mut std::fs::File>,
    ) {
        if print {
            println!("network stats:");
        }
        let mut tot_rx = 0u64;
        let mut tot_tx = 0u64;
        for (iface, after) in end {
            let mut write = false;
            if let Some(i) = interface {
                if iface != i {
                    continue; // process only the provided interface
                } else {
                    write = true;
                }
            }
            if let Some(before) = start.get(iface) {
                let rx_diff = after.rx_bytes - before.rx_bytes;
                let tx_diff = after.tx_bytes - before.tx_bytes;
                if rx_diff > 0 || tx_diff > 0 {
                    if write {
                        if let Some(f) = file.take() {
                            write_row(&[rx_diff, tx_diff], f);
                        }
                    }
                    if print {
                        println!(
                            "  - {:10} -> RX {:10} B, TX {:10} B",
                            iface, rx_diff, tx_diff
                        );
                    }
                    tot_rx += rx_diff;
                    tot_tx += tx_diff;
                }
            }
        }
        if print {
            println!(
                "  - totals     -> RX {:9.2} KB, TX {:9.2} KB",
                tot_rx as f64 / 1024.0,
                tot_tx as f64 / 1024.0
            );
        }
    }
    fn parse_snmp(prefix: &str) -> HashMap<String, i128> {
        let content = std::fs::read_to_string("/proc/net/snmp").unwrap();
        let mut result = HashMap::new();
        let mut headers: Vec<&str> = vec![];
        let mut values: Vec<&str> = vec![];
        for line in content.lines().filter(|l| l.starts_with(prefix)) {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if tokens[1].chars().any(|c| c.is_alphabetic()) {
                headers = line.split_whitespace().skip(1).collect();
            } else {
                values = line.split_whitespace().skip(1).collect();
            }
        }
        for (header, value) in headers.iter().zip(values.iter()) {
            result.insert(header.to_string(), value.parse::<i128>().unwrap());
        }
        result
    }
    pub fn get_tcp_stats() -> TcpStats {
        let mut stats = TcpStats::default();
        let snmp_tcp = parse_snmp("Tcp:");
        stats.active_opens = *snmp_tcp.get("ActiveOpens").unwrap_or(&0);
        stats.passive_opens = *snmp_tcp.get("PassiveOpens").unwrap_or(&0);
        stats.attempt_fails = *snmp_tcp.get("AttemptFails").unwrap_or(&0);
        stats.estab_resets = *snmp_tcp.get("EstabResets").unwrap_or(&0);
        stats.in_segs = *snmp_tcp.get("InSegs").unwrap_or(&0);
        stats.out_segs = *snmp_tcp.get("OutSegs").unwrap_or(&0);
        stats.retrans_segs = *snmp_tcp.get("RetransSegs").unwrap_or(&0);
        stats.in_errs = *snmp_tcp.get("InErrs").unwrap_or(&0);
        stats.out_rsts = *snmp_tcp.get("OutRsts").unwrap_or(&0);
        stats
    }
    pub fn get_udp_stats() -> UdpStats {
        let mut stats = UdpStats::default();
        let snmp_udp = parse_snmp("Udp:");
        stats.in_datagrams = *snmp_udp.get("InDatagrams").unwrap_or(&0);
        stats.no_ports = *snmp_udp.get("NoPorts").unwrap_or(&0);
        stats.in_errors = *snmp_udp.get("InErrors").unwrap_or(&0);
        stats.out_datagrams = *snmp_udp.get("OutDatagrams").unwrap_or(&0);
        stats.rcvbuf_errors = *snmp_udp.get("RcvbufErrors").unwrap_or(&0);
        stats.sndbuf_errors = *snmp_udp.get("SndbufErrors").unwrap_or(&0);
        stats.in_csum_errors = *snmp_udp.get("InCsumErrors").unwrap_or(&0);
        stats.ignored_multi = *snmp_udp.get("IgnoredMulti").unwrap_or(&0);
        stats
    }
    fn compute_tcp_delta(start: &TcpStats, end: &TcpStats) -> TcpStats {
        TcpStats {
            active_opens: end.active_opens - start.active_opens,
            passive_opens: end.passive_opens - start.passive_opens,
            attempt_fails: end.attempt_fails - start.attempt_fails,
            estab_resets: end.estab_resets - start.estab_resets,
            in_segs: end.in_segs - start.in_segs,
            out_segs: end.out_segs - start.out_segs,
            retrans_segs: end.retrans_segs - start.retrans_segs,
            in_errs: end.in_errs - start.in_errs,
            out_rsts: end.out_rsts - start.out_rsts,
        }
    }
    fn compute_udp_delta(start: &UdpStats, end: &UdpStats) -> UdpStats {
        UdpStats {
            in_datagrams: end.in_datagrams - start.in_datagrams,
            no_ports: end.no_ports - start.no_ports,
            in_errors: end.in_errors - start.in_errors,
            out_datagrams: end.out_datagrams - start.out_datagrams,
            rcvbuf_errors: end.rcvbuf_errors - start.rcvbuf_errors,
            sndbuf_errors: end.sndbuf_errors - start.sndbuf_errors,
            in_csum_errors: end.in_csum_errors - start.in_csum_errors,
            ignored_multi: end.ignored_multi - start.ignored_multi,
        }
    }
    fn print_tcp_stats(delta: &TcpStats) {
        println!("TCP stats:");
        println!("  - conns in {}", delta.passive_opens);
        println!("  - conns out {}", delta.active_opens);
        println!("  - conn fails {}", delta.attempt_fails);
        println!("  - packets in {}", delta.in_segs);
        println!("  - packets bad {}", delta.in_errs);
        println!("  - packets out {}", delta.out_segs);
        println!("  - packet retrans {}", delta.retrans_segs);
        println!("  - resets received {}", delta.estab_resets);
        println!("  - resets sent {}", delta.out_rsts);
    }
    fn print_udp_stats(delta: &UdpStats) {
        println!("UDP stats:");
        println!("  - packets in {}", delta.in_datagrams);
        println!("  - packets out {}", delta.out_datagrams);
        println!("  - dropped {}", delta.no_ports);
        println!("  - in total errs {}", delta.in_errors);
        println!("  - in checksum errs {}", delta.in_csum_errors);
        println!("  - rcvbuf errs {}", delta.rcvbuf_errors);
        println!("  - sndbuf errs {}", delta.sndbuf_errors);
        println!("  - ignored multicast {}", delta.ignored_multi);
    }
    fn write_tcp_stats(delta: &TcpStats, file: &mut std::fs::File) {
        let row = [
            delta.passive_opens,
            delta.active_opens,
            delta.attempt_fails,
            delta.in_segs,
            delta.in_errs,
            delta.out_segs,
            delta.retrans_segs,
            delta.estab_resets,
            delta.out_rsts,
        ];
        write_row(&row, file);
    }
    fn write_udp_stats(delta: &UdpStats, file: &mut std::fs::File) {
        let row = [
            delta.in_datagrams,
            delta.out_datagrams,
            delta.no_ports,
            delta.in_errors,
            delta.in_csum_errors,
            delta.rcvbuf_errors,
            delta.sndbuf_errors,
            delta.ignored_multi,
        ];
        write_row(&row, file);
    }

    // disk stats
    // - data structure
    #[derive(Debug, Clone, Copy)]
    struct DiskStat {
        read_bytes: u64,
        write_bytes: u64,
    }
    type DiskStats = HashMap<String, DiskStat>;
    // - helper
    fn get_disk_stats() -> DiskStats {
        let mut stats = HashMap::new();
        if let Ok(content) = std::fs::read_to_string("/proc/diskstats") {
            for line in content.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 14 {
                    let device = parts[2];
                    if let (Ok(read_sectors), Ok(write_sectors)) =
                        (parts[5].parse::<u64>(), parts[9].parse::<u64>())
                    {
                        // Convert (512-byte) sectors to bytes
                        stats.insert(
                            device.to_string(),
                            DiskStat {
                                read_bytes: read_sectors * 512,
                                write_bytes: write_sectors * 512,
                            },
                        );
                    }
                }
            }
        }
        stats
    }
    fn compute_disk_deltas(
        start: &DiskStats,
        end: &DiskStats,
        device: Option<&std::string::String>,
    ) -> DiskStats {
        let mut stats: DiskStats = HashMap::new();
        for (dev, after) in end {
            if let Some(d) = device {
                if dev != d {
                    continue; // process only the provided device
                }
            }
            let before = start.get(dev).unwrap();
            let read_diff = after.read_bytes - before.read_bytes;
            let write_diff = after.write_bytes - before.write_bytes;
            stats.insert(
                dev.to_string(),
                DiskStat {
                    read_bytes: read_diff,
                    write_bytes: write_diff,
                },
            );
        }
        stats
    }
    fn process_disk_deltas(deltas: &DiskStats, print: bool, mut file: Option<&mut std::fs::File>) {
        if print {
            println!("disk stats:");
        }
        let mut tot_read = 0u64;
        let mut tot_write = 0u64;
        let write = deltas.len() == 1 && file.is_some(); // write to file if single device selected
        for (dev, stat) in deltas {
            if write {
                write_row(&[stat.read_bytes, stat.write_bytes], file.take().unwrap());
            }
            if print {
                println!(
                    "  - {:10} -> read {:10} B, write {:10} B",
                    dev, stat.read_bytes, stat.write_bytes
                );
            }
            tot_read += stat.read_bytes;
            tot_write += stat.write_bytes;
        }
        if print {
            println!(
                "  - totals     -> read {:9.2} KB, write {:9.2} KB",
                tot_read as f64 / 1024.0,
                tot_write as f64 / 1024.0
            );
        }
    }

    // test data structures
    // - asset map: asset index -> (contract ID, contract schema)
    type Assets = BTreeMap<u8, (ContractId, String)>;
    // - contract balances map: contract index -> (wallet ID -> RGB balance)
    type Balances = BTreeMap<u8, BTreeMap<usize, u64>>;
    // - contract transfer map: contract index -> number of transfers
    type ContractTransferMap = BTreeMap<u8, u16>;
    // - UTXO map: outpoint -> BTC amount (sats), unique index
    type UTXOMap = HashMap<Outpoint, (Sats, usize)>;
    // - wallet map: index -> wallet, descriptor type, seed
    type WalletsType = HashMap<usize, (RefCell<TestWallet>, Vec<u8>)>;
    // - wallet holder, with helper methods
    struct Wallets {
        // infex for the next wallet
        next_wallet_idx: usize,
        // Asset map
        pub assets: Assets,
        // Balance map
        pub balances: Balances,
        // Wallet map
        wallets: WalletsType,
        // map of wallet index -> UTXO map
        outpoints: HashMap<usize, UTXOMap>,
    }
    #[derive(Serialize, Deserialize)]
    struct WalletsData {
        // Asset map
        assets: Assets,
        // Balance map
        balances: Balances,
        // map of wallet index -> descriptor type, seed
        wallets: HashMap<usize, Vec<u8>>,
        // map of wallet index -> UTXO map
        outpoints: HashMap<usize, UTXOMap>,
    }
    fn _get_save_fname(id: &str) -> PathBuf {
        let mut fname = PathBuf::from(TEST_DATA_DIR)
            .join(STRESS_DATA_DIR)
            .join(SAVE_DIR)
            .join(id);
        fname.set_extension("json");
        fname
    }
    impl Wallets {
        fn new() -> Self {
            Wallets {
                next_wallet_idx: 0,
                assets: BTreeMap::new(),
                balances: BTreeMap::new(),
                wallets: HashMap::new(),
                outpoints: HashMap::new(),
            }
        }

        fn load(&mut self, id: &str) {
            // reconstruct filename from id (see save)
            let fname = _get_save_fname(id);
            // load wallets data from file
            let json = std::fs::read_to_string(fname).unwrap();
            let wallets_data: WalletsData = serde_json::from_str(&json).unwrap();
            // recreate Wallets from data
            let wallets: WalletsType = wallets_data
                .wallets
                .iter()
                .map(|(idx, seed)| {
                    let xpriv_account = XprivAccount::with_seed(true, seed).derive(h![86, 1, 0]);
                    let fingerprint = xpriv_account.account_fp().to_string();
                    let wallet_dir = PathBuf::from(TEST_DATA_DIR)
                        .join(INTEGRATION_DATA_DIR)
                        .join(fingerprint);
                    let wallet = get_wallet_internal(
                        None,
                        Network::Regtest,
                        wallet_dir,
                        WalletAccount::Private(xpriv_account),
                        INSTANCE_1,
                        false,
                    );
                    (*idx, (RefCell::new(wallet), seed.clone()))
                })
                .collect();
            // restore data
            self.next_wallet_idx = wallets.len();
            self.assets = wallets_data.assets;
            self.balances = wallets_data.balances;
            self.wallets = wallets;
            self.outpoints = wallets_data.outpoints;
        }

        fn save(&self, id: &str) -> String {
            // generate file name
            // - integration data dir / wallet_saves / id (param) .json
            let fname = _get_save_fname(id);
            // save wallets data to file
            let wallets: HashMap<usize, Vec<u8>> = self
                .wallets
                .iter()
                .map(|(idx, (_, seed))| (*idx, seed.clone()))
                .collect();
            let wallets_data = WalletsData {
                assets: self.assets.clone(),
                balances: self.balances.clone(),
                wallets,
                outpoints: self.outpoints.clone(),
            };
            let json = serde_json::to_string(&wallets_data).unwrap();
            std::fs::create_dir_all(fname.parent().unwrap()).unwrap();
            std::fs::write(&fname, json).unwrap();
            // return filename as string
            fname.to_string_lossy().to_string()
        }

        fn len(&self) -> usize {
            self.wallets.len()
        }

        fn add(&mut self, wallet: TestWallet, seed: Vec<u8>) {
            self.wallets
                .insert(self.next_wallet_idx, (RefCell::new(wallet), seed));
            self.outpoints.insert(self.next_wallet_idx, HashMap::new());
            self.next_wallet_idx = self.len();
        }

        fn get_mut(&mut self, idx: usize) -> &mut TestWallet {
            self.wallets.get_mut(&idx).unwrap().0.get_mut()
        }

        fn add_outpoints(&mut self, idx: usize) {
            let wallet = self.wallets.get_mut(&idx).unwrap().0.get_mut();
            let utxos = wallet.get_unspents();
            let outpoints = self.outpoints.get_mut(&idx).unwrap();
            let idx_begin = outpoints.len();
            let mut idx_next = idx_begin;
            let mut new_outpoints = vec![];
            for (outpoint, sats) in &utxos {
                if outpoints.get(outpoint).is_none() {
                    idx_next += 1;
                    outpoints.insert(*outpoint, (*sats, idx_next));
                    new_outpoints.push((*outpoint, *sats, idx_next));
                }
            }
            let deterministic = if idx_next - idx_begin > 1 {
                let new_outpoint_txid_set: HashSet<&Txid> =
                    new_outpoints.iter().map(|(o, ..)| &o.txid).collect();
                let new_outpoint_sat_set: HashSet<&Sats> =
                    new_outpoints.iter().map(|(_, s, _)| s).collect();
                // if only a single TX is involved, sorting is deterministic due to vout
                // if multiple TXs are involved and sats are different, sorting is deterministic
                new_outpoint_txid_set.len() == 1 || new_outpoint_sat_set.len() == 1
            } else {
                // if only 1 outpoint has been added, sorting is deterministic
                true
            };
            if !deterministic {
                panic!("outpoint sorting may not be deterministic");
            }
        }

        fn del_outpoints(&mut self, idx: usize) {
            let wallet = self.wallets.get_mut(&idx).unwrap().0.get_mut();
            let utxos = wallet.get_unspents();
            let outpoints = self.outpoints.get_mut(&idx).unwrap();
            outpoints.retain(|o, _| utxos.contains_key(o));
            let idx_begin = outpoints.len();
            let mut idx_next = idx_begin;
            for (outpoint, sats) in &utxos {
                idx_next += 1;
                outpoints.entry(*outpoint).or_insert((*sats, idx_next));
            }
        }

        fn get_outpoint_map(&self, idx: usize) -> HashMap<Outpoint, (Sats, usize)> {
            self.outpoints.get(&idx).unwrap().clone()
        }

        fn get_outpoint_iter(
            &self,
            idx: usize,
        ) -> impl Iterator<Item = (Outpoint, Sats, usize)> + '_ {
            self.outpoints
                .get(&idx)
                .unwrap()
                .iter()
                .map(|(o, (s, i))| (*o, *s, *i))
        }

        fn print_debug_info(&mut self) {
            let outpoints = self.outpoints.clone();
            for i in 0..self.next_wallet_idx {
                let wallet = self.get_mut(i);
                let outpoint_map = &outpoints.get(&i).unwrap();
                eprintln!("- wallet {i}:");
                eprintln!("  - outpoints:");
                for (o, (s, i)) in *outpoint_map {
                    eprintln!("    - {o} -> {s} {i}");
                }
                eprintln!("  - utxos:");
                for u in wallet.utxos() {
                    eprintln!("    - {}: {} {}", u.outpoint, u.value, u.status.is_mined());
                }
                eprintln!("  - contracts:");
                for a in wallet.list_contracts() {
                    eprintln!("    - {}: {}", a.id, a.schema_id);
                }
            }
        }
    }

    // loop data structures
    // - contract allocation list: outpoint, RGB amount
    type Allocations = Vec<(Outpoint, u64)>;
    // - contract (being sent) remaining balance map: contract idx -> remaining balance
    type ContractReMap = BTreeMap<u8, u64>;
    // - recipient list: wallet ID, contract idx, transfer type, amount
    type Recipients = Vec<(usize, u8, TransferType, u64)>;
    // - contract allocations map: contract ID -> allocation list
    type ContractAllocMap = BTreeMap<ContractId, Allocations>;
    // - input RGB amount map: contract ID -> RGB amount
    type InputRGBAmountMap = BTreeMap<ContractId, u64>;
    // - input outpoint map: outpoint -> optional contract ID
    type InputOutpointMap = HashMap<Outpoint, Option<ContractId>>;
    // - asset coloring info map: contract ID -> AssetColoringInfo
    type AssetInfoMap = HashMap<ContractId, AssetColoringInfo>;
    // - assignment map: recipient idx -> AssetDestination
    type AssignmentMap = BTreeMap<usize, Vec<AssetDestination>>;

    // test helper functions
    // - choose the first wallet with some asset balance as sender, gathering contract states
    fn choose_sender(
        wallet_idxs: &[usize],
        wallets: &mut Wallets,
        rng: &mut StdRng,
    ) -> (usize, ContractReMap, UTXOMap, ContractAllocMap) {
        let mut send_idx: Option<usize> = None;
        let mut contract_re_map: ContractReMap = BTreeMap::new();
        for wi in wallet_idxs {
            let wallet = wallets.get_mut(*wi);
            // sort contracts deterministically using the RNG
            let mut contracts = wallet.list_contracts();
            contracts.sort_by(|a, b| a.issued_at.cmp(&b.issued_at));
            contracts.shuffle(rng);
            // build contract map
            for c in contracts {
                let (cidx, ..) = wallets
                    .assets
                    .iter()
                    .find(|(_, (cid, _))| *cid == c.id)
                    .unwrap();
                let bal = wallets.balances.get(cidx).unwrap().get(wi).unwrap();
                if *bal > 0 {
                    contract_re_map.insert(*cidx, *bal);
                }
            }
            // choose this wallet if it has any contract with balance
            if !contract_re_map.is_empty() {
                send_idx = Some(*wi);
                break;
            }
        }
        // make sure a sender has been chosen
        let send_idx = send_idx.expect("at least one wallet must have spendable assets");
        // get UTXO set
        let utxos = wallets.get_outpoint_map(send_idx);
        // get RGB allocation map
        let mut contract_allocations: ContractAllocMap = BTreeMap::new();
        let assets = wallets.assets.clone();
        let wallet = wallets.get_mut(send_idx);
        for cidx in contract_re_map.keys() {
            // - get the contract state, if available
            let contract_id = assets.get(cidx).unwrap().0;
            if let Ok(contract_state) = wallet.wallet.stock().contract_state(contract_id) {
                // - get the unspent contract fungible allocations from the state,
                //   summing amounts from multiple allocations on the same UTXO
                let mut allocations: Allocations = vec![];
                contract_state
                    .fungible_all()
                    .filter(|cf| utxos.contains_key(&cf.seal.outpoint().unwrap()))
                    .for_each(|cf| {
                        let outpoint = cf.seal.outpoint().unwrap();
                        let amt = cf.state.as_u64();
                        if let Some(allocation) = allocations.iter_mut().find(|a| a.0 == outpoint) {
                            allocation.1 += amt;
                        } else {
                            allocations.push((outpoint, amt));
                        }
                    });
                // - sort allocations by the asset amount
                let mut allocs_with_outpoint_index: Vec<(Outpoint, u64, usize)> = allocations
                    .iter()
                    .map(|(o, a)| (*o, *a, utxos[o].1))
                    .collect();
                allocs_with_outpoint_index.sort_by(|(_, a_amt, a_idx), (_, b_amt, b_idx)| {
                    a_amt.cmp(b_amt).then_with(|| a_idx.cmp(b_idx))
                });
                let allocations: Vec<(Outpoint, u64)> = allocs_with_outpoint_index
                    .iter()
                    .map(|(o, a, _)| (*o, *a))
                    .collect();
                contract_allocations.insert(contract_id, allocations);
            }
        }
        // return sender info
        (
            send_idx,
            contract_re_map,
            utxos.clone(),
            contract_allocations,
        )
    }
    // - choose the recipients (max 3), contract, amount and transfer type
    fn choose_recipients(
        wallet_idxs: &[usize],
        contract_re_map: &mut ContractReMap,
        transfer_map: &mut ContractTransferMap,
        rng: &mut StdRng,
    ) -> Recipients {
        // single recipient with 80% probability, multiple (random between 1 and max) with 20% probability
        let max_recipients = wallet_idxs.len().min(3);
        let num_recipients = if rng.gen_bool(0.8) {
            1
        } else {
            rng.gen_range(1..=max_recipients)
        };
        let mut recipients: Recipients = vec![];
        for i in 1..=num_recipients {
            // random wallet, possibly including the sender or duplicates
            let recv_idx = rng.gen_range(0..wallet_idxs.len());
            // random contract with available remaining balance
            let cidxs: Vec<u8> = contract_re_map
                .iter()
                .filter(|(_, r)| **r > 0)
                .map(|(c, _)| *c)
                .collect();
            let pick = rng.gen_range(0..cidxs.len());
            let cidx = *cidxs.get(pick).unwrap();
            // random transfer type
            let transfer_type = if rng.gen_bool(0.5) {
                TransferType::Witness
            } else {
                TransferType::Blinded
            };
            // random send amount (send all remaining to last recipient with 10% probability)
            let remaining = contract_re_map.get_mut(&cidx).unwrap();
            let divisor = if i == num_recipients && rng.gen_bool(0.1) {
                1
            } else {
                rng.gen_range(num_recipients as u64..=10)
            };
            let frac = max(1, *remaining / divisor);
            // update contract send amount and remaining balance and add the new recipient
            *remaining -= frac;
            recipients.push((recv_idx, cidx, transfer_type, frac));
            *transfer_map.entry(cidx).or_insert(0) += 1;
        }
        recipients
    }
    // - compute required input BTC amount
    fn get_required_btc(fee: &Sats, recipients: &Recipients, sats_send: u64) -> u64 {
        // compute required BTC
        let mut required_sats = fee.sats();
        for (_, _, transfer_type, _) in recipients {
            if matches!(transfer_type, TransferType::Witness) {
                required_sats += sats_send;
            }
        }
        required_sats
    }
    // - input selection
    #[allow(clippy::too_many_arguments)]
    fn select_inputs_change_method(
        recipients: &Recipients,
        contract_allocations: &ContractAllocMap,
        sats_info: (u64, u64), // required, new UTXO size
        utxos: &UTXOMap,
        send_idx: usize,
        wallets: &mut Wallets,
        rng: &mut StdRng,
    ) -> (
        InputOutpointMap,
        u64,
        Vec<(Outpoint, Sats)>,
        Option<Outpoint>,
        CloseMethod,
    ) {
        // compute per-contract required amount
        let mut target_rgb_amounts: InputRGBAmountMap = BTreeMap::new();
        recipients.iter().for_each(|(_, cidx, _, amt)| {
            let cid = wallets.assets.get(cidx).unwrap().0;
            *target_rgb_amounts.entry(cid).or_insert(0) += amt;
        });
        // select RGB inputs
        let mut input_outpoints: InputOutpointMap = HashMap::new();
        let mut input_rgb_amounts: InputRGBAmountMap = BTreeMap::new();
        let mut change = false;
        for (cid, target_rgb_amount) in &target_rgb_amounts {
            let allocations = contract_allocations.get(cid).unwrap();
            let input_rgb_amount = input_rgb_amounts.entry(*cid).or_insert(0);
            for (outpoint, amount) in allocations {
                if *input_rgb_amount >= *target_rgb_amount {
                    break;
                }
                input_outpoints.insert(outpoint.to_owned(), Some(*cid));
                *input_rgb_amount += *amount;
            }
            assert!(*input_rgb_amount >= *target_rgb_amount);
            if *input_rgb_amount > *target_rgb_amount {
                change = true
            }
        }
        // compute input BTC amount
        let mut input_sats = 0;
        for (outpoint, ..) in &input_outpoints {
            let sats = utxos.iter().find(|u| u.0 == outpoint).unwrap().1 .0;
            input_sats += sats.sats();
        }
        // add more inputs to cover the BTC required amount, if needed
        let mut remaining_utxos: Vec<(&Outpoint, &Sats, usize)> = utxos
            .iter()
            .filter(|(o, _)| !input_outpoints.contains_key(o))
            .map(|(o, (s, i))| (o, s, *i))
            .collect();
        // sort by sat amount and unique index (as tie-breaker) to avoid non-deterministic sort
        remaining_utxos.sort_by(|(_, a_sats, a_idx), (_, b_sats, b_idx)| {
            a_sats
                .sats()
                .cmp(&b_sats.sats())
                .then_with(|| a_idx.cmp(b_idx))
        });
        //remaining_utxos.sort_by(|(_, a_sats), (_, b_sats)| a_sats.sats().cmp(&b_sats.sats()));
        for (o, s, _) in remaining_utxos.iter() {
            if input_sats >= sats_info.0 {
                break;
            }
            assert!(input_outpoints.insert(**o, None).is_none());
            input_sats += s.sats();
        }
        remaining_utxos.retain(|(o, ..)| !input_outpoints.contains_key(o));
        // create a new bitcoin UTXO and add it as input, if needed to cover the BTC required amount
        if input_sats < sats_info.0 {
            let sats = sats_info.1;
            input_outpoints.insert(wallets.get_mut(send_idx).get_utxo(Some(sats)), None);
            wallets.add_outpoints(send_idx);
        };
        // choose a UTXO for change, if needed
        let change_utxo = if change {
            // choose a random available UTXO (if any) with 80% probability
            if !remaining_utxos.is_empty() && rng.gen_bool(0.8) {
                let change_utxo_idx = rng.gen_range(0..remaining_utxos.len());
                Some(*remaining_utxos[change_utxo_idx].0)
            } else {
                None
            }
        } else {
            None
        };
        // select closing method: use opret if no change and no tapret recipients
        let tr_recipients = recipients
            .iter()
            .any(|(r, ..)| wallets.get_mut(*r).close_method() == CloseMethod::TapretFirst);
        let close_method = if !change && !tr_recipients {
            CloseMethod::OpretFirst
        } else {
            wallets.get_mut(send_idx).close_method()
        };
        // return selected inputs and related info
        let remaining_utxos: Vec<(Outpoint, Sats)> = remaining_utxos
            .into_iter()
            .map(|(o, s, _)| (*o, *s))
            .collect();
        (
            input_outpoints,
            input_sats,
            remaining_utxos,
            change_utxo,
            close_method,
        )
    }
    // - construct ColoringInfo
    fn get_coloring_info(
        recipients: &Recipients,
        sats_send: u64,
        input_outpoints: &InputOutpointMap,
        wallets: &mut Wallets,
        rng: &mut StdRng,
        close_method: CloseMethod,
    ) -> (ColoringInfo, AssignmentMap) {
        let mut asset_info_map: AssetInfoMap = HashMap::new();
        let mut assignment_map: AssignmentMap = BTreeMap::new();
        for (r, cidx, tt, amt) in recipients {
            let destination = match tt {
                TransferType::Witness => AssetDestination::Witness(
                    wallets.get_mut(*r).get_witness_info(Some(sats_send), None),
                ),
                TransferType::Blinded => {
                    let utxos_being_spent: Vec<&Outpoint> = input_outpoints.keys().collect();
                    let mut usable_utxos: Vec<(Outpoint, Sats, usize)> = wallets
                        .get_outpoint_iter(*r)
                        .filter(|u| !utxos_being_spent.contains(&&u.0))
                        .collect();
                    // use a random available UTXO (if any) with 80% probability
                    let outpoint = if !usable_utxos.is_empty() && rng.gen_bool(0.8) {
                        usable_utxos.sort_by(|(_, a_sats, a_idx), (_, b_sats, b_idx)| {
                            a_sats
                                .sats()
                                .cmp(&b_sats.sats())
                                .then_with(|| a_idx.cmp(b_idx))
                        });
                        let utxo_idx = rng.gen_range(0..usable_utxos.len());
                        Some(usable_utxos[utxo_idx].0)
                    } else {
                        None
                    };
                    let ad = AssetDestination::Blinded(
                        wallets.get_mut(*r).get_secret_seal(outpoint, None),
                    );
                    if outpoint.is_none() {
                        wallets.add_outpoints(*r);
                    }
                    ad
                }
            };
            let contract_id = wallets.assets.get(cidx).unwrap().0;
            let aim = asset_info_map
                .entry(contract_id)
                .or_insert(AssetColoringInfo {
                    input_outpoints: vec![],
                    assignments: vec![],
                });
            aim.input_outpoints = input_outpoints.keys().copied().collect();
            aim.assignments.push(AssetAssignment {
                destination: destination.clone(),
                amount: *amt,
            });
            assignment_map.entry(*r).or_default().push(destination);
        }
        let coloring_info = ColoringInfo {
            asset_info_map,
            static_blinding: None,
            nonce: None,
            close_method,
        };
        (coloring_info, assignment_map)
    }

    // load and save parameters
    let mut default_loops = s!("50");
    if !load_id.is_empty() {
        // load test params from ID
        println!("\nloading test parameters");
        let mut test_params = load_id.split('-');
        std::env::set_var("SEED", test_params.next().unwrap());
        default_loops = test_params.next().unwrap().to_string();
        std::env::set_var("ASSETS", test_params.next().unwrap());
        std::env::set_var("WALLETS", test_params.next().unwrap());
    }

    // test parameters
    let seed: u64 = std::env::var("SEED")
        .unwrap_or_else(|_| rand::random::<u64>().to_string())
        .parse()
        .unwrap();
    let mut rng = StdRng::seed_from_u64(seed);
    let default_num_assets = s!("5");
    let default_num_wallets = s!("5");
    let loops: u16 = std::env::var("LOOPS")
        .unwrap_or(default_loops)
        .parse()
        .unwrap();
    if loops < 1 {
        panic!("LOOPS must be at least 1");
    }
    let num_assets: u8 = std::env::var("ASSETS")
        .unwrap_or(default_num_assets)
        .parse()
        .unwrap();
    if num_assets < 1 {
        panic!("ASSETS must be at least 1");
    }
    let num_wallets: u8 = std::env::var("WALLETS")
        .unwrap_or(default_num_wallets)
        .parse()
        .unwrap();
    if num_wallets < 1 {
        panic!("WALLETS must be at least 1");
    }
    let net_iface: Option<String> = std::env::var("NETIF").ok();
    let disk_dev: Option<String> = std::env::var("DISK").ok();
    let verbose = std::env::var("VERBOSE").is_ok();
    let fee = Sats::from_sats(DEFAULT_FEE_ABS * 2);
    let sats_send = 1000;
    let new_utxo_sats = (sats_send + fee.sats()) * 10;
    let schemas = ["CFA", "NIA"];

    // load wallets from ID, if provided
    let mut wallets = Wallets::new();
    if !load_id.is_empty() {
        // load wallets
        println!("\nloading wallets");
        wallets.load(&load_id);
        if verbose {
            wallets.print_debug_info();
        }
    }

    // parameters console log
    println!(
        "\nrunning test with: {} seed, {} loops, {} assets, {} wallets",
        seed, loops, num_assets, num_wallets
    );
    #[cfg(feature = "memprof")]
    println!("memory profiling enabled");

    // initial setup
    // - data dir and base output file
    let stress_tests_dir = PathBuf::from(TEST_DATA_DIR).join(STRESS_DATA_DIR);
    std::fs::create_dir_all(&stress_tests_dir).unwrap();
    let ts = OffsetDateTime::unix_timestamp(OffsetDateTime::now_utc()).to_string();
    let test_params_str = format!("{}-{}-{}-{}-{}", seed, loops, num_assets, num_wallets, ts);
    let fname_base = format!("random_transfers_seeded-{test_params_str}");
    println!("\nreport files:");
    // - times + consignment sizes report file
    let mut fpath_rep = stress_tests_dir.join(&fname_base);
    fpath_rep.set_extension("csv");
    println!("  - transfers   {}", fpath_rep.to_string_lossy());
    let report = Report {
        report_path: fpath_rep.clone(),
    };
    report.write_header(&[
        "sender",
        "recipients",
        "setup ms",
        "send ms",
        "mine ms",
        "validate 1 ms",
        "accept 1 ms",
        "validate 2 ms",
        "accept 2 ms",
        "validate 3 ms",
        "accept 3 ms",
        "cons 1 B",
        "cons 2 B",
        "cons 3 B",
        "sync ms",
        "sync n",
    ]);
    // - CPU usage file and start
    let fname_cpu = format!("{fname_base}_cpu");
    let mut fpath_cpu = stress_tests_dir.join(fname_cpu);
    fpath_cpu.set_extension("csv");
    println!("  - CPU usage   {}", fpath_cpu.to_string_lossy());
    let mut file_cpu = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&fpath_cpu)
        .unwrap();
    file_cpu
        .write_all("%;wall ms;usr ms;sys ms\n".to_string().as_bytes())
        .unwrap();
    let cpu_start = get_cpu_time();
    // - network data usage file and start
    let (fpath_net, mut file_net) = if let Some(i) = &net_iface {
        let fname_net = format!("{fname_base}_network_{i}");
        let mut fpath_net = stress_tests_dir.join(fname_net);
        fpath_net.set_extension("csv");
        println!("  - network stats   {}", fpath_net.to_string_lossy());
        let mut file_net = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&fpath_net)
            .unwrap();
        file_net
            .write_all("RX bytes;TX bytes\n".to_string().as_bytes())
            .unwrap();
        (Some(fpath_net), Some(file_net))
    } else {
        (None, None)
    };
    let net_start = get_network_stats();
    // - network connections files and start
    let fname_tcp = format!("{fname_base}_tcp");
    let fname_udp = format!("{fname_base}_udp");
    let mut fpath_tcp = stress_tests_dir.join(fname_tcp);
    let mut fpath_udp = stress_tests_dir.join(fname_udp);
    fpath_tcp.set_extension("csv");
    fpath_udp.set_extension("csv");
    println!("  - TCP         {}", fpath_tcp.to_string_lossy());
    println!("  - UDP         {}", fpath_udp.to_string_lossy());
    let mut file_tcp = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&fpath_tcp)
        .unwrap();
    let mut file_udp = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&fpath_udp)
        .unwrap();
    file_tcp.write_all("conns in;conns out;conn fails;packets in;packets bad;packets out;packet retrans;resets received;resets sent\n".to_string().as_bytes()).unwrap();
    file_udp.write_all("packets in;packets out;dropped;in total errs;in checksum errs;rcvbuf errs;sndbuf errs;ignored multicast\n".to_string().as_bytes()).unwrap();
    let tcp_start = get_tcp_stats();
    let udp_start = get_udp_stats();
    // - disk usage file and start
    let (fpath_dio, mut file_dio) = if let Some(i) = &disk_dev {
        let fname_dio = format!("{fname_base}_disk_{i}");
        let mut fpath_dio = stress_tests_dir.join(fname_dio);
        fpath_dio.set_extension("csv");
        println!("  - disk stats  {}", fpath_dio.to_string_lossy());
        let mut file_dio = std::fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&fpath_dio)
            .unwrap();
        file_dio
            .write_all("read bytes;write bytes\n".to_string().as_bytes())
            .unwrap();
        (Some(fpath_dio), Some(file_dio))
    } else {
        (None, None)
    };
    let disk_start = get_disk_stats();
    // - memory sampler PID, system info and file
    #[cfg(feature = "memprof")]
    let (tx_mem, sampler, fpath_mem) = {
        let pid = sysinfo::get_current_pid().unwrap();
        let mut sys = sysinfo::System::new_all();
        let fname_mem = format!("{fname_base}_memory_samples");
        let mut fpath_mem = stress_tests_dir.join(fname_mem);
        fpath_mem.set_extension("csv");
        println!("  - mem samples {}", fpath_mem.to_string_lossy());
        // - memory sampler thread spawn
        let fpath_mem_sampler = fpath_mem.clone();
        let (tx_mem, rx_mem) = crossbeam::channel::unbounded::<SamplerMessage>();
        let sampler = std::thread::spawn(move || {
            // samples file
            let mut file_mem = std::fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&fpath_mem_sampler)
                .unwrap();
            file_mem.write_all("kB\n".to_string().as_bytes()).unwrap();
            // sampling loop
            //
            // - start sampling on StartSampling
            // - start a new row on NewIteration
            // - sample memory usage while there are no messages
            // - stop sampling on stop
            // - panic if the communication channel gets disconnected
            let mut row = vec![];
            let mut sample = false;
            loop {
                match rx_mem.try_recv() {
                    Ok(SamplerMessage::StartSampling) => {
                        sample = true;
                    }
                    Ok(SamplerMessage::NewIteration) => {
                        // write the current row to the CSV + clear row
                        write_row(&row, &mut file_mem);
                        row = vec![];
                    }
                    Ok(SamplerMessage::Stop) => {
                        // write the current row to the CSV + quit
                        write_row(&row, &mut file_mem);
                        break;
                    }
                    Err(crossbeam::channel::TryRecvError::Empty) => {
                        if sample {
                            // sample current memory usage + add to current row
                            sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), false);
                            let mem_kb = sys.process(pid).map(|p| p.memory()).unwrap_or(0) / 1000;
                            row.push(mem_kb);
                        }
                        // sleep for the configured duration
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(crossbeam::channel::TryRecvError::Disconnected) => {
                        panic!("crossbeam channel disconnected")
                    }
                }
            }
        });
        (tx_mem, sampler, fpath_mem)
    };
    // - memory profiler file and start (if enabled via feature)
    //   data gets written to file when dropped at the end of the test
    #[cfg(feature = "memprof")]
    let _profiler = {
        let fname_pro = format!("{fname_base}_memory_profile");
        let mut fpath_pro = stress_tests_dir.join(fname_pro);
        fpath_pro.set_extension("json");
        println!("  - mem profile {}", fpath_pro.to_string_lossy());
        dhat::Profiler::builder().file_name(&fpath_pro).build()
    };

    // wallet setup (if not loaded)
    if load_id.is_empty() {
        println!("\ngenerating wallets");
        for _ in 0..num_wallets {
            // terminate early if requested
            if term.load(Ordering::Relaxed) {
                println!("termination requested, exiting...");
                return;
            }
            let descriptor_type = if rng.gen_bool(0.5) {
                DescriptorType::Wpkh
            } else {
                DescriptorType::Tr
            };
            let (wlt, seed) = get_wallet_and_seed(&descriptor_type, None, true, None);
            wallets.add(wlt, seed);
        }
    }

    // asset setup (if not loaded)
    let mut transfer_map: ContractTransferMap = BTreeMap::new();
    if load_id.is_empty() {
        println!("\nissuing assets");
        let issued_supply = 1000000;
        for i in 0..num_assets {
            // terminate early if requested
            if term.load(Ordering::Relaxed) {
                println!("termination requested, exiting...");
                return;
            }
            let wallet_idx = rng.gen_range(0..wallets.len());
            let wallet = wallets.get_mut(wallet_idx);
            let schema_idx = rng.gen_range(0..schemas.len());
            let schema = schemas[schema_idx];
            let asset = match schema_idx {
                0 => wallet.issue_cfa(issued_supply, None),
                1 => wallet.issue_nia(issued_supply, None),
                _ => panic!("unexpected issuance schema"),
            };
            println!("wallet {wallet_idx: >3} -> {} ({})", asset, schema);
            wallets.assets.insert(i, (asset, schema.to_owned()));
            let mut wallet_balances = BTreeMap::new();
            for w in 0..wallets.len() {
                if w == wallet_idx {
                    wallet_balances.insert(w, issued_supply);
                } else {
                    wallet_balances.insert(w, 0);
                }
            }
            wallets.balances.insert(i, wallet_balances);
            wallets.add_outpoints(wallet_idx);
            transfer_map.insert(i, 0);
        }
    }

    // transfer loop
    let start = Instant::now();
    #[cfg(feature = "memprof")]
    tx_mem.send(SamplerMessage::StartSampling).unwrap();
    for i in 1..=loops {
        // terminate early if requested
        if term.load(Ordering::Relaxed) {
            println!("termination requested, gracefully exiting...");
            break;
        }

        // flush freed memory to get better per-loop memory sampling accuracy
        #[cfg(feature = "memprof")]
        flush_allocator();

        let cpu_loop_start = get_cpu_time();
        let tcp_loop_start = get_tcp_stats();
        let udp_loop_start = get_udp_stats();
        let net_loop_start = get_network_stats();
        let disk_loop_start = get_disk_stats();
        let loop_start = Instant::now();
        if verbose {
            print!("\n--------");
        }

        // randomly order wallets
        let mut wallet_idxs: Vec<usize> = (0..wallets.len()).collect();
        wallet_idxs.shuffle(&mut rng);

        // choose the sender wallet + get its data
        let (send_idx, mut contract_re_map, utxos, contract_allocations) =
            choose_sender(&wallet_idxs, &mut wallets, &mut rng);
        report.write_displayable(send_idx);

        // choose the recipients
        let recipients = choose_recipients(
            &wallet_idxs,
            &mut contract_re_map,
            &mut transfer_map,
            &mut rng,
        );

        // print loop log
        let recipient_list = &recipients
            .iter()
            .map(|(r, c, t, a)| {
                format!(
                    "{r}({c}/{a}/{})",
                    if matches!(t, TransferType::Blinded) {
                        "b"
                    } else {
                        "w"
                    }
                )
            })
            .collect::<Vec<String>>()
            .join("|");
        report.write_displayable(recipient_list);
        println!(
            "\nloop {i:3}/{:3}: from {:2} to {:2}",
            loops, send_idx, recipient_list
        );
        if verbose {
            println!("sender data:");
            println!("  - utxos (sats):");
            for (out, (sats, _)) in &utxos {
                println!("    * {out} => {sats}");
            }
            println!("  - contract allocation map (outpoint, amount):");
            for (cid, all) in &contract_allocations {
                let (cidx, ..) = wallets.assets.iter().find(|(_, (c, _))| c == cid).unwrap();
                let a = all
                    .iter()
                    .map(|(out, amt)| format!("{out} {amt}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("    * {} => {}", cidx, a);
            }
            println!("recipient map (contract, transfer type, amount):");
            for (r, cidx, tt, amt) in &recipients {
                println!("  - {r}: {cidx} {tt} {amt}");
            }
        }

        // determine required input BTC amount
        let required_sats = get_required_btc(&fee, &recipients, sats_send);

        // select inputs
        let (input_outpoints, input_sats, remaining_utxos, change_utxo, close_method) =
            select_inputs_change_method(
                &recipients,
                &contract_allocations,
                (required_sats, new_utxo_sats),
                &utxos,
                send_idx,
                &mut wallets,
                &mut rng,
            );
        if verbose {
            println!("required sats: {required_sats}");
            println!("input sats: {input_sats}");
            println!("input outpoint map (contract):");
            for (out, cid_opt) in input_outpoints.iter() {
                let cid_s = if let Some(cid) = cid_opt {
                    let (cidx, ..) = wallets.assets.iter().find(|(_, (c, _))| c == cid).unwrap();
                    format!("{cidx}")
                } else {
                    s!("")
                };
                println!("  - {out} {cid_s}");
            }
            println!("remaining sender utxos:");
            for (out, sats) in remaining_utxos.iter() {
                println!("  - {out} {sats}");
            }
            if let Some(change_utxo) = change_utxo {
                println!("change UTXO: {change_utxo}");
            }
        }

        // construct ColoringInfo
        let (coloring_info, assignment_map) = get_coloring_info(
            &recipients,
            sats_send,
            &input_outpoints,
            &mut wallets,
            &mut rng,
            close_method,
        );

        // update expected balances
        for (r, cidx, _, amount) in &recipients {
            let asset_balances = wallets.balances.get_mut(cidx).unwrap();
            // move balance from sender to recipient
            *asset_balances.get_mut(&send_idx).unwrap() -= amount;
            *asset_balances.get_mut(r).unwrap() += amount;
        }
        if verbose {
            println!("expected balances:");
            for (cidx, wb) in &wallets.balances {
                let wallet_balances = wb
                    .iter()
                    .map(|(w, b)| format!("w{w:2}>{b:7}"))
                    .collect::<Vec<_>>();
                println!("  - contract {cidx}: {}", wallet_balances.join(" | "));
            }
        }
        let setup_duration = loop_start.elapsed();
        report.write_duration(setup_duration);

        // send assets
        let send_start = Instant::now();
        let (consignment_map, tx, _, tweak_info) = wallets.get_mut(send_idx).pay_full_flexible(
            coloring_info,
            Some(fee.sats()),
            change_utxo,
        );
        if change_utxo.is_none() {
            wallets.add_outpoints(send_idx);
        }
        let send_duration = send_start.elapsed();
        report.write_duration(send_duration);

        // mine a block + wait for TX to be confirmed in indexer
        let mine_start = Instant::now();
        let txid = tx.txid();
        wallets.get_mut(send_idx).mine_tx(&txid, false);
        let mine_duration = mine_start.elapsed();
        report.write_duration(mine_duration);

        // accept transfers
        // - accept transfers on the receiver side
        let accept_start = Instant::now();
        let mut accept_count = 0;
        let mut consignment_sizes: Vec<u64> = vec![];
        for (r, cidx, tt, _) in &recipients {
            if *r == send_idx && *tt == TransferType::Witness {
                continue; // skip sender witness transfers to self
            }
            let contract_id = wallets.assets.get(cidx).unwrap().0;
            let (_, consignment) = consignment_map
                .iter()
                .find(|(cid, _)| **cid == contract_id)
                .unwrap();
            if verbose {
                println!("accepting consignment for contract {cidx} on recipient {r}");
            }
            accept_count += 1;
            wallets
                .get_mut(*r)
                .accept_transfer(consignment.clone(), Some(&report));
            // add size of accepted consignment to list
            let mut buff: Vec<u8> = vec![];
            consignment.save(&mut buff).expect("failed saving transfer");
            consignment_sizes.push(buff.len() as u64);
        }
        while accept_count < 3 {
            report.write_displayable(0);
            report.write_displayable(0);
            accept_count += 1;
        }
        let consignment_num = consignment_sizes.len();
        consignment_sizes.iter().for_each(|s| {
            report.write_displayable(s);
        });
        for _ in 0..(3 - consignment_num) {
            report.write_displayable(0);
        }
        // - add tapret tweak to recipient if needed
        if let Some((wi, tc)) = tweak_info {
            // detect if wallet is owner of tweaked output
            let tweaked_addr = wi.derived_address.addr;
            let (tweaked_recipient, _) = assignment_map
                .iter()
                .find(|(_, asset_dests)| {
                    asset_dests.iter().any(|ad| match ad {
                        AssetDestination::Witness(wi) => wi.derived_address.addr == tweaked_addr,
                        AssetDestination::Blinded(_) => false,
                    })
                })
                .unwrap();
            if verbose {
                println!("tweaked_addr: {tweaked_addr}, tweaked_recipient: {tweaked_recipient:?}");
            }
            // add tapret tweak
            let wallet = wallets.get_mut(*tweaked_recipient);
            wallet.add_tapret_tweak(wi.terminal(), tc);
        }
        let accept_duration = accept_start.elapsed();

        // sync wallets
        let sync_start = Instant::now();
        // - sync sender (always, due to input(s) being spent)
        wallets.get_mut(send_idx).sync();
        // - sync recipients (witness only)
        let mut wallet_num = 1; // include sender
        for (r, _, tt, _) in &recipients {
            // sender synced already
            if *r != send_idx {
                wallet_num += 1;
                if *tt == TransferType::Witness {
                    wallets.get_mut(*r).sync();
                }
            }
        }
        // - add new outpoints created by witness transfers
        for (r, _, tt, _) in &recipients {
            if *tt == TransferType::Witness {
                wallets.add_outpoints(*r);
            }
        }
        // - remove spent outpoints
        wallets.del_outpoints(send_idx);
        let sync_duration = sync_start.elapsed();
        report.write_duration(sync_duration);
        report.write_displayable(wallet_num);

        // loop stats
        // - collect
        let loop_duration = loop_start.elapsed();
        let disk_loop_end = get_disk_stats();
        let net_loop_end = get_network_stats();
        let tcp_loop_end = get_tcp_stats();
        let udp_loop_end = get_udp_stats();
        let cpu_loop_end = get_cpu_time();
        // - process
        let disk_loop_deltas =
            compute_disk_deltas(&disk_loop_start, &disk_loop_end, disk_dev.as_ref());
        process_disk_deltas(&disk_loop_deltas, false, file_dio.as_mut());
        process_network_stats(
            &net_loop_start,
            &net_loop_end,
            net_iface.as_ref(),
            false,
            file_net.as_mut(),
        );
        let (cpu_delta_usr, cpu_delta_sys, cpu_percent) =
            get_cpu_usage(cpu_loop_start, cpu_loop_end, loop_duration);
        let tcp_loop_delta = compute_tcp_delta(&tcp_loop_start, &tcp_loop_end);
        let udp_loop_delta = compute_udp_delta(&udp_loop_start, &udp_loop_end);
        write_tcp_stats(&tcp_loop_delta, &mut file_tcp);
        write_udp_stats(&udp_loop_delta, &mut file_udp);
        write_row(
            &[
                cpu_percent as u128,
                loop_duration.as_millis(),
                cpu_delta_usr.as_millis(),
                cpu_delta_sys.as_millis(),
            ],
            &mut file_cpu,
        );
        report.end_line();
        // - print
        if verbose {
            println!("times:");
            println!("  - setup ms: {}", setup_duration.as_millis());
            println!("  - send ms: {}", send_duration.as_millis());
            println!("  - mine ms: {}", mine_duration.as_millis());
            println!("  - accept ms: {}", accept_duration.as_millis());
            println!("  - sync ms: {}", sync_duration.as_millis());
            println!(
                "consignment bytes: {}",
                consignment_sizes
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")
            );
            println!(
                "CPU usage: {cpu_percent:.0}%, {:?}ms usr + {:?}ms sys",
                cpu_delta_usr.as_millis(),
                cpu_delta_sys.as_millis()
            );
            println!(
                "TCP conns in/out: {}/{}",
                tcp_loop_delta.passive_opens, tcp_loop_delta.active_opens
            );
            println!(
                "TCP packets in/out: {}/{}",
                tcp_loop_delta.in_segs, tcp_loop_delta.out_segs
            );
            println!(
                "UDP packets in/out: {}/{}",
                udp_loop_delta.in_datagrams, udp_loop_delta.out_datagrams
            );
            if disk_loop_deltas.len() == 1 {
                let disk_delta = disk_loop_deltas.get(disk_dev.as_ref().unwrap()).unwrap();
                println!(
                    "disk I/O bytes: {}/{}",
                    disk_delta.read_bytes, disk_delta.write_bytes
                );
            }
        }
        println!(
            "completed transfer with txid {} in {} ms",
            &tx.txid(),
            loop_duration.as_millis()
        );

        // check that balances are the expected ones
        for (cidx, (cid, schema)) in &wallets.assets.clone() {
            let balances = wallets.balances.clone();
            let expected_balances = balances.get(cidx).unwrap();
            for w in 0..wallets.len() {
                let wallet = wallets.get_mut(w);
                let wallet_contracts = wallet.list_contracts();
                let actual_balance = if wallet_contracts.iter().any(|c| c.id == *cid) {
                    match schema.as_str() {
                        "CFA" | "NIA" => wallet.get_contract_balance(*cid),
                        _ => panic!("unexpected schema"),
                    }
                } else {
                    0 // wallet doesn't have this contract
                };
                let expected_balance = expected_balances.get(&w).unwrap();

                if actual_balance != *expected_balance {
                    wallets
                        .get_mut(send_idx)
                        .debug_logs(*cid, AllocationFilter::WalletAll);
                    panic!("actual balance ({actual_balance}) is not the expected one {expected_balance}");
                }
            }
        }

        // start a new row in the memory sampler CSV
        #[cfg(feature = "memprof")]
        tx_mem.send(SamplerMessage::NewIteration).unwrap();
    }

    // stop the memory sampler
    #[cfg(feature = "memprof")]
    {
        tx_mem.send(SamplerMessage::Stop).unwrap();
        sampler.join().unwrap();
    }

    // final report
    let elapsed = start.elapsed();
    let cpu_end = get_cpu_time();
    let tcp_end = get_tcp_stats();
    let udp_end = get_udp_stats();
    let net_end = get_network_stats();
    let disk_end = get_disk_stats();

    // save wallets
    eprintln!("\n---\nsaving wallets...");
    if verbose {
        wallets.print_debug_info();
    }
    wallets.save(&test_params_str);
    //      requires an RNG that supports rand::De/SerializeRng to save/load its state
    println!(
        "saved test wallets to {}",
        _get_save_fname(&test_params_str).to_str().unwrap()
    );

    println!("\n---\ntest run complete");
    let (cpu_delta_usr, cpu_delta_sys, cpu_percent) = get_cpu_usage(cpu_start, cpu_end, elapsed);
    let tcp_delta = compute_tcp_delta(&tcp_start, &tcp_end);
    let udp_delta = compute_udp_delta(&udp_start, &udp_end);
    let disk_deltas = compute_disk_deltas(&disk_start, &disk_end, disk_dev.as_ref());
    println!("\nsummary:");
    println!("  - total elapsed: {:.2?}s", elapsed.as_secs());
    println!("  - random seed: {seed}");
    println!("  - transfer number by contract:");
    for (cidx, n) in transfer_map {
        println!("    * {cidx}: {n} transfers");
    }
    println!("\nCPU usage:");
    println!("  - utilization: {cpu_percent:.0}%");
    println!(
        "  - usr/sys time: {:?}s/{:?}s",
        cpu_delta_usr.as_secs(),
        cpu_delta_sys.as_secs()
    );
    print_tcp_stats(&tcp_delta);
    print_udp_stats(&udp_delta);
    process_network_stats(&net_start, &net_end, net_iface.as_ref(), true, None);
    process_disk_deltas(&disk_deltas, true, None);
    println!("\nreport files:");
    println!("  - transfers   {}", fpath_rep.to_string_lossy());
    println!("  - CPU usage   {}", fpath_cpu.to_string_lossy());
    if let Some(fpath_net) = fpath_net {
        println!("  - net stats   {}", fpath_net.to_string_lossy());
    }
    println!("  - TCP         {}", fpath_tcp.to_string_lossy());
    println!("  - UDP         {}", fpath_udp.to_string_lossy());
    if let Some(fpath_dio) = fpath_dio {
        println!("  - disk stats  {}", fpath_dio.to_string_lossy());
    }
    #[cfg(feature = "memprof")]
    {
        println!("  - mem samples {}", fpath_mem.to_string_lossy());
        println!("\ndhat report:") // printed automatically when dropped
    }
}
