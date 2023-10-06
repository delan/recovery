use std::{fs::File, env::args, error::Error, io::{BufWriter, Write}, mem::size_of};

use byteorder::{NetworkEndian, ByteOrder};
use rand::random;

fn main() -> Result<(), Box<dyn Error>> {
    let disk = File::create(args().nth(1).unwrap())?;
    let mut disk = BufWriter::with_capacity(1048576, disk);

    for lba in 0u64.. {
        let megs = lba / (1048576 / 512);
        let remainder = lba % (1048576 / 512);
        if remainder == 0 {
            dbg!(megs);
        }

        const COUNT: usize = 512 / size_of::<u64>();
        let mut data = [lba + 0x8000000000000000; COUNT];
        for i in 0..(COUNT / 2) {
            data[i * 2 + 1] = random();
        }

        let mut block = [0u8; 512];
        NetworkEndian::write_u64_into(&data, &mut block);
        disk.write_all(&block)?;
    }

    Ok(())
}
