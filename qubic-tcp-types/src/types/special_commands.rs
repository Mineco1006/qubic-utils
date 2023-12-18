use qubic_types::Signature;
use crate::consts::NUMBER_OF_COMPUTORS;

use crate::utils::QubicRequest;


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CommandType {
    SpecialCommandShutDown = 0,
    SpecialCommandGetProposalAndBallotRequest = 1,
    SpecialCommandGetProposalAndBallotResponse = 2,
    SpecialCommandSetProposalAndBallotRequest = 3,
    SpecialCommandSetProposalAndBallotResponse = 4,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(8))]
pub struct CommandDescriptor {
    pub command_type: CommandType,
    pub nonce: [u8; 7]
}

pub(crate) trait GetCommandType {
    fn get_command_type() -> CommandType;
}

#[macro_export]
macro_rules! set_command_type {
    ($impl: ident, $message_type: expr) => {
        impl GetCommandType for $impl {
            fn get_command_type() -> CommandType {
                $message_type
            }
        }
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SpecialCommand<T: Copy> {
    pub descriptor: CommandDescriptor,
    pub payload: T
}

impl<T: Copy> QubicRequest for SpecialCommand<T> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Ballot {
    pub zero: u16,
    pub votes: [u8; (NUMBER_OF_COMPUTORS * 3 + 7)/8],
    pub quasi_random_number: u8
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
struct GetProposalAndBallotRequest {
    pub computor_index: u16,
    pub padding: [u8; 6],
    pub signature: Signature
}

set_command_type!(GetProposalAndBallotRequest, CommandType::SpecialCommandGetProposalAndBallotRequest);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
struct GetProposalAndBallotResponse {
    pub computor_index: u16,
    pub padding: [u8; 6],
    pub proposal: Proposal,
    pub ballot: Ballot
}

set_command_type!(GetProposalAndBallotResponse, CommandType::SpecialCommandGetProposalAndBallotResponse);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
struct SetProposalAndBallotRequest {
    pub computor_index: u16,
    pub padding: [u8; 6],
    pub proposal: Proposal,
    pub ballot: Ballot,
    pub signature: Signature
}

set_command_type!(SetProposalAndBallotRequest, CommandType::SpecialCommandSetProposalAndBallotRequest);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
struct SetProposalAndBallotResponse {
    pub computor_index: u16,
    pub padding: [u8; 6],
}

set_command_type!(SetProposalAndBallotResponse, CommandType::SpecialCommandSetProposalAndBallotResponse);