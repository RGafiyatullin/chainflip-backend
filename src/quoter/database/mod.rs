use self::types::TransactionType;

use super::{BlockProcessor, StateProvider};
use crate::{
    common::store::utils::SQLite as KVS,
    side_chain::{SideChainBlock, SideChainTx},
};
use rusqlite::{self, Error, Transaction};
use rusqlite::{params, Connection};
use serde::Deserialize;
use uuid::Uuid;

mod migration;
mod types;

/// A database for storing and accessing local state
#[derive(Debug)]
pub struct Database {
    connection: Connection,
}

impl Database {
    /// Returns a database instance from the given path.
    pub fn open(file: &str) -> Self {
        let connection = Connection::open(file).expect("Could not open the database");
        Database::new(connection)
    }

    /// Returns a database instance with the given connection.
    pub fn new(mut connection: Connection) -> Self {
        migration::migrate_database(&mut connection);
        KVS::create_kvs_table(&connection);
        Database { connection }
    }

    fn set_last_processed_block_number(&self, block_number: u32) -> Result<(), String> {
        KVS::set_data(
            &self.connection,
            "last_processed_block_number",
            Some(block_number),
        )
    }

    fn process_transactions(db: &Transaction, txs: &[SideChainTx]) {
        for tx in txs {
            match tx {
                SideChainTx::PoolChangeTx(tx) => {
                    let serialized = serde_json::to_string(tx).unwrap();
                    Database::insert_transaction(db, tx.id, None, tx.into(), serialized)
                }
                SideChainTx::QuoteTx(tx) => {
                    let serialized = serde_json::to_string(tx).unwrap();
                    Database::insert_transaction(db, tx.id, None, tx.into(), serialized)
                }
                SideChainTx::StakeQuoteTx(tx) => {
                    let serialized = serde_json::to_string(tx).unwrap();
                    Database::insert_transaction(db, tx.id, None, tx.into(), serialized)
                }
                SideChainTx::WitnessTx(tx) => {
                    let serialized = serde_json::to_string(tx).unwrap();
                    Database::insert_transaction(
                        db,
                        tx.id,
                        Some(tx.quote_id.to_string()),
                        tx.into(),
                        serialized,
                    )
                }
                SideChainTx::OutputTx(tx) => {
                    let serialized = serde_json::to_string(tx).unwrap();
                    Database::insert_transaction(
                        db,
                        tx.id,
                        Some(tx.quote_tx.to_string()),
                        tx.into(),
                        serialized,
                    )
                }
                SideChainTx::OutputSentTx(tx) => {
                    let serialized = serde_json::to_string(tx).unwrap();
                    Database::insert_transaction(db, tx.id, None, tx.into(), serialized)
                }
                _ => warn!("Failed to process transaction: {:?}", tx),
            }
        }
    }

    fn insert_transaction(
        db: &Transaction,
        uuid: Uuid,
        meta: Option<String>,
        tx_type: TransactionType,
        data: String,
    ) {
        db.execute(
            "INSERT OR REPLACE INTO transactions (id, meta, type, data) VALUES (?1, ?2, ?3, ?4)",
            params![uuid.to_string(), meta, tx_type.to_string(), data],
        )
        .expect("Failed to create statement");
    }

    fn deserialize<'a, T: Deserialize<'a>>(data: &'a str) -> Option<T> {
        match serde_json::from_str::<T>(data) {
            Ok(block) => Some(block),
            Err(err) => {
                error!("Failed to parse json: {}", err);
                None
            }
        }
    }
}

impl BlockProcessor for Database {
    fn get_last_processed_block_number(&self) -> Option<u32> {
        KVS::get_data(&self.connection, "last_processed_block_number")
    }

    fn process_blocks(&mut self, blocks: &[SideChainBlock]) -> Result<(), String> {
        let tx = match self.connection.transaction() {
            Ok(transaction) => transaction,
            Err(err) => {
                error!("Failed to open database transaction: {}", err);
                return Err("Failed to process block".to_owned());
            }
        };

        for block in blocks.iter() {
            Database::process_transactions(&tx, &block.txs)
        }

        if let Err(err) = tx.commit() {
            error!("Failed to commit process block changes: {}", err);
            return Err("Failed to commit process block changes".to_owned());
        };

        let last_block_number = blocks.iter().map(|b| b.id).max();
        if let Some(last_block_number) = last_block_number {
            self.set_last_processed_block_number(last_block_number)?;
        }

        Ok(())
    }
}

