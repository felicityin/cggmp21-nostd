// extern crate alloc;

// use alloc::borrow::ToOwned;
// use alloc::vec::Vec;

// use digest::Digest;
// use generic_ec::{Curve, Point, Scalar, SecretScalar};
// use generic_ec_zkp::polynomial::Polynomial;
// use generic_ec_zkp::schnorr_pok::{self, ProverSecret};
// use rand_core::{CryptoRng, RngCore};

// use crate::execution_id::ExecutionId;
// use crate::security_level::SecurityLevel;
// use crate::utils;

// /// Message from round 1
// #[derive(Clone, udigest::Digestable)]
// #[udigest(bound = "")]
// #[udigest(tag = "dfns.cggmp21.keygen.non_threshold.round1")]
// pub struct MsgRound1<D: Digest> {
//     /// $V_i$
//     #[udigest(as_bytes)]
//     pub commitment: digest::Output<D>,
// }

// /// Message from round 2 broadcasted to everyone
// #[derive(Clone, udigest::Digestable)]
// #[udigest(bound = "")]
// #[udigest(tag = "dfns.cggmp21.keygen.threshold.round1")]
// pub struct MsgRound2Broad<E: Curve, L: SecurityLevel> {
//     /// `rid_i`
//     #[udigest(as_bytes)]
//     pub rid: L::Rid,
//     /// $\vec S_i$
//     pub F: Polynomial<Point<E>>,
//     /// $A_i$
//     pub sch_commit: schnorr_pok::Commit<E>,
//     /// Party contribution to chain code
//     #[cfg(feature = "hd-wallets")]
//     #[serde_as(as = "Option<utils::HexOrBin>")]
//     #[udigest(with = utils::encoding::maybe_bytes)]
//     pub chain_code: Option<slip_10::ChainCode>,
//     /// $u_i$
//     #[udigest(as_bytes)]
//     pub decommit: L::Rid,
// }

// /// Message from round 2 unicasted to each party
// #[derive(Clone)]
// pub struct MsgRound2Uni<E: Curve> {
//     /// desnate party
//     pub to_party_index: u16,
//     /// $\sigma_{i,j}$
//     pub sigma: Scalar<E>,
// }

// pub struct Secrets<E: Curve> {
//     // pub x_i: SecretScalar<E>,
//     pub sch_secret: ProverSecret<E>
// }

// #[derive(udigest::Digestable)]
// #[udigest(tag = "dfns.cggmp21.keygen.threshold.tag")]
// enum Tag<'a> {
//     /// Tag that includes the prover index
//     Indexed {
//         party_index: u16,
//         #[udigest(as_bytes)]
//         sid: &'a [u8],
//     },
//     /// Tag w/o party index
//     Unindexed {
//         #[udigest(as_bytes)]
//         sid: &'a [u8],
//     },
// }

// pub fn create_tag<D>(party_index: u16, sid: &[u8]) -> udigest::Tag<D>
// where D: Digest + Clone + 'static,
// {
//     udigest::Tag::<D>::new_structured(Tag::Indexed {
//         party_index,
//         sid,
//     })
// }

// pub async fn round_1_2<E, R, L, D>(
//     i: u16,
//     t: u16,
//     n: u16,
//     execution_id: ExecutionId<'_>,
//     rng: &mut R,
// ) -> (MsgRound1<D>, MsgRound2Broad<E, L>, Vec<MsgRound2Uni<E>>, Secrets<E>)
// where
//     E: Curve,
//     L: SecurityLevel,
//     D: Digest + Clone + 'static,
//     R: RngCore + CryptoRng,
// {
//     let sid = execution_id.as_bytes();
//     let tag = create_tag::<D>(i, sid);

//     let mut rid = L::Rid::default();
//     rng.fill_bytes(rid.as_mut());

//     let (r, h) = schnorr_pok::prover_commits_ephemeral_secret::<E, _>(rng);

//     let f = Polynomial::<SecretScalar<E>>::sample(rng, usize::from(t) - 1);
//     let F = &f * &Point::generator();
//     let sigmas = (0..n)
//         .map(|j| {
//             let x = Scalar::from(j + 1);
//             f.value(&x)
//         })
//         .collect::<Vec<_>>();

//     let my_decommitment = MsgRound2Broad {
//         rid,
//         F: F.clone(),
//         sch_commit: h,
//         #[cfg(feature = "hd-wallets")]
//         chain_code: chain_code_local,
//         decommit: {
//             let mut nonce = L::Rid::default();
//             rng.fill_bytes(nonce.as_mut());
//             nonce
//         },
//     };
//     let hash_commit = tag.digest(&my_decommitment);
    
//     let my_commitment = MsgRound1 {
//         commitment: hash_commit,
//     };

//     let mut msg_round2_unis = Vec::new();

//     for j in utils::iter_peers(i, n) {
//         msg_round2_unis.push(MsgRound2Uni {
//             to_party_index: j,
//             sigma: sigmas[usize::from(j)],
//         });
//     }

//     (
//         my_commitment,
//         my_decommitment,
//         msg_round2_unis,
//         Secrets { sch_secret: r },
//     )
// }

// pub async fn round_3() {

// }
