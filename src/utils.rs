#[allow(dead_code)]
pub fn rgba_to_bgra(pixels: &mut [u8]) {
    let len = pixels.len();
    let ptr = pixels.as_mut_ptr();
    unsafe {
        for i in (0..len).step_by(4) {
            let r = ptr.add(i).read();
            let b = ptr.add(i + 2).read();
            ptr.add(i).write(b); // 写入 B
            ptr.add(i + 2).write(r); // 写入 R
        }
    }
}

#[allow(dead_code)]
pub fn bgra_to_rgba(pixels: &mut [u8]) {
    let len = pixels.len();
    let ptr = pixels.as_mut_ptr();
    unsafe {
        for i in (0..len).step_by(4) {
            let b = ptr.add(i).read();
            let r = ptr.add(i + 2).read();
            ptr.add(i).write(r); // 写入 R
            ptr.add(i + 2).write(b); // 写入 B
        }
    }
}