impl StateProvider for Database {
    fn get_swap_quotes(&self) -> Option<Vec<crate::transactions::QuoteTx>> {
        todo!()
    }

    fn get_swap_quote_tx(&self, id: String) -> Option<crate::transactions::QuoteTx> {
        todo!()
    }

    fn get_stake_quotes(&self) -> Option<Vec<crate::transactions::StakeQuoteTx>> {
        todo!()
    }

    fn get_stake_quote_tx(&self, id: String) -> Option<crate::transactions::StakeQuoteTx> {
        todo!()
    }

    fn get_witness_txs(&self, quote_id: String) -> Option<Vec<crate::transactions::WitnessTx>> {
        todo!()
    }

    fn get_output_txs(&self, quote_id: String) -> Option<Vec<crate::transactions::OutputTx>> {
        todo!()
    }

    fn get_output_sent_txs(&self) -> Option<Vec<crate::transactions::OutputSentTx>> {
        todo!()
    }

    fn get_pools(
        &self,
    ) -> std::collections::HashMap<crate::common::PoolCoin, crate::common::Liquidity> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;
    use crate::{
        common::Coin,
        common::LokiPaymentId,
        common::PoolCoin,
        common::Timestamp,
        common::WalletAddress,
        transactions::{OutputSentTx, PoolChangeTx, StakeQuoteTx, WitnessTx},
        utils::test_utils::create_fake_output_tx,
        utils::test_utils::create_fake_quote_tx_eth_loki,
        utils::test_utils::TEST_ETH_ADDRESS,
    };
    use rusqlite::NO_PARAMS;

    fn setup() -> Database {
        let connection = Connection::open_in_memory().expect("Failed to open connection");
        Database::new(connection)
    }

    struct RawData {
        id: String,
        meta: Option<String>,
        data: String,
    }

    #[test]
    fn inserts_transaction() {
        let mut db = setup();
        let tx = db.connection.transaction().unwrap();

        let uuid = Uuid::new_v4();
        Database::insert_transaction(&tx, uuid, None, TransactionType::PoolChange, "Hello".into());

        tx.commit().unwrap();

        let results = db
            .connection
            .query_row(
                "select id, meta, data from transactions",
                NO_PARAMS,
                |row| {
                    Ok(RawData {
                        id: row.get(0).unwrap(),
                        meta: row.get(1).unwrap(),
                        data: row.get(2).unwrap(),
                    })
                },
            )
            .unwrap();

        assert_eq!(results.id, uuid.to_string());
        assert_eq!(&results.data, "Hello");
        assert!(results.meta.is_none());
    }

    #[test]
    fn processes_blocks() {
        let mut db = setup();

        assert!(db.get_last_processed_block_number().is_none());

        let blocks: Vec<SideChainBlock> = vec![
            SideChainBlock { id: 1, txs: vec![] },
            SideChainBlock { id: 2, txs: vec![] },
            SideChainBlock {
                id: 10,
                txs: vec![],
            },
        ];

        db.process_blocks(&blocks).unwrap();

        assert_eq!(db.get_last_processed_block_number(), Some(10));
    }

    #[test]
    fn processes_transactions() {
        let mut db = setup();
        let tx = db.connection.transaction().unwrap();

        let payment_id = LokiPaymentId::from_str("60900e5603bf96e3").unwrap();

        let transactions: Vec<SideChainTx> = vec![
            PoolChangeTx::new(PoolCoin::BTC, 100, -100).into(),
            create_fake_quote_tx_eth_loki().into(), // Quote Tx
            StakeQuoteTx::new(payment_id, 100, PoolCoin::BTC, 200, "id".to_owned()).into(),
            WitnessTx::new(
                Timestamp::now(),
                Uuid::new_v4(),
                "txid".to_owned(),
                0,
                0,
                100,
                Coin::ETH,
                None,
            )
            .into(),
            create_fake_output_tx(Coin::ETH).into(), // Output tx
            OutputSentTx::new(
                Timestamp::now(),
                vec![Uuid::new_v4()],
                Coin::ETH,
                WalletAddress::new(TEST_ETH_ADDRESS),
                100,
                0,
                "txid".to_owned(),
            )
            .unwrap()
            .into(),
        ];

        Database::process_transactions(&tx, &transactions);

        tx.commit().expect("Expected transactions to be added");

        let count: u32 = db
            .connection
            .query_row("SELECT COUNT(*) from transactions", NO_PARAMS, |r| r.get(0))
            .unwrap();

        assert_eq!(count, 6);
    }
}
