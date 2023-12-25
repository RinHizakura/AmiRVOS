use std::fs::File;
use std::io::{Write, Seek, SeekFrom};
use std::env;

// block size
const BLKSZ: usize = 1024;
// size of file system in blocks
const FS_BLKSZ: usize = 2000;

fn wsect(f: &mut File, sec: u64, buf: &[u8]) {
    let off = f.seek(SeekFrom::Start(sec * BLKSZ as u64)).expect("seek for wsect fail");
    assert!(off as u64 == sec * BLKSZ as u64);

    let size = f.write(buf).expect("write for wsect fail");
    assert!(size == BLKSZ);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let img_name = &args[1];

    let mut f = File::create(img_name).expect("create()");

    for i in 0..FS_BLKSZ {
        wsect(&mut f, i as u64, &[0;BLKSZ]);
    }
}
