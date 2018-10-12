use std::cmp;

#[derive(Debug)]
pub struct CONCISE {
    pub words: Option<Vec<i32>>,
    last: i32,
    size: i32,
    last_word_index: i32,
}

impl CONCISE {
    pub fn new() -> CONCISE {
        return CONCISE{
            words: None,
            last: -1,
            size: 0,
            last_word_index: -1,
        }
    }

    const MAX_LITERAL_LENGTH: i32 = 31;
    const ALL_ZEROS_LITERAL: i32 = 0x80000000;
    const ALL_ONES_LITERAL: i32 = 0xFFFFFFFF;
    const SEQUENCE_BIT: i32 = 0x40000000;

    pub fn append(&mut self, i: i32) {
        if self.words.is_none() {
            let zero_blocks = i / 31;
            if zero_blocks == 0 {
                self.words = Some(vec![0; 1]);
                self.last_word_index = 0;
            } else if zero_blocks == 1 {
                self.words = Some(vec![0; 2]);
                self.last_word_index = 1;
                self.words.as_mut().unwrap()[0] = CONCISE::ALL_ZEROS_LITERAL;
            } else {
                self.words = Some(vec![0; 2]);
                self.last_word_index = 1;
                self.words.as_mut().unwrap()[0] = zero_blocks - 1;
            }
            self.last = i;
            self.size = 1;
            self.words.as_mut().unwrap()[self.last_word_index as usize] = CONCISE::ALL_ZEROS_LITERAL | (1 << (i % 31));
            return;
        }

        let mut bit = self.last % 31 + i - self.last;

        if bit >= CONCISE::MAX_LITERAL_LENGTH {
            let zero_blocks = bit / 31 - 1;
            bit %= 31;
            if zero_blocks == 0 {
                self.ensure_capacity((self.last_word_index + 1) as usize);
            } else {
                self.ensure_capacity((self.last_word_index + 2) as usize);
                self.append_fill(zero_blocks, 0);
            }
            self.append_literal(CONCISE::ALL_ZEROS_LITERAL | 1 << bit);
        } else {
            self.words.as_mut().unwrap()[self.last_word_index as usize] |= 1 << bit;
            if self.words.as_mut().unwrap()[self.last_word_index as usize] == CONCISE::ALL_ONES_LITERAL {
                self.last_word_index -= 1;
                self.append_literal(CONCISE::ALL_ONES_LITERAL);
            }
        }

        self.last = i;
        if self.size >= 0 {
            self.size += 1;
        }
    }

    fn ensure_capacity(&mut self, index: usize) {
        let mut capacity = if self.words.is_none() { 0 } else { self.words.as_mut().unwrap().len() };
        if capacity > index {
            return;
        }
        capacity = cmp::max(capacity << 1, index + 1);

        // XXX: This is probably inefficient
        if self.words.is_none() {
            self.words = Some(vec![0; capacity]);
            return;
        }
        let mut new_words = vec![0; capacity];
        for (i, word) in self.words.as_mut().unwrap().iter().enumerate() {
            new_words[i] = *word;
        }
        self.words = Some(new_words);
    }

