use core::mem;

pub fn to_struct<T: plain::Plain>(args: &[u8]) -> &T {
    let size = mem::size_of::<T>();
    let slice = &args[0..size];
    plain::from_bytes::<T>(slice).expect("Fail to cast bytes to Args")
}

pub fn to_struct_mut<T: plain::Plain>(args: &mut [u8]) -> &mut T {
    let size = mem::size_of::<T>();
    let slice = &mut args[0..size];
    plain::from_mut_bytes::<T>(slice).expect("Fail to cast bytes to Args")
}
