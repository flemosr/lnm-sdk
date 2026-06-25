use serde::Deserialize;

/// Wallet deposit event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamWalletDeposit {
    currency: String,
    network: String,
    id: String,
    amount: f64,
    balance: f64,
    status: String,
    #[serde(default)]
    tx_id: Option<String>,
}

impl StreamWalletDeposit {
    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn network(&self) -> &str {
        &self.network
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn tx_id(&self) -> Option<&str> {
        self.tx_id.as_deref()
    }
}

/// Wallet withdrawal event notification payload.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamWalletWithdrawal {
    currency: String,
    network: String,
    id: String,
    amount: f64,
    fee: f64,
    balance: f64,
    status: String,
    #[serde(default)]
    tx_id: Option<String>,
}

impl StreamWalletWithdrawal {
    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn network(&self) -> &str {
        &self.network
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn fee(&self) -> f64 {
        self.fee
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn tx_id(&self) -> Option<&str> {
        self.tx_id.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_wallet_deposit_deserializes() {
        let deposit: StreamWalletDeposit = serde_json::from_str(
            r#"{ "currency": "btc", "network": "lightning", "id": "deposit-1", "amount": 1, "balance": 2, "status": "confirmed", "txId": "tx-1" }"#,
        )
        .expect("must deserialize wallet deposit");

        assert_eq!(deposit.currency(), "btc");
        assert_eq!(deposit.network(), "lightning");
        assert_eq!(deposit.id(), "deposit-1");
        assert_eq!(deposit.amount(), 1.0);
        assert_eq!(deposit.balance(), 2.0);
        assert_eq!(deposit.status(), "confirmed");
        assert_eq!(deposit.tx_id(), Some("tx-1"));
    }

    #[test]
    fn stream_wallet_withdrawal_deserializes() {
        let withdrawal: StreamWalletWithdrawal = serde_json::from_str(
            r#"{ "currency": "btc", "network": "lightning", "id": "withdrawal-1", "amount": 1, "fee": 2, "balance": 3, "status": "confirmed", "txId": "tx-1" }"#,
        )
        .expect("must deserialize wallet withdrawal");

        assert_eq!(withdrawal.currency(), "btc");
        assert_eq!(withdrawal.network(), "lightning");
        assert_eq!(withdrawal.id(), "withdrawal-1");
        assert_eq!(withdrawal.amount(), 1.0);
        assert_eq!(withdrawal.fee(), 2.0);
        assert_eq!(withdrawal.balance(), 3.0);
        assert_eq!(withdrawal.status(), "confirmed");
        assert_eq!(withdrawal.tx_id(), Some("tx-1"));
    }
}
