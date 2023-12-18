use crate::{types::{BroadcastMessage, ExchangePublicPeers, ContractIpoBid, assets::{TransferAssetOwnershipAndPossessionInput, IssueAssetInput}}, prelude::{Transaction, TickData, FutureTickData, Call}};


#[derive(Debug)]
pub enum NetworkEvent {
    ExchangePublicPeers(ExchangePublicPeers),
    BroadcastMessage(BroadcastMessage),
    BroadcastTransaction(Transaction),
    BroadcastQxTransfer(Call<TransferAssetOwnershipAndPossessionInput>),
    BroadcasQxAssetIssuance(Call<IssueAssetInput>),
    BroadcastIpoBid(Call<ContractIpoBid>),
    BroadcastTick(TickData),
    BroadcastFutureTick(FutureTickData)
}