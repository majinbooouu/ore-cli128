use std::{sync::Arc, time::Instant};

use drillx::equix;
use solana_rpc_client::spinner;

use crate::{args::BenchmarkArgs, Miner};

const TEST_DURATION: i64 = 30;
const HARDCODED_CORES: u64 = 128; // 코어 수 하드코딩

impl Miner {
    pub async fn benchmark(&self, _args: BenchmarkArgs) {
        // Check num threads (하드코딩된 코어 수 사용)
        self.check_num_cores(HARDCODED_CORES);

        // Dispatch job to each thread
        let challenge = [0; 32];
        let progress_bar = Arc::new(spinner::new_progress_bar());
        progress_bar.set_message(format!(
            "Benchmarking. This will take {} sec...",
            TEST_DURATION
        ));
        
        // 하드코딩된 코어 수만큼 쓰레드를 생성
        let handles: Vec<_> = (0..HARDCODED_CORES)
            .map(|i| {
                std::thread::spawn({
                    move || {
                        let timer = Instant::now();
                        let first_nonce = u64::MAX
                            .saturating_div(HARDCODED_CORES) // 하드코딩된 코어 수로 나눔
                            .saturating_mul(i as u64); // 각 코어에 논스 분배
                        let mut nonce = first_nonce;
                        let mut memory = equix::SolverMemory::new();
                        loop {
                            // Pin to core (하드코딩된 i 값 사용)
                            // 실제 코어에 핀닝하는 대신 이 작업을 수행하지 않음
                            // 코어에 할당된 작업을 실행

                            // Create hash
                            let _hx = drillx::hash_with_memory(
                                &mut memory,
                                &challenge,
                                &nonce.to_le_bytes(),
                            );

                            // Increment nonce
                            nonce += 1;

                            // Exit if time has elapsed
                            if (timer.elapsed().as_secs() as i64).ge(&TEST_DURATION) {
                                break;
                            }
                        }

                        // Return hash count
                        nonce - first_nonce
                    }
                })
            })
            .collect();

        // Join handles and return total nonce
        let mut total_nonces = 0;
        for h in handles {
            if let Ok(count) = h.join() {
                total_nonces += count;
            }
        }

        // Update log
        progress_bar.finish_with_message(format!(
            "Hashpower: {} H/sec",
            total_nonces.saturating_div(TEST_DURATION as u64),
        ));
    }
}