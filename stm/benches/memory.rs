// Copyright 2020-2021 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

use criterion::Criterion;
use stronghold_stm as stm;
use tokio::runtime::Runtime;

/// Primitve benchmark
pub fn bnc_memory_usize(c: &mut Criterion) {
    c.bench_function("bench_memory", |b| {
        let var = stm::TVar::new(8usize);

        b.to_async(Runtime::new().expect("")).iter(move || {
            let v2 = var.clone();

            stm::transactional(move |tx| {
                tx.write(234, &v2.clone())?;
                Ok(())
            })
        })
    });
}
