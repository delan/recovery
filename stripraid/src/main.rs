use std::{fs::File, io::{self, BufReader, Read, stdout, Write, Seek}, error::Error, time::Instant};

use rayon::prelude::{IntoParallelRefMutIterator, ParallelIterator};

const STRIPE: usize = 262144;
const BUFREADER_SIZE: usize = 1048576;

macro_rules! warn {
    ($sink:expr, $fmt:literal $(, $arg:expr)* $(,)*) => {{
        let msg = format!($fmt $(, $arg)*);
        if let Some(sink) = $sink.as_mut() {
            writeln!(sink, "{}", msg)?;
        }
        eprintln!("{}", msg);
    }};
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut log = std::env::var("STRIPRAID_LOG").ok()
        .map(|x| File::options().read(true).write(true).create_new(true).open(x))
        .transpose()?;
    let skip_rows = std::env::var("STRIPRAID_SKIP_ROWS").unwrap_or("0".to_owned());
    let skip_rows = usize::from_str_radix(&skip_rows, 10)?;
    let raid6 = std::env::var_os("STRIPRAID_RAID5").is_none();

    let mut out = stdout().lock();
    let mut files = std::env::args().skip(1)
        .map(File::open)
        .collect::<io::Result<Vec<_>>>()?;
    for file in files.iter_mut() {
        file.seek(io::SeekFrom::Start((skip_rows * STRIPE).try_into()?))?;
    }

    let files = files.iter()
        .map(|f| BufReader::with_capacity(BUFREADER_SIZE, f))
        .collect::<Vec<_>>();
    let datas = files.iter()
        .map(|_| vec![0u8; STRIPE])
        .collect::<Vec<_>>();
    let mut pairs = files.into_iter()
        .zip(datas.into_iter())
        .collect::<Vec<_>>();

    let mut q = raid6.then(|| pairs.len() - 2);
    let mut p = pairs.len() - 1;
    for row in 0.. {
        if row >= skip_rows {
            let t1 = Instant::now();

            let results = pairs.par_iter_mut()
                .map(|(file, data)| {
                    file.read_exact(data)
                })
                .collect::<Vec<_>>();
            let mut q_failed = false;
            let mut p_failed = false;
            let mut data_failed = None;
            for (i, result) in results.iter().enumerate() {
                if let Err(error) = result {
                    // if the read failed, seek to the start of the next row.
                    // we only want to do this if the read failed, because it wipes our BufReader.
                    // per linux read(2):
                    //      On error, -1 is returned, and errno is set to indicate the error.
                    //      In this case, it is left unspecified whether the file position
                    //      (if any) changes.
                    // per rust Read::read_exact:
                    //      If this function returns an error, it is unspecified how many
                    //      bytes it has read, but it will never read more than would be
                    //      necessary to completely fill the buffer.
                    pairs[i].0.seek(io::SeekFrom::Start(((row + 1) * STRIPE).try_into()?))?;

                    let kind = if Some(i) == q {
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
                    if Some(i) == q {
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
                    .filter(|(i, _)| Some(*i) != q && *i != f)
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
                    .filter(|(i, _)| Some(*i) != q)
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
                if Some(i) != q && i != p {
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
        q = q.map(|q| if q == 0 {
            pairs.len() - 1
        } else {
            q - 1
        });
        p = if p == 0 {
            pairs.len() - 1
        } else {
            p - 1
        };
    }

    Ok(())
}