    fn append_fill(&mut self, length: i32, mut fill_type: i32) {
        // XXX: Are these really necessary?
        assert!(length > 0);
        assert!(self.last_word_index >= -1);

        fill_type &= CONCISE::SEQUENCE_BIT;

        if length == 1 {
            self.append_literal(if fill_type == 0 { CONCISE::ALL_ZEROS_LITERAL } else { CONCISE::ALL_ONES_LITERAL });
            return;
        }

        if self.last_word_index < 0 {
            self.words.as_mut().unwrap()[self.last_word_index as usize] = fill_type | (length - 1);
            return;
        }

        let last_word = self.words.as_mut().unwrap()[self.last_word_index as usize];
        if self.is_literal(last_word) {
            if fill_type == 0 && last_word == CONCISE::ALL_ZEROS_LITERAL {
                self.words.as_mut().unwrap()[self.last_word_index as usize] = length;
            } else if fill_type == CONCISE::SEQUENCE_BIT && last_word == CONCISE::ALL_ONES_LITERAL {
                self.words.as_mut().unwrap()[self.last_word_index as usize] = CONCISE::SEQUENCE_BIT | length;
            } else {
                if fill_type == 0 && self.contains_only_one_bit(self.get_literal_bits(last_word)) {
                    self.words.as_mut().unwrap()[self.last_word_index as usize] = length | ((1 + last_word.trailing_zeros() as i32) << 25);
                } else if fill_type == CONCISE::SEQUENCE_BIT && self.contains_only_one_bit(!last_word) {
                    self.words.as_mut().unwrap()[self.last_word_index as usize] = CONCISE::SEQUENCE_BIT | length | ((1 + (!last_word).trailing_zeros() as i32) << 25);
                } else {
                    self.last_word_index += 1;
                    self.words.as_mut().unwrap()[self.last_word_index as usize] = fill_type | (length - 1);
                }
            }
        } else {
            if last_word & 0xC0000000 == fill_type {
                self.words.as_mut().unwrap()[self.last_word_index as usize] += length;
            } else {
                self.last_word_index += 1;
                self.words.as_mut().unwrap()[self.last_word_index as usize] = fill_type | (length - 1);
            }
        }
    }

    fn append_literal(&mut self, word: i32) {
        if self.last_word_index == 0 && word == CONCISE::ALL_ZEROS_LITERAL && self.words.as_mut().unwrap()[0] == 0x01FFFFFF {
            return;
        }

        if self.last_word_index < 0 {
            self.last_word_index = 0;
            self.words.as_mut().unwrap()[self.last_word_index as usize] = word;
            return;
        }

        let last_word = self.words.as_mut().unwrap()[self.last_word_index as usize];
        if word == CONCISE::ALL_ZEROS_LITERAL {
            if last_word == CONCISE::ALL_ZEROS_LITERAL {
                self.words.as_mut().unwrap()[self.last_word_index as usize] = 1;
            } else if self.is_zero_sequence(last_word) {
                self.words.as_mut().unwrap()[self.last_word_index as usize] += 1;
            } else if self.contains_only_one_bit(self.get_literal_bits(last_word)) {
                self.words.as_mut().unwrap()[self.last_word_index as usize] = 1 | ((1 + last_word.trailing_zeros() as i32) << 25);
            } else {
                self.last_word_index += 1;
                self.words.as_mut().unwrap()[self.last_word_index as usize] = word;
            }
        } else if word == CONCISE::ALL_ONES_LITERAL {
            if last_word == CONCISE::ALL_ONES_LITERAL {
                self.words.as_mut().unwrap()[self.last_word_index as usize] = CONCISE::SEQUENCE_BIT | 1;
            } else if self.is_one_sequence(last_word) {
                self.words.as_mut().unwrap()[self.last_word_index as usize] += 1;
            } else if self.contains_only_one_bit(!last_word) {
                self.words.as_mut().unwrap()[self.last_word_index as usize] = CONCISE::SEQUENCE_BIT | 1 | ((1 + (!last_word).trailing_zeros() as i32) << 25);
            } else {
                self.last_word_index += 1;
                self.words.as_mut().unwrap()[self.last_word_index as usize] = word;
            }
        } else {
            self.last_word_index += 1;
            self.words.as_mut().unwrap()[self.last_word_index as usize] = word;
        }
    }

    fn is_zero_sequence(&self, word: i32) -> bool {
        return (word & 0xC0000000) == 0;
    }

    fn is_one_sequence(&self, word: i32) -> bool {
        return (word & 0xC0000000) == CONCISE::SEQUENCE_BIT;
    }

    fn is_literal(&self, word: i32) -> bool {
        return (word & 0x80000000) != 0;
    }

    fn contains_only_one_bit(&self, literal: i32) -> bool {
        return (literal & (literal - 1)) == 0;
    }

    fn get_literal_bits(&self, word: i32) -> i32 {
        return 0x7FFFFFFF & word;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
