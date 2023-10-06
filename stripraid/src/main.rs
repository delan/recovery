use std::{fs::File, io::{self, BufReader, Read, stdout, Write, Seek}, error::Error, time::Instant};

use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator, IntoParallelRefIterator, IndexedParallelIterator};

const STRIPE: usize = 262144;
const SKIP: usize = 500000 * 1048576 / STRIPE / 12;
const SEEK: usize = SKIP * 12;

fn main() -> Result<(), Box<dyn Error>> {
    let mut out = stdout().lock();
    let mut files = std::env::args().skip(1)
        .map(File::open)
        .collect::<io::Result<Vec<_>>>()?;
    for file in files.iter_mut() {
        file.seek(io::SeekFrom::Start((SKIP * STRIPE).try_into()?))?;
    }

    let files = files.iter()
        .map(|f| BufReader::with_capacity(STRIPE, f))
        .collect::<Vec<_>>();
    let datas = files.iter()
        .map(|_| vec![0u8; STRIPE])
        .collect::<Vec<_>>();
    let mut pairs = files.into_iter()
        .zip(datas.into_iter())
        .collect::<Vec<_>>();

    let mut first_error = None;
    let mut q = pairs.len() - 2;
    let mut p = pairs.len() - 1;
    for row in 0.. {
        if row >= SKIP {
            let t1 = Instant::now();
            let results = pairs.par_iter_mut()
                .map(|(file, data)| {
                    file.read_exact(data)
                })
                .collect::<Vec<_>>();
            for result in results {
                result?; // :)
            }
            let t2 = Instant::now();
            let xor = pairs./*par_*/iter()
                .enumerate()
                .filter(|(i, _)| *i != q)
                .map(|(_, (_, data))| data)
                .cloned()
                .reduce(/*|| vec![0u8; STRIPE],*/ |p, q| {
                    let mut result = vec![0u8; STRIPE];
                    for i in 0..result.len() {
                        result[i] = p[i] ^ q[i];
                    }
                    result
                })
                .unwrap();
            if let Some((offset, _)) = xor.iter().enumerate().find(|(_, x)| **x != 0) {
                first_error = first_error.or(Some((row, offset)));
                let (first_row, first_offset) = first_error.unwrap();
                let size = first_row * STRIPE * (pairs.len() - 2) / 1024;
                eprintln!(
                    "warning: bad parity at ({}, {}); first error at ({}, {}) or logical {} KiB",
                    row, offset, first_row, first_offset, size
                );
            }
            let t3 = Instant::now();
            for (i, (_, data)) in pairs.iter().enumerate() {
                if i != p && i != q {
                    out.write_all(data)?;
                }
            }
            if row % 300 == 0 {
                eprintln!("\rrow {}: read {:?}, xor {:?}, write {:?}", row, t2 - t1, t3 - t2, Instant::now() - t3);
            }
        }
        q = if q == 0 {
            pairs.len() - 1
        } else {
            q - 1
        };
        p = if p == 0 {
            pairs.len() - 1
        } else {
            p - 1
        };
    }

    Ok(())
}
