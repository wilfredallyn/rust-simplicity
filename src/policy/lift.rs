// Simplicity lifting to miniscript semantic representation
// Written in 2020 by
//     Sanket Kanjalkar <sanket1729@gmail.com>
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Lift simplicity programs into miniscript semantic policies
//! Not all simplicity programs can be lifted to semantic langauge.
//! Currently the policy compilation is one to one mapping
//! between policy fragment and a simplicity program.

use crate::core::TermDag;
use crate::core::Value;
use crate::jet;
use crate::jet::application::Bitcoin;
use crate::jet::Application;
use crate::policy::key::PublicKey32;
use crate::util::slice_to_u32_be;
use bitcoin_hashes::{sha256, Hash};
use miniscript::policy::Liftable;
use miniscript::policy::Semantic;
use miniscript::{DummyKey, MiniscriptKey};
use std::rc::Rc;

/// Functional opposite of scribe. Read the scribed value
/// by interpretting that as constant function and return
/// a value corresponding to it.
pub fn read_scribed_value<Witness, App: Application>(dag: Rc<TermDag<Witness, App>>) -> Value {
    match dag.as_ref() {
        TermDag::Unit => Value::Unit,
        TermDag::InjL(l) => Value::sum_l(read_scribed_value(Rc::clone(l))),
        TermDag::InjR(r) => Value::sum_r(read_scribed_value(Rc::clone(r))),
        TermDag::Pair(l, r) => Value::prod(
            read_scribed_value(Rc::clone(l)),
            read_scribed_value(Rc::clone(r)),
        ),
        // Fixme: Change to errors
        _ => unreachable!(),
    }
}

// FIXME: Wait for 32 byte pubkeys to be added to rust-bitcoin.
// Then, we can add implementations that depend on bitcoin::PublicKey
impl<Witness> Liftable<DummyKey> for TermDag<Witness, Bitcoin>
where
    Witness: Eq,
{
    // Lift a simplicity program into a semantic policy
    fn lift(&self) -> Result<Semantic<DummyKey>, miniscript::Error> {
        let ret = match self {
            TermDag::Unit => Semantic::Trivial,
            TermDag::Comp(l, r) => {
                // check for Key
                match (&**l, &**r) {
                    (TermDag::Pair(key, w), TermDag::Jet(&jet::bitcoin::BIP_0340_VERIFY)) => {
                        let key_value = read_scribed_value(Rc::clone(&Rc::clone(key)));
                        let key_bytes = key_value.try_to_bytes().unwrap();
                        let k = DummyKey::from_32_bytes(&key_bytes);
                        match &**w {
                            TermDag::Witness(..) => Semantic::KeyHash(k.to_pubkeyhash()),
                            _ => unimplemented!(),
                        }
                    }
                    (
                        TermDag::Pair(scribed_hash, computed_hash),
                        TermDag::Jet(&jet::bitcoin::EQ256_VERIFY),
                    ) => {
                        let hash_value = read_scribed_value(Rc::clone(&Rc::clone(scribed_hash)));
                        let hash_bytes = hash_value.try_to_bytes().unwrap();
                        let h = sha256::Hash::from_slice(&hash_bytes).unwrap();
                        match &**computed_hash {
                            TermDag::Pair(w, sha_jet) => match (&**w, &**sha_jet) {
                                (TermDag::Witness(..), TermDag::Jet(&jet::bitcoin::SHA256)) => {
                                    Semantic::Sha256(h)
                                }
                                _ => unimplemented!(),
                            },
                            _ => unimplemented!(),
                        }
                    }
                    (
                        TermDag::Pair(scibe_t, computed_t),
                        TermDag::Jet(&jet::bitcoin::LT32_VERIFY),
                    ) => {
                        let timelock_value = read_scribed_value(Rc::clone(&Rc::clone(scibe_t)));
                        let timelock_bytes = timelock_value.try_to_bytes().unwrap();
                        let t = slice_to_u32_be(&timelock_bytes);
                        match &**computed_t {
                            TermDag::Jet(&jet::bitcoin::LOCK_TIME) => Semantic::After(t),
                            TermDag::Jet(&jet::bitcoin::CURRENT_SEQUENCE) => Semantic::After(t),
                            _ => unimplemented!(),
                        }
                    }
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        };

        Ok(ret)
    }
}
