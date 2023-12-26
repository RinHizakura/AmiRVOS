use core::mem;

pub fn to_struct<T: plain::Plain>(args: &[u8]) -> &T {
    let size = mem::size_of::<T>();
    let slice = &args[0..size];
    return plain::from_bytes::<T>(slice).expect("Fail to cast bytes to Args");
}
