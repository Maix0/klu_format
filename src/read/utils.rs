pub fn slice_to_u64(slice: &[u8]) -> u64 {
    let mut out = 0;
    for (index, byte) in slice[0..8].iter().rev().enumerate() {
        out += (*byte as u64) << (index * 8)
    }
    out
}
/**(flag,headersize,file_name)*/
pub fn parse_header(slice: &[u8]) -> (bool, u64, String) {
    let filename_length = slice[0] as u64 >> 1;
    let filesize = slice_to_u64(&slice[1..9]);
    let flag = (slice[0] & 1) == 1;
    let filename = String::from_utf8(
        slice[9..(9 + filename_length) as usize]
            .iter()
            .map(|x| *x)
            .collect(),
    )
    .expect("A filename isn't valid UTF-8")
    .to_owned();
    (flag, filesize, filename)
}

pub fn split_in_place<T: Copy>(v: &mut Vec<T>, boundry: usize) -> Vec<T> {
    let out = v[..boundry].iter().map(|x| *x).collect();
    *v = v[boundry..].iter().map(|x| *x).collect();
    out
}
