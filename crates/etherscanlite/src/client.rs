//! The RPC client. Takes local types, returns local types.
//!
//! This module and `boundary.rs` are the only two that may name an alloy type (R4). It
//! exists because the alternative — calling the provider from `main.rs` — leaks alloy
//! types into `main` by *inference* even when no `alloy_*` path appears in the source, so
//! the isolation grep would pass while the isolation was already broken.

use anyhow::{Context, Result};
use core::future::IntoFuture;
use eth_primitives::{Address, B256, U256};

use crate::boundary::{FromAlloy, ToAlloy, TryFromAlloy, TxSummary};
use crate::retry;

pub(crate) struct Client<P> {
    provider: P,
}

impl Client<()> {
    /// Read-only calls, so the recommended fillers (gas estimation, nonce management,
    /// chain-id) are dead weight — they add an `eth_chainId` round-trip at construction.
    pub(crate) fn connect(rpc_url: &str) -> Result<Client<impl alloy_provider::Provider>> {
        let url = rpc_url.parse().context("ETH_RPC_URL is not a valid URL")?;
        let provider = alloy_provider::ProviderBuilder::new()
            .disable_recommended_fillers()
            .connect_http(url);

        Ok(Client { provider })
    }
}

impl<P: alloy_provider::Provider> Client<P> {
    pub(crate) async fn balance(&self, address: Address) -> Result<U256> {
        let wei = retry::with_backoff(|| self.provider.get_balance(address.to_alloy()).into_future())
            .await
            .context("eth_getBalance failed")?;

        Ok(<U256 as FromAlloy<_>>::from(wei))
    }

    pub(crate) async fn nonce(&self, address: Address) -> Result<u64> {
        retry::with_backoff(|| {
            self.provider
                .get_transaction_count(address.to_alloy())
                .into_future()
        })
        .await
        .context("eth_getTransactionCount failed")
    }

    pub(crate) async fn transaction(&self, hash: B256) -> Result<Option<TxSummary>> {
        let tx = retry::with_backoff(|| self.provider.get_transaction_by_hash(hash.to_alloy()))
            .await
            .context("eth_getTransactionByHash failed")?;

        tx.map(<TxSummary as TryFromAlloy<_>>::try_from)
            .transpose()
            .context("converting transaction")
    }
}
