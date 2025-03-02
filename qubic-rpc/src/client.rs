use std::collections::HashMap;

use anyhow::{anyhow, Result};
use base64::Engine;
use serde_json::Value;
use url::Url;

use crate::qubic_rpc_types::{
    Asset, AssetType, Balance, ComputorInfos, ComputorsWrapper, Hash, LatestStats,
    LatestStatsWrapper, LatestTick, QuorumTickData, QuorumTickDataWrapper, RichEntity,
    RichListWrapper, RpcHealth, RpcStatus, SmartContract, TickData, TickDataWrapper, TickInfo,
    TickInfoWrapper, TickTransactions, TransactionResponse, TransactionResponseData,
    TransactionsResponse, WalletBalance,
};
use qubic_rs::{
    qubic_tcp_types::types::transactions::TransactionWithData, qubic_types::traits::ToBytes,
};

/// Qubic RPC Client
///
/// # Examples
///
/// ```rust
/// use qubic_rpc::client::RpcClient;
///
/// let client = RpcClient::new("https://rpc.qubic.org");
/// ```
pub struct RpcClient {
    client: reqwest::Client,
    base_url: String,
}

impl RpcClient {
    pub fn new(base_url: &str) -> Self {
        let client = reqwest::Client::new();
        RpcClient {
            client,
            base_url: base_url.to_string(),
        }
    }
    /// Get the balance of a wallet
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::qubic_rpc_types::Balance;
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let wallet_id = "IJFDMOBGBPKVJFFUVFOXYJSFVQYAKRIBPRHXNPRXLALSKYDVLNNSUPBAQJFC".to_string();
    /// let balance: Balance = rpc_client.get_balance(wallet_id).await.unwrap();
    ///
    /// println!("{:#?}", balance);
    /// ```
    pub async fn get_balance(&self, wallet_id: String) -> Result<Balance> {
        let url = format!("{}/v1/balances/{}", self.base_url, wallet_id);
        let rpc_response: WalletBalance = self.client.get(&url).send().await?.json().await?;
        Ok(rpc_response.balance)
    }

    /// Get transaction information
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::qubic_rpc_types::TransactionResponseData;
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let tx_id = "yhxstxyqmofoihqxpmjkzbhpkejecdnxtmqxkguimcduomifdftpewhefvri".to_string();
    /// let tx: TransactionResponseData = rpc_client.get_transaction(tx_id).await.unwrap();
    ///
    /// println!("from: {}\nto: {}", tx.from, tx.to);
    /// ```
    pub async fn get_transaction(&self, tx_id: String) -> Result<TransactionResponseData> {
        let url = format!("{}/v2/transactions/{}", self.base_url, tx_id);
        let rpc_response: TransactionResponse = self.client.get(&url).send().await?.json().await?;
        Ok(rpc_response.transaction)
    }

    /// Get all transactions in tick
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::qubic_rpc_types::TransactionResponseData;
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let tick: u32 = 19969917;
    /// let txs: Vec<TransactionResponseData> =
    ///     rpc_client.get_tick_transactions(tick).await.unwrap();
    /// dbg!(&txs[0]);
    /// assert_eq!(txs.len(), 2);
    /// assert_eq!(
    ///     txs[0].source_id,
    ///     "FPFVZFNQFSIAUBMGNTZYLHXNFFSDFDMICEEOSIYYFALZUXKYAGHORKABCJGG"
    /// );
    /// ```
    pub async fn get_tick_transactions(&self, tick: u32) -> Result<Vec<TransactionResponseData>> {
        let url = format!("{}/v2/ticks/{}/transactions", self.base_url, tick);
        let rpc_response: TransactionsResponse = self.client.get(&url).send().await?.json().await?;
        Ok(rpc_response.into())
    }

