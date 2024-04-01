extern crate alloc;

use alloc::vec::Vec;

use crate::PartyIndex;
use crate::MsgId;

/// Iterate peers of i-th party
pub fn iter_peers(i: u16, n: u16) -> impl Iterator<Item = u16> {
    (0..n).filter(move |x| *x != i)
}

/// For some messages it is possible to precisely identify where the fault
/// happened and which party is to blame. Use this struct to collect present the
/// blame.
///
/// In the future we might want to replace the data_message and proof_message
/// with a generic vec of messages.
#[derive(Debug)]
pub struct AbortBlame {
    /// Party which can be blamed for breaking the protocol
    pub faulty_party: PartyIndex,
    /// Message with initial data
    pub data_message: MsgId,
    /// Message with some kind of proof related to the data
    pub proof_message: MsgId,
}

impl AbortBlame {
    pub fn new(faulty_party: PartyIndex, data_message: MsgId, proof_message: MsgId) -> Self {
        Self {
            faulty_party,
            data_message,
            proof_message,
        }
    }
}

/// List of received messages
#[derive(Debug, Clone)]
pub struct RoundMsgs<M> {
    i: PartyIndex,
    ids: Vec<MsgId>,
    messages: Vec<M>,
}

impl<M> RoundMsgs<M> {
    /// Returns iterator over messages with sender indexes
    ///
    /// Iterator yields `(sender_index, msg_id, &message)`
    pub fn iter_indexed(&self) -> impl Iterator<Item = (PartyIndex, MsgId, &M)> {
        let parties_indexes = (0..self.i).chain(self.i + 1..);
        parties_indexes
            .zip(&self.ids)
            .zip(&self.messages)
            .map(|((party_ind, msg_id), msg)| (party_ind, *msg_id, msg))
    }
}

/// Filter returns `true` for every __faulty__ message pair
pub fn collect_blame<D, P, F>(
    data_messages: &RoundMsgs<D>,
    proof_messages: &RoundMsgs<P>,
    mut filter: F,
) -> Vec<AbortBlame>
where
    F: FnMut(PartyIndex, &D, &P) -> bool,
{
    data_messages
        .iter_indexed()
        .zip(proof_messages.iter_indexed())
        .filter_map(|((j, data_msg_id, data), (_, proof_msg_id, proof))| {
            if filter(j, data, proof) {
                Some(AbortBlame::new(j, data_msg_id, proof_msg_id))
            } else {
                None
            }
        })
        .collect()
}
