use super::keyseq::{KeySeq, KeySeqWithoutShiftMask};
use std::collections::HashMap;

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum KeymapEntry<T> {
    Complete(T),
    Incomplete,
    None,
}

pub struct Keymap<T>(HashMap<KeySeqWithoutShiftMask, KeymapEntry<T>>);

impl<T> Keymap<T>
where
    T: core::fmt::Debug + Clone,
{
    pub fn new(mut map: HashMap<KeySeq, T>) -> Self {
        let mut keymap = HashMap::new();

        for (mut keyseq, value) in map.drain() {
            assert!(!keyseq.is_empty());

            keymap.insert(keyseq.clone().into(), KeymapEntry::Complete(value));

            while !keyseq.is_empty() {
                keyseq.pop();
                keymap.insert(keyseq.clone().into(), KeymapEntry::Incomplete);
            }
        }

        Self(keymap)
    }

    pub fn get(&self, keyseq: &KeySeq) -> &KeymapEntry<T> {
        let keyseq = keyseq.clone().into();
        self.0.get(&keyseq).unwrap_or(&KeymapEntry::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::keyseq::{Key, KeySeqSerde, ModMask};
    use big_s::S;
    use xkbcommon::xkb::Keysym;

    #[test]
    fn test() {
        let keyseq_serde = KeySeqSerde::new(hashmap! {
            S("C") => ModMask::CONTROL,
            S("M") => ModMask::MOD1,
            S("s") => ModMask::MOD4,
            S("H") => ModMask::MOD5,
        });
        let kbd = |s| keyseq_serde.kbd(s).unwrap();
        let keymap = Keymap::new(hashmap! {
            kbd("a") => "a",
            kbd("A") => "A",
            kbd("dollar") => "$",
            kbd("H-x H-t") => "alacritty",
        });

        // Match without shift mask.
        let keyseq = vec![Key {
            modmask: ModMask::default(),
            keysym: Keysym::a,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::Complete("a"));
        let keyseq = vec![Key {
            modmask: ModMask::SHIFT,
            keysym: Keysym::A,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::Complete("A"));
        let keyseq = vec![Key {
            modmask: ModMask::SHIFT,
            keysym: Keysym::a,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::Complete("a"));
        let keyseq = vec![Key {
            modmask: ModMask::default(),
            keysym: Keysym::A,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::Complete("A"));

        let keyseq = vec![Key {
            modmask: ModMask::default(),
            keysym: Keysym::b,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::None);

        // Match without shift mask.
        //
        // For ascii characters, we can get know the shift mask is set by `!(c & 0x10)`.
        // E.g. '$' as u8 = ('4' as u8) ^ 0x10.
        // Common keyboard layouts follows the fashion, but we can't assume it under xkb in general.
        // So, `KeySeqWithoutShiftMask` is necessary.
        let keyseq = vec![Key {
            modmask: ModMask::SHIFT,
            keysym: Keysym::dollar,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::Complete("$"));
        let keyseq = vec![Key {
            modmask: ModMask::default(),
            keysym: Keysym::dollar,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::Complete("$"));
        let keyseq = vec![Key {
            modmask: ModMask::default(),
            keysym: Keysym::_4,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::None);
        let keyseq = vec![Key {
            modmask: ModMask::SHIFT,
            keysym: Keysym::_4,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::None);

        // Key sequence
        let keyseq = vec![Key {
            modmask: ModMask::MOD5,
            keysym: Keysym::x,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::Incomplete);
        let keyseq = vec![
            Key {
                modmask: ModMask::MOD5,
                keysym: Keysym::x,
            },
            Key {
                modmask: ModMask::MOD5,
                keysym: Keysym::t,
            },
        ]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::Complete("alacritty"));
        let keyseq = vec![
            Key {
                modmask: ModMask::MOD5,
                keysym: Keysym::x,
            },
            Key {
                modmask: ModMask::MOD5,
                keysym: Keysym::t,
            },
            Key {
                modmask: ModMask::MOD5,
                keysym: Keysym::t,
            },
        ]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::None);
        let keyseq = vec![Key {
            modmask: ModMask::MOD5,
            keysym: Keysym::t,
        }]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::None);
        let keyseq = vec![
            Key {
                modmask: ModMask::MOD5,
                keysym: Keysym::x,
            },
            Key {
                modmask: ModMask::MOD5,
                keysym: Keysym::x,
            },
        ]
        .into();
        assert_eq!(*keymap.get(&keyseq), KeymapEntry::None);
    }
}
