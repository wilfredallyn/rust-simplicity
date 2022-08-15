// Rust Simplicity Library
// Written in 2022 by
//   Christian Lewe <clewe@blockstream.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the CC0 Public Domain Dedication
// along with this software.
// If not, see <http://creativecommons.org/publicdomain/zero/1.0/>.
//

//! # Type Merkle roots
//!
//! Used at time of redemption (see [`super::imr`]).
//! Uniquely identifies the tree structure of a Simplicity type.

use crate::core::types::TypeInner;
use crate::impl_midstate_wrapper;
use crate::merkle::common::{MerkleRoot, TypeMerkleRoot};
use bitcoin_hashes::sha256::Midstate;

/// Type Merkle root
#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tmr(Midstate);

impl_midstate_wrapper!(Tmr);

impl TypeMerkleRoot for Tmr {
    fn get_iv(ty: &TypeInner) -> Self {
        match ty {
            TypeInner::Unit => Tmr::tag_iv(b"Simplicity-Draft\x1fType\x1fone"),
            TypeInner::Sum(..) => Tmr::tag_iv(b"Simplicity-Draft\x1fType\x1fsum"),
            TypeInner::Product(..) => Tmr::tag_iv(b"Simplicity-Draft\x1fType\x1fprod"),
        }
    }
}