    /// Get all approved transactions in tick
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::qubic_rpc_types::TransactionResponseData;
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let tick: u32 = 19969917;
    /// let txs: Vec<TransactionResponseData> =
    ///     rpc_client.get_tick_approved_transactions(tick).await.unwrap();
    /// dbg!(&txs[0]);
    /// assert_eq!(txs.len(), 1);
    /// assert_eq!(
    ///     txs[0].source_id,
    ///     "FPFVZFNQFSIAUBMGNTZYLHXNFFSDFDMICEEOSIYYFALZUXKYAGHORKABCJGG"
    /// );
    /// ```
    pub async fn get_tick_approved_transactions(
        &self,
        tick: u32,
    ) -> Result<Vec<TransactionResponseData>> {
        let url = format!(
            "{}/v2/ticks/{}/transactions?approved=true",
            self.base_url, tick
        );
        let rpc_response: TransactionsResponse = self.client.get(&url).send().await?.json().await?;
        Ok(rpc_response.into())
    }
    /// Get all transfer transactions in tick
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::qubic_rpc_types::TransactionResponseData;
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let tick: u32 = 19969917;
    /// let txs: Vec<TransactionResponseData> =
    ///     rpc_client.get_tick_transfer_transactions(tick).await.unwrap();
    /// dbg!(&txs[0]);
    /// assert_eq!(txs.len(), 1);
    /// assert_eq!(
    ///     txs[0].source_id,
    ///     "FPFVZFNQFSIAUBMGNTZYLHXNFFSDFDMICEEOSIYYFALZUXKYAGHORKABCJGG"
    /// );
    /// ```
    pub async fn get_tick_transfer_transactions(
        &self,
        tick: u32,
    ) -> Result<Vec<TransactionResponseData>> {
        let url = format!(
            "{}/v2/ticks/{}/transactions?transfers=true",
            self.base_url, tick
        );
        let rpc_response: TransactionsResponse = self.client.get(&url).send().await?.json().await?;
        Ok(rpc_response.into())
    }

