pub trait RandomSource {
    fn gen(&mut self) -> u8;
}

pub struct FixedRandomSource {
    numbers: Vec<u8>,
    index: usize,
}

impl FixedRandomSource {
    pub fn new(numbers: Vec<u8>) -> FixedRandomSource {
        if numbers.is_empty() {
            panic!("Must provide at least one number in the random source");
        }

        FixedRandomSource { numbers, index: 0 }
    }
}

impl RandomSource for FixedRandomSource {
    fn gen(&mut self) -> u8 {
        let number = *self.numbers.get(self.index).unwrap();
        self.index = (self.index + 1) % self.numbers.len();
        number
    }
}

impl<T> RandomSource for T
where
    T: rand::Rng,
{
    fn gen(&mut self) -> u8 {
        rand::Rng::gen(self)
    }
}

// TODO need a true random source

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn fixed_random_source() {
        let mut rand = FixedRandomSource::new(vec![12, 34, 56]);

        assert_eq!(12, rand.gen());
        assert_eq!(34, rand.gen());
        assert_eq!(56, rand.gen());
        assert_eq!(12, rand.gen());
        assert_eq!(34, rand.gen());
        assert_eq!(56, rand.gen());
    }
}
