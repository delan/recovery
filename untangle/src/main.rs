use std::{fs::File, io::{self, BufReader, Read}, error::Error, collections::BTreeSet};

use reed_solomon_erasure::galois_8::ReedSolomon;

const SECTOR: usize = 512;
const STRIPE: usize = 262144;
const READ_BS: usize = 1048576;
const READ_LIMIT: usize = 16777216;

fn main() -> Result<(), Box<dyn Error>> {
    let files = std::env::args().skip(1)
        .map(File::open)
        .collect::<io::Result<Vec<_>>>()?;
    let mut files = files.iter()
        .map(|f| BufReader::with_capacity(READ_BS, f))
        .collect::<Vec<_>>();
    let mut datas = files.iter()
        .map(|_| vec![0u8; READ_LIMIT])
        .collect::<Vec<Vec<u8>>>();
    for (i, file) in files.iter_mut().enumerate() {
        file.read_exact(&mut datas[i])?;
    }

    // ReedSolomon::new(files.len(), parity_shards)
    // for offset in 0usize..16 {
    //     print!("{:016X}", offset);
    //     for disk in 0..datas.len() {
    //         print!(" {:02X}", datas[disk][offset]);
    //     }
    //     println!();
    // }

    // adaptec puts q before p in the first stripe (row)
    let mut candidates = PickPQ::new(datas.len())
        .collect::<BTreeSet<_>>();
    eprintln!("{:?}", candidates);
    for offset in 0usize..STRIPE {
        for &(q, p) in candidates.clone().iter() {
            let expected = datas[p][offset];
            let actual = datas.iter()
                .enumerate()
                .filter(|(i, _)| *i != q && *i != p)
                .map(|(_, x)| x[offset])
                // .inspect(|x| eprintln!("{:02X}h", x))
                .reduce(|r, x| r ^ x)
                .unwrap();
            // eprintln!("({},{}) => {:02X} => {}", q, p, actual, actual == expected);
            // eprintln!("{:02X}h <<< {:02X}h", actual, expected);
            if actual != expected {
                candidates.remove(&(q, p));
            }
        }
        eprintln!("{:016X} >>> {} candidates: {:?}", offset, candidates.len(), candidates);
        let qs = candidates.iter().map(|(q, _)| q)
            .collect::<BTreeSet<_>>();
        // eprintln!("{:?}", ps);
        if qs.len() == 1 {
            eprintln!("found q = {}", qs.iter().next().unwrap());
            break;
        }
    }

    Ok(())
}

struct PickPQ {
    n: usize,
    p: usize,
    q: usize,
}

impl PickPQ {
    fn new(n: usize) -> Self {
        Self { n, p: 0, q: 1 }
    }
}

impl Iterator for PickPQ {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let Self { n, p, q } = *self;

        if p >= n || q >= n {
            return None;
        }

        let result = (p, q);
        let (mut q, oldq) = (q, q);
        while q == oldq || q == p {
            q += 1;
        }

        if q >= n {
            self.p = p + 1;
            self.q = 0;
        } else {
            self.p = p;
            self.q = q;
        };

        Some(result)
    }
}
