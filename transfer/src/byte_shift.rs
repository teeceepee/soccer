
/// 字节转换器
pub struct ByteShifter {
    sugar: u8,
}

impl ByteShifter {
    pub fn new(sugar: u8) -> Self {
        Self {
            sugar,
        }
    }

    pub fn encode(&self, b: u8) -> u8 {
        if b <= 255 - self.sugar {
            b + self.sugar
        } else {
            self.sugar - (255 - b) - 1
        }
    }

    pub fn decode(&self, b: u8) -> u8 {
        if b >= self.sugar {
            b - self.sugar
        } else {
            255 - (self.sugar - b) + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_test() {
        let shifter = ByteShifter::new(10);

        assert_eq!(shifter.encode(0), 10);
        assert_eq!(shifter.encode(1), 11);
        assert_eq!(shifter.encode(10), 20);
        assert_eq!(shifter.encode(245), 255);
        assert_eq!(shifter.encode(246), 0);
        assert_eq!(shifter.encode(250), 4);
        assert_eq!(shifter.encode(255), 9);
    }

    #[test]
    fn decode_test() {
        let shifter = ByteShifter::new(10);

        assert_eq!(shifter.decode(255), 245);
        assert_eq!(shifter.decode(254), 244);
        assert_eq!(shifter.decode(11), 1);
        assert_eq!(shifter.decode(10), 0);
        assert_eq!(shifter.decode(9), 255);
        assert_eq!(shifter.decode(8), 254);
        assert_eq!(shifter.decode(1), 247);
        assert_eq!(shifter.decode(0), 246);
    }
}
