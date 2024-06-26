use qubic_types::traits::{FromBytes, ToBytes};
use qubic_types::Signature;
use crate::consts::NUMBER_OF_COMPUTORS;

use crate::utils::QubicRequest;

use super::time::QubicSetUtcTime;


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CommandType {
    SpecialCommandShutDown                     = 0,
    SpecialCommandGetProposalAndBallotRequest  = 1,
    SpecialCommandGetProposalAndBallotResponse = 2,
    SpecialCommandSetProposalAndBallotRequest  = 3,
    SpecialCommandSetProposalAndBallotResponse = 4,
    SpecialCommandSetSolutionThresholdRequest  = 5,
    SpecialCommandSetSolutionThresholdResponse = 6,

    /// F12
    SpecialCommandToggleMainAuxRequest         = 7,
    SpecialCommandToggleMainAuxResponse        = 8,
    /// F4
    SpecialCommandRefreshPeerList              = 9,
    /// F5
    SpecialCommandForceNextTick                = 10,
    /// F9
    SpecialCommandReissueVote                  = 11,

    SpecialCommandQueryTime                    = 12,
    SpecialCommandSendTime                     = 13
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(8))]
pub struct CommandDescriptor {
    pub command_type: CommandType,
    pub nonce: [u8; 7]
}

#[cfg(feature = "std")]
impl CommandDescriptor {
    pub fn new(command_type: CommandType) -> Self {
        let mut nonce = [0u8; 7];

        let now = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap().as_secs();

        nonce.copy_from_slice(&now.to_le_bytes()[..7]);

        Self {
            command_type,
            nonce
        }
    }
}

pub trait GetCommandType {
    fn get_command_type() -> CommandType;
}

macro_rules! set_command_type {
    ($impl: ident, $message_type: expr) => {
        impl GetCommandType for $impl {
            fn get_command_type() -> CommandType {
                $message_type
            }
        }
    };
}

#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SpecialCommand<T: ToBytes + FromBytes> {
    pub descriptor: CommandDescriptor,
    pub payload: T
}

impl<T: ToBytes + FromBytes> ToBytes for SpecialCommand<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.descriptor.to_bytes();
        bytes.extend(self.payload.to_bytes());
        bytes
    }
}

impl<T: ToBytes + FromBytes> FromBytes for SpecialCommand<T> {
    fn from_bytes(data: &[u8]) -> Result<Self, qubic_types::errors::ByteEncodingError> {
        let desc = CommandDescriptor::from_bytes(&data[..core::mem::size_of::<CommandDescriptor>()])?;
        let payload = T::from_bytes(&data[core::mem::size_of::<CommandDescriptor>()..])?;

        Ok(Self {
            descriptor: desc,
            payload
        })
    }
}

impl<T: ToBytes + FromBytes> QubicRequest for SpecialCommand<T> {
    fn get_message_type() -> crate::MessageType {
        crate::MessageType::ProcessSpecialCommand
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Proposal {
    pub uri_size: u8,
    pub uri: [u8; 255]
}

impl Proposal {
    pub fn from_uri(uri: String) -> Option<Proposal> {
        if uri.len() > 255 || !uri.is_ascii() {
            return None;
        }

        let uri = uri.as_bytes();
        let mut uri_buffer = [0; 255];

        uri_buffer[0..uri.len()].copy_from_slice(uri);

        Some(
            Self { uri_size: uri.len() as u8, uri: uri_buffer }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Ballot {
    pub zero: u8,
    pub votes: [u8; (NUMBER_OF_COMPUTORS * 3 + 7)/8],
    pub quasi_random_number: u8
}

impl Default for Ballot {
    fn default() -> Self {
        Self { zero: 0, votes: [0; (NUMBER_OF_COMPUTORS * 3 + 7)/8], quasi_random_number: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoteOption {
    NotVoted,
    Option(u8)
}

impl Ballot {
    pub fn get_computor_vote(&self, computor_index: usize) -> VoteOption {
        let compressed_index = (computor_index*3)/8;
        let index_offset = computor_index % 3;
        let expanded_index = 8 - index_offset;

        let vote = if index_offset == 0 {
            (self.votes[compressed_index] >> expanded_index) & 0b111
        } else {
            ((self.votes[compressed_index] >> expanded_index) & (u8::MAX >> expanded_index)) | ((self.votes[compressed_index + 1] & (u8::MAX >> (8 - 3 + index_offset))) << index_offset)
        };
        
        if vote == 0 {
            VoteOption::NotVoted
        } else{
            VoteOption::Option(vote)
        }
    }

    pub fn get_votes(&self) -> Vec<VoteOption> {
        let mut votes = Vec::with_capacity(NUMBER_OF_COMPUTORS);
        for i in 0..NUMBER_OF_COMPUTORS {
            votes.push(self.get_computor_vote(i));
        }

        votes
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GetProposalAndBallotRequest {
    pub computor_index: u16,
    pub padding: [u8; 6],
    pub signature: Signature
}

set_command_type!(GetProposalAndBallotRequest, CommandType::SpecialCommandGetProposalAndBallotRequest);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct GetProposalAndBallotResponse {
    pub computor_index: u16,
    pub padding: [u8; 6],
    pub proposal: Proposal,
    pub ballot: Ballot
}

set_command_type!(GetProposalAndBallotResponse, CommandType::SpecialCommandGetProposalAndBallotResponse);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SetProposalAndBallotRequest {
    pub computor_index: u16,
    pub padding: [u8; 6],
    pub proposal: Proposal,
    pub ballot: Ballot,
    pub signature: Signature
}

set_command_type!(SetProposalAndBallotRequest, CommandType::SpecialCommandSetProposalAndBallotRequest);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SetProposalAndBallotResponse {
    pub computor_index: u16,
    pub padding: [u8; 6],
}

set_command_type!(SetProposalAndBallotResponse, CommandType::SpecialCommandSetProposalAndBallotResponse);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum NodeMode {
    Aux = 0,
    Main = 1
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct ToggleMainMode {
    pub mode: NodeMode
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SetEpochParams {
    pub epoch: u32,
    pub treshold: i32
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SetTime {
    pub time: QubicSetUtcTime
}