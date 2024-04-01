extern crate alloc;

use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::result::Result;

use digest::Digest;
use generic_ec::{Curve, Point, Scalar, SecretScalar};
use generic_ec_zkp::schnorr_pok::{self, ProverSecret};
use rand_core::{CryptoRng, RngCore};

use crate::execution_id::ExecutionId;
use crate::security_level::SecurityLevel;
use crate::{utils, KeygenAborted, KeygenError};

/// Message from round 1
#[derive(Clone, udigest::Digestable)]
#[udigest(bound = "")]
#[udigest(tag = "dfns.cggmp21.keygen.non_threshold.round1")]
pub struct MsgRound1<D: Digest> {
    /// $V_i$
    #[udigest(as_bytes)]
    pub commitment: digest::Output<D>,
}

/// Message from round 2
#[derive(Clone, udigest::Digestable)]
#[udigest(bound = "")]
#[udigest(tag = "dfns.cggmp21.keygen.non_threshold.round2")]
pub struct MsgRound2<L: SecurityLevel> {
    /// `rid_i`
    #[udigest(as_bytes)]
    pub rid: L::Rid,
    /// $X_i$
    pub X: Vec<u8>, // generic_ec::Point<E>.to_bytes(false)
    /// $A_i$
    pub sch_commit: Vec<u8>, // schnorr_pok::Commit<E>.to_bytes(false)
    /// Party contribution to chain code
    #[cfg(feature = "hd-wallets")]
    #[udigest(with = utils::encoding::maybe_bytes)]
    pub chain_code: Option<slip_10::ChainCode>,
    /// $u_i$
    #[udigest(as_bytes)]
    pub decommit: L::Rid,
}

/// Message from round 3
#[derive(Clone)]
pub struct MsgRound3<E: Curve> {
    /// $\psi_i$
    pub sch_proof: schnorr_pok::Proof<E>,
}

pub struct Secrets<E: Curve> {
    pub x_i: SecretScalar<E>,
    pub sch_secret: ProverSecret<E>
}

#[derive(udigest::Digestable)]
#[udigest(tag = "dfns.cggmp21.keygen.threshold.tag")]
enum Tag<'a> {
    /// Tag that includes the prover index
    Indexed {
        party_index: u16,
        #[udigest(as_bytes)]
        sid: &'a [u8],
    },
    /// Tag w/o party index
    Unindexed {
        #[udigest(as_bytes)]
        sid: &'a [u8],
    },
}

pub fn create_tag<D>(party_index: u16, sid: &[u8]) -> udigest::Tag<D>
where D: Digest + Clone + 'static,
{
    udigest::Tag::<D>::new_structured(Tag::Indexed {
        party_index,
        sid,
    })
}

pub async fn round_1_2<E, R, L, D>(
    i: u16,
    execution_id: ExecutionId<'_>,
    rng: &mut R,
) -> (MsgRound1<D>, MsgRound2<L>, Secrets<E>)
where
    E: Curve,
    L: SecurityLevel,
    D: Digest + Clone + 'static,
    R: RngCore + CryptoRng,
{
    let sid = execution_id.as_bytes();

    let x_i = SecretScalar::<E>::random(rng);
    let X_i = Point::generator() * &x_i;

    let mut rid = L::Rid::default();
    rng.fill_bytes(rid.as_mut());

    let (sch_secret, sch_commit) = schnorr_pok::prover_commits_ephemeral_secret::<E, _>(rng);

    let my_decommitment = MsgRound2::<L> {
        rid,
        X: X_i.to_bytes(false).as_bytes().to_owned(),
        sch_commit: sch_commit.0.to_bytes(false).as_bytes().to_owned(),
        #[cfg(feature = "hd-wallets")]
        chain_code: chain_code_local,
        decommit: {
            let mut nonce = L::Rid::default();
            rng.fill_bytes(nonce.as_mut());
            nonce
        },
    };
    let hash_commit = create_tag::<D>(i, sid).digest(&my_decommitment);
    let my_commitment = MsgRound1::<D> {
        commitment: hash_commit,
    };
    (my_commitment, my_decommitment, Secrets { x_i, sch_secret })
}

// pub async fn round_3<E>(
//     secrits: Secrets<E>,
//     challenge: Scalar<E>,
// ) -> MsgRound3<E>
// where
//     E: Curve,
// {
//     let challenge = schnorr_pok::Challenge { nonce: challenge };
//     let Secrets { x_i, sch_secret } = secrits;

//     let sch_proof = schnorr_pok::prove(&sch_secret, &challenge, &x_i);
//     let my_sch_proof = MsgRound3 { sch_proof };
//     my_sch_proof
// }

pub async fn round_3<E, R, L, D>(
    i: u16,
    execution_id: ExecutionId<'_>,
    commitments: Vec<MsgRound1<D>>,
    decommitments: Vec<MsgRound2<L>>,
    secrits: Secrets<E>,
    challenge: Scalar<E>,
) -> Result<MsgRound3<E>, KeygenError>
where
    E: Curve,
    L: SecurityLevel,
    D: Digest + Clone + 'static,
    R: RngCore + CryptoRng,
{
    // let sid = execution_id.as_bytes();

    // let blame = utils::collect_blame(&commitments, &decommitments, |j, com, decom| {
    //     let com_expected = create_tag::<D>(j, sid).digest(decom);
    //     com.commitment != com_expected
    // });
    // if !blame.is_empty() {
    //     return Err(KeygenAborted::InvalidDecommitment(blame).into());
    // }

    let challenge = schnorr_pok::Challenge { nonce: challenge };
    let Secrets { x_i, sch_secret } = secrits;

    let sch_proof = schnorr_pok::prove(&sch_secret, &challenge, &x_i);
    let my_sch_proof = MsgRound3 { sch_proof };
    Ok(my_sch_proof)
}
