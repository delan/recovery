const STRIPE: usize = 262144;
const SECTOR: usize = 512;
// Huge memory allocation (size 1861953576) rejected - metadata corruption?
// wc yields 1077156270 bytes
// const ROWS: usize = 1953525168 * SECTOR / STRIPE;
const ROWS: usize = 1953525168 * SECTOR / STRIPE / 100;

fn main() {
    let paths = std::env::args().skip(1)
        .collect::<Vec<_>>();

    let mut q = paths.len() - 2;
    let mut p = paths.len() - 1;
    let mut logical = 0;
    let mut physical = 0;
    for _ in 0..ROWS {
        for (i, path) in paths.iter().enumerate() {
            if i == q || i == p {
                continue;
            }
            let size = STRIPE / SECTOR;
            println!("{} {} linear {} {}", logical, size, path, physical);
            logical += size;
        }
        q = if q == 0 {
            paths.len() - 1
        } else {
            q - 1
        };
        p = if p == 0 {
            paths.len() - 1
        } else {
            p - 1
        };
        physical += STRIPE / SECTOR;
    }
}
