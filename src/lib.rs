#[derive(Debug, Eq, PartialEq)]
pub struct Block {
    bytes: Vec<u8>,
}

impl Block {
    pub fn new(vector: &Vec<u8>) -> Self {
        Block {
            bytes: vector.clone(),
        }
    }

    pub fn from_message(message: &Vec<u8>) -> Result<Vec<Self>, &'static str> {
        if message.len() % 64 != 0 {
            Err("Message is not align, len must be multiple of 64 bytes!")
        } else {
            Ok(message
                .chunks(64)
                .map(|x| Block {
                    bytes: Vec::from(x),
                })
                .collect())
        }
    }
}

pub fn pad_message(message: &Vec<u8>) -> Result<Vec<Block>, &'static str> {
    let mut paded_message = message.clone();
    paded_message.push(128); // 0x80 or 0xb1000_0000

    while (paded_message.len() + 8) % 64 != 0 {
        paded_message.push(0);
    }

    for byte in &((message.len() * 8) as u64).to_be_bytes() {
        paded_message.push(*byte);
    }

    println!("Message len: {:?}", message.len() * 8);

    Ok(Block::from_message(&paded_message).unwrap())
}

fn k_for(t: usize) -> u32 {
    match t {
        n if n < 20 => 0x5A827999,
        n if 20 <= n && n < 40 => 0x6ED9EBA1,
        n if 40 <= n && n < 60 => 0x8F1BBCDC,
        n if 60 <= n && n < 80 => 0xCA62C1D6,
        _ => 0,
    }
}

fn function_for(t: usize, b: u32, c: u32, d: u32) -> u32 {
    match t {
        n if n < 20 => (b & c) | ((!b) & d),
        n if 20 <= n && n < 40 => b ^ c ^ d,
        n if 40 <= n && n < 60 => (b & c) | (b & d) | (c & d),
        n if 60 <= n && n < 80 => b ^ c ^ d,
        _ => 0,
    }
}

pub fn compute(blocks: Vec<Block>) -> Result<String, &'static str> {
    let mut buf: [u32; 5]; // Buffer one, A..E
    let mut h: [u32; 5] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0]; // Buffer two, H0..H4
    let mut w = [0u32; 80]; // Sequance of W(0)..W(79)
    let mut temp: u32;

    for block in blocks {
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                block.bytes[i * 4],
                block.bytes[i * 4 + 1],
                block.bytes[i * 4 + 2],
                block.bytes[i * 4 + 3],
            ]);
        }

        for t in 16..80 {
            w[t] = (w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16]).rotate_left(1);
        }

        buf = h;

        for t in 0..80 {
            temp = buf[0].rotate_left(5).wrapping_add(
                function_for(t, buf[1], buf[2], buf[3])
                    .wrapping_add(buf[4].wrapping_add(w[t].wrapping_add(k_for(t)))),
            );
            buf[4] = buf[3]; // E = D
            buf[3] = buf[2]; // D = C
            buf[2] = buf[1].rotate_left(30); // C = S^30(B)
            buf[1] = buf[0]; // B = A
            buf[0] = temp; // A = temp

            println!(
                "[i = {i}] A={a}, B={b}, C={c}, D={d}, E={e}",
                i = t,
                a = buf[0],
                b = buf[1],
                c = buf[2],
                d = buf[3],
                e = buf[4]
            );
        }

        for i in 0..5 {
            h[i] = h[i].wrapping_add(buf[i]);
        }
    }

    Ok(format!(
        "{:8x}{:8x}{:8x}{:8x}{:8x}",
        h[0], h[1], h[2], h[3], h[4]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pad_message_with_one_block() {
        let message: Vec<u8> = vec![97, 98, 99, 100, 101];
        let mut expected = message.clone();
        expected.push(128);
        for _i in 0..57 {
            expected.push(0);
        }
        expected.push(40);
        let expected = Block::new(&expected);

        let paded_message = pad_message(&message).unwrap();
        assert_eq!(expected, paded_message[0]);
    }

    #[test]
    fn test_pad_message_with_multiple_block() {
        let a: &[u8; 64] = b"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let message: Vec<u8> = Vec::from(&a[..]);
        let mut expected = message.clone();
        expected.push(128);

        for _i in 0..61 {
            expected.push(0);
        }

        expected.push(2);
        expected.push(0);
        let expected = Block::from_message(&expected).unwrap();

        let paded_message = pad_message(&message).unwrap();
        assert_eq!(expected, paded_message);
    }

    #[test]
    fn test_compute_one_block() {
        let message: Vec<u8> = b"hello".to_vec();

        let input = pad_message(&message).unwrap();
        let hash = compute(input).unwrap();

        let expected = "aaf4c61ddcc5e8a2dabede0f3b482cd9aea9434d";

        assert_eq!(expected, hash);
    }

    #[test]
    fn test_compute_multi_block() {
        let message: Vec<u8> = b"Repellendus quae illo placeat. Ut id quaerat et architecto et inventore qui. Perferendis et provident sint animi exercitationem reprehenderit. Excepturi recusandae assumenda quia dolore.".to_vec();

        let input = pad_message(&message).unwrap();
        let hash = compute(input).unwrap();

        let expected = "a0a1284ae92e61f9fc48f998f4d28d34ee4c48de";

        assert_eq!(expected, hash);
    }
}
