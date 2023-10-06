use std::{fs::File, io::{self, BufReader, Read}, error::Error, collections::BTreeSet};

const STRIPE: usize = 262144;

fn main() -> Result<(), Box<dyn Error>> {
    let files = std::env::args().skip(1)
        .map(File::open)
        .collect::<io::Result<Vec<_>>>()?;
    let mut files = files.iter()
        .map(|f| BufReader::with_capacity(STRIPE, f))
        .collect::<Vec<_>>();
    let mut order = vec![None; files.len()];

    for row in 0.. {
        let mut datas = files.iter()
            .map(|_| vec![0u8; STRIPE])
            .collect::<Vec<Vec<u8>>>();
        for (i, file) in files.iter_mut().enumerate() {
            file.read_exact(&mut datas[i])?;
        }

        let mut candidates = (0..datas.len())
            .collect::<BTreeSet<_>>();
        // eprintln!("{:?}", candidates);
        for offset in 0usize..STRIPE {
            // xor all disks = data ^ p ^ q = p ^ p ^ q = q
            let xor = datas.iter()
                .map(|x| x[offset])
                .reduce(|r, x| r ^ x)
                .unwrap();
            // remove disks that can’t be q
            for &q in candidates.clone().iter() {
                if datas[q][offset] != xor {
                    candidates.remove(&q);
                }
            }
            // eprintln!("{:016X} >>> {} candidates: {:?}", offset, candidates.len(), candidates);
            if candidates.len() == 1 {
                let q = candidates.iter().next().unwrap().clone();
                eprintln!("stripe {}: found q = {} at offset {:X}h", row, q, offset);
                let i = row % order.len();
                if let Some(old) = order[i] {
                    assert_eq!(q, old);
                } else {
                    order[i] = Some(q);
                }
                break;
            }
        }
        if candidates.len() > 1 {
            eprintln!("stripe {}: no q found", row);
        }

        if order.iter().all(|x| x.is_some()) {
            let order = order.iter().map(|x| x.unwrap()).collect::<Vec<_>>();
            // adaptec puts q before p in the first stripe (row), so the actual
            // order is from right to left starting from the second last index,
            // followed by the last index
            println!("order is {:?}", order);
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