    /// Get all transfer transactions for specific wallet ID
    /// within a range of ticks
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::qubic_rpc_types::TransactionResponseData;
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let wallet_id = "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ".to_string();
    /// let start_tick: u32 = 19385438;
    /// let end_tick: u32 = 19385439;
    /// let sc_only = false;
    /// let desc = false;
    /// let txs: Vec<TransactionResponseData> = rpc_client
    ///     .get_id_transfer_transactions(
    ///         wallet_id,
    ///         Some(start_tick),
    ///         Some(end_tick),
    ///         sc_only,
    ///         desc,
    ///     )
    ///     .await
    ///     .unwrap();
    /// assert_eq!(txs.len(), 1);
    /// assert_eq!(
    ///     txs[0].source_id,
    ///     "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ"
    /// );
    /// ```
    pub async fn get_id_transfer_transactions(
        &self,
        wallet_id: String,
        start_tick: Option<u32>,
        end_tick: Option<u32>,
        sc_only: bool,
        desc: bool,
    ) -> Result<Vec<TransactionResponseData>> {
        let mut url = Url::parse(&format!(
            "{}/v2/identities/{}/transfers",
            self.base_url, wallet_id
        ))?;

        if let Some(start) = start_tick {
            url.query_pairs_mut()
                .append_pair("start_tick", &start.to_string());
        }
        if let Some(end) = end_tick {
            url.query_pairs_mut()
                .append_pair("end_tick", &end.to_string());
        }
        url.query_pairs_mut()
            .append_pair("sc_only", &sc_only.to_string());
        url.query_pairs_mut().append_pair("desc", &desc.to_string());

        let rpc_response: TickTransactions = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.into())
    }

    /// Get tick data
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::qubic_rpc_types::TickData;
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let tick_data: TickData = rpc_client.get_tick_data(19969917).await.unwrap();
    /// assert_eq!(tick_data.epoch, 149);
    /// assert_eq!(tick_data.tick_number, 19969917);
    /// assert_eq!(tick_data.timestamp, "1740314534000");
    /// assert_eq!(tick_data.transaction_ids.len(), 2);
    /// assert_eq!(tick_data.signature_hex, "19a7106d59f3a9fd122cf1d3cef09801dde9e539fc5cc8ee4a298630203d66915fad741502a3db0a0258710d5d94f774babc6041dc7cf27cd84b6174279f2100");
    /// ```
    pub async fn get_tick_data(&self, tick: u32) -> Result<TickData> {
        let url = format!("{}/v1/ticks/{}/tick-data", self.base_url, tick);
        let rpc_response: TickDataWrapper = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.tick_data)
    }

    /// Get chain hash at a particular tick
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let hash: String = rpc_client.get_chain_hash(19969917).await.unwrap();
    /// assert_eq!(
    ///     hash,
    ///     "ecfa17369e1f2cd2bfc2dbd25287915bdcb885d4e0ab6ea70554cb755cf9343f"
    /// );
    /// ```
    pub async fn get_chain_hash(&self, tick: u32) -> Result<String> {
        let url = format!("{}/v2/ticks/{}/hash", self.base_url, tick);
        let rpc_response: Hash = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.hex_digest)
    }

    /// Get quorum data at a particular tick
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::QuorumTickData;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let quorum_tick_data: QuorumTickData =
    ///     rpc_client.get_quorum_tick_data(19969917).await.unwrap();
    /// assert_eq!(quorum_tick_data.quorum_tick_structure.epoch, 149);
    /// assert_eq!(quorum_tick_data.quorum_tick_structure.tick_number, 19969917);
    /// assert_eq!(quorum_tick_data.quorum_diff_per_computor.len(), 454);
    /// ```
    pub async fn get_quorum_tick_data(&self, tick: u32) -> Result<QuorumTickData> {
        let url = format!("{}/v2/ticks/{}/quorum-data", self.base_url, tick);
        let rpc_response: QuorumTickDataWrapper = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.quorum_tick_data)
    }

    /// Get store hash at a particular tick
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let hash: String = rpc_client.get_store_hash(19969917).await.unwrap();
    /// assert_eq!(
    ///     hash,
    ///     "c35f22dac0001ff37c695980cfdccc33e24eb78f83d181a027c32e7c484dd558"
    /// );
    /// ```
    pub async fn get_store_hash(&self, tick: u32) -> Result<String> {
        let url = format!("{}/v2/ticks/{}/store-hash", self.base_url, tick);
        let rpc_response: Hash = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.hex_digest)
    }

    /// Get issued, possessed, or owned assets for a wallet_id
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::{Asset, AssetType};
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let wallet_id = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB".to_string();
    /// let assets: Vec<Asset> = rpc_client
    ///     .get_assets(wallet_id, AssetType::Issued)
    ///     .await
    ///     .unwrap();
    /// assert!(assets.len() > 0);
    /// ```
    pub async fn get_assets(&self, wallet_id: String, asset_type: AssetType) -> Result<Vec<Asset>> {
        let url = format!(
            "{}/v1/assets/{}/{}",
            self.base_url,
            wallet_id,
            asset_type.to_string()
        );
        let rpc_response: Value = self.client.get(url).send().await?.json().await?;
        let response_key = format!("{}Assets", asset_type.to_string());
        let assets: Vec<Asset> = rpc_response
            .get(response_key)
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!(format!("{} key is missing", asset_type.to_string())))?
            .iter()
            .map(|v| serde_json::from_value(v.clone()))
            .collect::<Result<_, _>>()?;
        Ok(assets)
    }

    /// Get latest tick
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let latest_tick: u32 = rpc_client.get_latest_tick().await.unwrap();
    /// assert!(latest_tick > 0);
    /// ```
    pub async fn get_latest_tick(&self) -> Result<u32> {
        let url = format!("{}/v1/latestTick", self.base_url,);
        let rpc_response: LatestTick = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.latest_tick)
    }

    /// Get RPC server status
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::RpcStatus;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let status: RpcStatus = rpc_client.get_status().await.unwrap();
    /// assert!(status.empty_ticks_per_epoch.len() > 0);
    /// assert!(status.processed_tick_intervals_per_epoch.len() > 0);
    /// assert!(status.last_processed_ticks_per_epoch.len() > 0);
    /// assert!(status.last_processed_tick.tick_number > 0);
    /// assert!(status.skipped_ticks.len() > 0);
    /// ```
    pub async fn get_status(&self) -> Result<RpcStatus> {
        let url = format!("{}/v1/status", self.base_url,);
        let rpc_response: RpcStatus = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response)
    }

    /// Get RPC server health check (true if server is healthy)
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::RpcHealth;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let health_check: bool = rpc_client.get_health_check().await.unwrap();
    /// assert!(health_check);
    /// ```
    pub async fn get_health_check(&self) -> Result<bool> {
        let url = format!("{}/v1/healthcheck", self.base_url);
        let rpc_response: RpcHealth = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.status)
    }

    /// Get computors in epoch
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::ComputorInfos;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let computor_infos: ComputorInfos = rpc_client.get_computors(119).await.unwrap();
    /// assert_eq!(computor_infos.signature_hex, "0420902b687f71fdc6dc64ffe6ad0916ad1b98eea9c4b32af3f102855e7a6a41494fb5ae2a8fe9f228da9b42ed7df8c9f0c93e23f869d5b581f715a742071200");
    /// ```
    pub async fn get_computors(&self, epoch: u16) -> Result<ComputorInfos> {
        let url = format!("{}/v1/epochs/{}/computors", self.base_url, epoch);
        let rpc_response: ComputorsWrapper = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.computors)
    }

    /// Get current tick info
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::TickInfo;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let tick_info: TickInfo = rpc_client.get_current_tick_info().await.unwrap();
    /// assert!(tick_info.epoch > 0);
    /// assert!(tick_info.tick > 0);
    /// ```
    pub async fn get_current_tick_info(&self) -> Result<TickInfo> {
        let url = format!("{}/v1/tick-info", self.base_url);
        let rpc_response: TickInfoWrapper = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.tick_info)
    }

    /// Alias for `get_current_tick_info`
    pub async fn get_block_height(&self) -> Result<TickInfo> {
        self.get_current_tick_info().await
    }

    /// Get latest stats
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::LatestStats;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let latest_stats: LatestStats = rpc_client.get_latest_stats().await.unwrap();
    /// assert!(latest_stats.circulating_supply.len() > 0);
    /// assert!(latest_stats.active_addresses > 0);
    /// assert!(latest_stats.price > 0.0);
    /// ```
    pub async fn get_latest_stats(&self) -> Result<LatestStats> {
        let url = format!("{}/v1/latest-stats", self.base_url);
        let rpc_response: LatestStatsWrapper = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.data)
    }

    /// Get rich list
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::RichEntity;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let rich: Vec<RichEntity> = rpc_client.get_rich_list(None, None).await.unwrap();
    /// assert!(rich.len() > 0);
    /// ```
    ///
    /// You can also pass `page` and `page_size` for pagination
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::RichEntity;
    ///
    /// let rpc_client = RpcClient::new("https://rpc.qubic.org");
    /// let page = 2;
    /// let page_size = 100;
    /// let rich: Vec<RichEntity> = rpc_client.get_rich_list(Some(page), Some(page_size)).await.unwrap();
    /// assert!(rich.len() > 0);
    /// ```
    pub async fn get_rich_list(
        &self,
        page: Option<u32>,
        page_size: Option<u32>,
    ) -> Result<Vec<RichEntity>> {
        let mut url = Url::parse(&format!("{}/v1/rich-list", self.base_url))?;

        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(50);
        url.query_pairs_mut().append_pair("page", &page.to_string());
        url.query_pairs_mut()
            .append_pair("page_size", &page_size.to_string());

        let rpc_response: RichListWrapper = self.client.get(url).send().await?.json().await?;
        Ok(rpc_response.rich_list.entities)
    }

    /// Broadcast a transaction to the network
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_types::{traits::Sign, QubicWallet},
    /// use qubic_tcp_types::types::transactions::TransactionWithData,
    ///
    /// let wallet =
    ///     QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    ///         .unwrap();
    /// let mut tx = TransactionWithData::default(); // create new transaction tx
    /// let _ = tx.sign(&wallet); // sign transaction with wallet
    /// rpc_client.broadcast_transaction(tx).await.unwrap();
    /// ```
    pub async fn broadcast_transaction(&self, tx: TransactionWithData) -> Result<()> {
        let url = format!("{}/v1/broadcast-transaction", self.base_url);
        let mut payload = HashMap::new();
        payload.insert(
            "encodedTransaction",
            base64::engine::general_purpose::STANDARD.encode(tx.to_bytes()),
        );
        let _ = self
            .client
            .post(url)
            .json(&payload)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        Ok(())
    }

    /// Query a previously deployed smart contract
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use qubic_rpc::client::RpcClient;
    /// use qubic_rpc::qubic_rpc_types::SmartContract;
    ///
    /// let contract_index = 1;
    /// let input_type = 1;
    /// let input_size = 0;
    /// let request_data = "".to_string();
    /// let sc: SmartContract = rpc_client
    ///     .query_smart_contract(contract_index, input_type, input_size, request_data)
    ///     .await
    ///     .unwrap();
    ///
    /// println!("{:#?}", sc.response_data):
    /// ```
    pub async fn query_smart_contract(
        &self,
        contract_index: u32,
        input_type: u16,
        input_size: u16,
        request_data: String,
    ) -> Result<SmartContract> {
        let url = format!("{}/v1/querySmartContract", self.base_url);

        let mut payload = HashMap::new();
        payload.insert("contractIndex", contract_index.to_string());
        payload.insert("inputType", input_type.to_string());
        payload.insert("inputSize", input_size.to_string());
        payload.insert(
            "requestData",
            base64::engine::general_purpose::STANDARD.encode(request_data.as_bytes()),
        );
        let smart_contract: SmartContract = self
            .client
            .post(url)
            .json(&payload)
            .header("Content-Type", "application/json")
            .send()
            .await?
            .json()
            .await?;
        Ok(smart_contract)
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        client::RpcClient,
        qubic_rpc_types::{
            Asset, AssetType, Balance, ComputorInfos, LatestStats, QuorumTickData, RichEntity,
            RpcStatus, SmartContract, TickData, TickInfo, TransactionResponseData,
        },
    };
    use qubic_rs::{
        qubic_tcp_types::types::transactions::TransactionWithData,
        qubic_types::{traits::Sign, QubicWallet},
    };

    fn setup() -> RpcClient {
        RpcClient::new("https://rpc.qubic.org")
    }
    #[tokio::test]
    async fn get_balance() {
        let rpc_client = setup();
        let wallet_id = "IJFDMOBGBPKVJFFUVFOXYJSFVQYAKRIBPRHXNPRXLALSKYDVLNNSUPBAQJFC".to_string();
        let balance: Balance = rpc_client.get_balance(wallet_id).await.unwrap();
        assert!(balance.balance.len() > 0);
    }
    #[tokio::test]
    async fn get_transaction() {
        let rpc_client = setup();
        let tx_id = "yhxstxyqmofoihqxpmjkzbhpkejecdnxtmqxkguimcduomifdftpewhefvri".to_string();
        let tx: TransactionResponseData = rpc_client.get_transaction(tx_id).await.unwrap();
        assert_eq!(
            tx.source_id,
            "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ"
        );
        assert_eq!(
            tx.dest_id,
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB"
        );
    }
    #[tokio::test]
    async fn get_tick_transactions() {
        let rpc_client = setup();
        let tick: u32 = 19969917;
        let txs: Vec<TransactionResponseData> =
            rpc_client.get_tick_transactions(tick).await.unwrap();
        assert_eq!(txs.len(), 2);
        assert_eq!(
            txs[0].source_id,
            "FPFVZFNQFSIAUBMGNTZYLHXNFFSDFDMICEEOSIYYFALZUXKYAGHORKABCJGG"
        );
    }
    #[tokio::test]
    async fn get_tick_approved_transactions() {
        let rpc_client = setup();
        let tick: u32 = 19969917;
        let txs: Vec<TransactionResponseData> = rpc_client
            .get_tick_approved_transactions(tick)
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(
            txs[0].source_id,
            "FPFVZFNQFSIAUBMGNTZYLHXNFFSDFDMICEEOSIYYFALZUXKYAGHORKABCJGG"
        );
    }
    #[tokio::test]
    async fn get_tick_transfer_transactions() {
        let rpc_client = setup();
        let tick: u32 = 19969917;
        let txs: Vec<TransactionResponseData> = rpc_client
            .get_tick_transfer_transactions(tick)
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(
            txs[0].source_id,
            "FPFVZFNQFSIAUBMGNTZYLHXNFFSDFDMICEEOSIYYFALZUXKYAGHORKABCJGG"
        );
    }
    #[tokio::test]
    async fn get_id_transfer_transactions() {
        let rpc_client = setup();
        let wallet_id = "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ".to_string();
        let start_tick: u32 = 19385438;
        let end_tick: u32 = 19385439;
        let sc_only = false;
        let desc = false;
        let txs: Vec<TransactionResponseData> = rpc_client
            .get_id_transfer_transactions(
                wallet_id,
                Some(start_tick),
                Some(end_tick),
                sc_only,
                desc,
            )
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
        assert_eq!(
            txs[0].source_id,
            "FGKEMNSAUKDCXFPJPHHSNXOLPRECNPJXPIVJRGKFODFFVKWLSOGAJEQAXFIJ"
        );
    }

    #[tokio::test]
    async fn get_tick_data() {
        let rpc_client = setup();
        let tick_data: TickData = rpc_client.get_tick_data(19969917).await.unwrap();
        assert_eq!(tick_data.epoch, 149);
        assert_eq!(tick_data.tick_number, 19969917);
        assert_eq!(tick_data.timestamp, "1740314534000");
        assert_eq!(tick_data.transaction_ids.len(), 2);
        assert_eq!(tick_data.signature_hex, "19a7106d59f3a9fd122cf1d3cef09801dde9e539fc5cc8ee4a298630203d66915fad741502a3db0a0258710d5d94f774babc6041dc7cf27cd84b6174279f2100");
    }

    #[tokio::test]
    async fn get_chain_hash() {
        let rpc_client = setup();
        let hash: String = rpc_client.get_chain_hash(19969917).await.unwrap();
        assert_eq!(
            hash,
            "ecfa17369e1f2cd2bfc2dbd25287915bdcb885d4e0ab6ea70554cb755cf9343f"
        );
    }
    #[tokio::test]
    async fn get_quorum_data() {
        let rpc_client = setup();
        let quorum_tick_data: QuorumTickData =
            rpc_client.get_quorum_tick_data(19969917).await.unwrap();
        assert_eq!(quorum_tick_data.quorum_tick_structure.epoch, 149);
        assert_eq!(quorum_tick_data.quorum_tick_structure.tick_number, 19969917);
    }
    #[tokio::test]
    async fn get_store_hash() {
        let rpc_client = setup();
        let hash: String = rpc_client.get_store_hash(19969917).await.unwrap();
        assert_eq!(
            hash,
            "c35f22dac0001ff37c695980cfdccc33e24eb78f83d181a027c32e7c484dd558"
        );
    }
    #[tokio::test]
    async fn get_assets() {
        let rpc_client = setup();
        let wallet_id = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFXIB".to_string();
        let assets: Vec<Asset> = rpc_client
            .get_assets(wallet_id, AssetType::Issued)
            .await
            .unwrap();
        assert!(assets.len() > 0);
    }

    #[tokio::test]
    async fn get_latest_tick() {
        let rpc_client = setup();
        let latest_tick: u32 = rpc_client.get_latest_tick().await.unwrap();
        assert!(latest_tick > 0);
    }

    #[tokio::test]
    async fn get_status() {
        let rpc_client = setup();
        let status: RpcStatus = rpc_client.get_status().await.unwrap();
        assert!(status.empty_ticks_per_epoch.len() > 0);
        assert!(status.processed_tick_intervals_per_epoch.len() > 0);
        assert!(status.last_processed_ticks_per_epoch.len() > 0);
        assert!(status.last_processed_tick.tick_number > 0);
        assert!(status.skipped_ticks.len() > 0);
    }

    #[tokio::test]
    async fn get_health_check() {
        let rpc_client = setup();
        let health_check: bool = rpc_client.get_health_check().await.unwrap();
        assert!(health_check);
    }

    #[tokio::test]
    async fn get_computors() {
        let rpc_client = setup();
        let computor_infos: ComputorInfos = rpc_client.get_computors(119).await.unwrap();
        assert_eq!(computor_infos.signature_hex, "0420902b687f71fdc6dc64ffe6ad0916ad1b98eea9c4b32af3f102855e7a6a41494fb5ae2a8fe9f228da9b42ed7df8c9f0c93e23f869d5b581f715a742071200");
    }

    #[tokio::test]
    async fn get_current_tick_info() {
        let rpc_client = setup();
        let tick_info: TickInfo = rpc_client.get_current_tick_info().await.unwrap();
        assert!(tick_info.epoch > 0);
        assert!(tick_info.tick > 0);
    }

    #[tokio::test]
    async fn get_latest_stats() {
        let rpc_client = setup();
        let latest_stats: LatestStats = rpc_client.get_latest_stats().await.unwrap();
        assert!(latest_stats.circulating_supply.len() > 0);
        assert!(latest_stats.active_addresses > 0);
        assert!(latest_stats.price > 0.0);
    }

    #[tokio::test]
    async fn get_rich_list() {
        let rpc_client = setup();
        let rich: Vec<RichEntity> = rpc_client.get_rich_list(None, None).await.unwrap();
        assert!(rich.len() > 0);
    }

    #[tokio::test]
    async fn broadcast_transaction() {
        let rpc_client = setup();
        let wallet =
            QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap();
        let mut tx = TransactionWithData::default(); // create new transaction tx
        let _ = tx.sign(&wallet); // sign transaction with wallet
        rpc_client.broadcast_transaction(tx).await.unwrap();
    }

    #[tokio::test]
    async fn query_smart_contract() {
        let rpc_client = setup();
        let contract_index = 1;
        let input_type = 1;
        let input_size = 0;
        let request_data = "".to_string();
        let _sc: SmartContract = rpc_client
            .query_smart_contract(contract_index, input_type, input_size, request_data)
            .await
            .unwrap();
        // smart contracts are not required to provide responseData. If this test doesn't panic the
        // endpoint is likely fine
    }
}
