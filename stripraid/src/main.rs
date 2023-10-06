use std::{fs::File, io::{self, BufReader, Read, stdout, Write, Seek}, error::Error, time::Instant};

use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

const STRIPE: usize = 262144;
const SKIP_ROWS: usize = 4753000 * 1048576 / STRIPE / 12;
const _YOU_SHOULD_SEEK_DD_BY_THIS_MANY_STRIPES: usize = SKIP_ROWS * 12;

macro_rules! warn {
    ($sink:expr, $fmt:literal $(, $arg:expr)* $(,)*) => {{
        let msg = format!($fmt $(, $arg)*);
        writeln!($sink, "{}", msg)?;
        eprintln!("{}", msg);
    }};
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut log = File::options()
        .read(true)
        .write(true)
        .create_new(true)
        .open(std::env::var("STRIPRAID_LOG")?)?;

    let mut out = stdout().lock();
    let mut files = std::env::args().skip(1)
        .map(File::open)
        .collect::<io::Result<Vec<_>>>()?;
    for file in files.iter_mut() {
        file.seek(io::SeekFrom::Start((SKIP_ROWS * STRIPE).try_into()?))?;
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

    let mut q = pairs.len() - 2;
    let mut p = pairs.len() - 1;
    for row in 0.. {
        if row >= SKIP_ROWS {
            let t1 = Instant::now();

            let results = pairs.par_iter_mut()
                .map(|(file, data)| {
                    // always seek before reading; this not only handles SKIP_ROWS, but also
                    // ensures that we advance to the correct position after read errors.
                    // per linux read(2):
                    //      On error, -1 is returned, and errno is set to indicate the error.
                    //      In this case, it is left unspecified whether the file position
                    //      (if any) changes.
                    // per rust Read::read_exact:
                    //      If this function returns an error, it is unspecified how many
                    //      bytes it has read, but it will never read more than would be
                    //      necessary to completely fill the buffer.
                    file.seek(io::SeekFrom::Start((row * STRIPE).try_into().unwrap()))?;
                    file.read_exact(data)
                })
                .collect::<Vec<_>>();
            let mut q_failed = false;
            let mut p_failed = false;
            let mut data_failed = None;
            for (i, result) in results.iter().enumerate() {
                if let Err(error) = result {
                    let kind = if i == q {
                        "q"
                    } else if i == p {
                        "p"
                    } else {
                        "data"
                    };
                    warn!(
                        log, "warning: {:?}\nwarning: read error at row {} disk {} ({})",
                        error, row, i, kind,
                    );
                    if i == q {
                        q_failed = true;
                    } else if i == p {
                        p_failed = true;
                    } else {
                        if data_failed.replace(i).is_some() {
                            panic!("fatal: two data read errors in the same row");
                        }
                    }
                }
            }

            let t2 = Instant::now();

            if q_failed {
                warn!(log, "warning: no action needed; q is not yet implemented");
            }

            if p_failed && data_failed.is_some() {
                panic!("fatal: p and data read errors in the same row");
            } else if let Some(f) = data_failed {
                // restore failed data from xor parity (p)
                warn!(log, "warning: restoring data from xor parity (p)");
                let xor = pairs.iter()
                    .enumerate()
                    .filter(|(i, _)| *i != q && *i != f)
                    .map(|(_, (_, data))| data)
                    .cloned()
                    .reduce(|p, q| {
                        let mut result = vec![0u8; STRIPE];
                        for i in 0..result.len() {
                            result[i] = p[i] ^ q[i];
                        }
                        result
                    })
                    .unwrap();
                pairs[f].1.copy_from_slice(&xor);
            }

            if p_failed {
                warn!(log, "warning: no action needed; we are stripping p anyway");
            } else {
                // check xor parity (p)
                let xor = pairs.iter()
                    .enumerate()
                    .filter(|(i, _)| *i != q)
                    .map(|(_, (_, data))| data)
                    .cloned()
                    .reduce(|p, q| {
                        let mut result = vec![0u8; STRIPE];
                        for i in 0..result.len() {
                            result[i] = p[i] ^ q[i];
                        }
                        result
                    })
                    .unwrap();
                if let Some((offset, _)) = xor.iter().enumerate().find(|(_, x)| **x != 0) {
                    warn!(
                        log, "warning: bad parity at row {} offset {}; data may be corrupt!",
                        row, offset,
                    );
                }
            }

            let t3 = Instant::now();

            for (i, (_, data)) in pairs.iter().enumerate() {
                if i != p && i != q {
                    out.write_all(data)?;
                }
            }
            if row % 300 == 0 {
                eprintln!(
                    "\rrow {}: read {:?}, xor {:?}, write {:?}",
                    row, t2 - t1, t3 - t2, Instant::now() - t3,
                );
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
