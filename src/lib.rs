use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use utils::{PoolPriceInfo, PoolTimeStr};

pub const WSOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

/// 将不同 Quote 构成的内盘池子抽象成统一参数
#[derive(Debug, Clone)]
pub struct PumpQuoteConfig {
    pub quote_mint: Pubkey,
    pub max_quote_reserves: u64,
    pub virtual_initial_quote: u64,
    pub virtual_initial_base: u64,
}

impl PumpQuoteConfig {
    /// 计算业务定好的比例 (rate)，换算成精确的绝对计价最小单位 (如 Lamports，USDC uUnit)
    #[inline]
    pub fn rate_of(&self, rate: f64) -> u64 {
        (self.max_quote_reserves as f64 * rate) as u64
    }

    /// 状态一：内盘刚发币的初始虚拟 AMM 状态（零状态）
    pub fn pump_initial_pool_info(&self) -> PoolPriceInfo {
        let mut info = PoolPriceInfo {
            pool_address: Pubkey::default(),
            base_mint: Pubkey::default(),
            quote_mint: self.quote_mint,
            base_reserve: self.virtual_initial_base,
            quote_reserve: self.virtual_initial_quote,
            base_price_in_quote: 0.0,
            last_updated: PoolTimeStr(0),
        };
        info.update_price();
        info
    }

    /// 状态二：内盘打满时的状态（Quote为最大值，Base库存耗尽为0）
    pub fn pump_full_pool_info(&self) -> PoolPriceInfo {
        let mut info = PoolPriceInfo {
            pool_address: Pubkey::default(),
            base_mint: Pubkey::default(),
            quote_mint: self.quote_mint,
            base_reserve: 0,
            quote_reserve: self.max_quote_reserves,
            base_price_in_quote: 0.0,
            last_updated: PoolTimeStr(0),
        };
        info.update_price(); // 这里由于 base_reserve 为 0 可能会 Infinity，外层使用需注意场景
        info
    }

    /// 状态三：刚迁移到外盘的初始状态
    /// Quote 继承真实满状态额度，Base 真实事件校验为 206.9M（精度6位：206,900,000,000,000）
    pub fn migrated_pool(&self) -> PoolPriceInfo {
        let mut info = PoolPriceInfo {
            pool_address: Pubkey::default(),
            base_mint: Pubkey::default(),
            quote_mint: self.quote_mint,
            base_reserve: 206_900_000_000_000, // 从外盘真实 createPoolEvent 验证: 206900000000000
            quote_reserve: self.max_quote_reserves, // 继承内盘上限满容量
            base_price_in_quote: 0.0,
            last_updated: PoolTimeStr(0),
        };
        info.update_price();
        info
    }
}

/// 全局工厂：传入 quote_mint 返回通用配置，不符合返回 None
pub fn get_quote_config(quote_mint: &Pubkey) -> Option<PumpQuoteConfig> {
    let default_pubkey = Pubkey::default();

    if quote_mint == &WSOL_MINT || quote_mint == &default_pubkey {
        Some(PumpQuoteConfig {
            quote_mint: WSOL_MINT,
            max_quote_reserves: 85_000_000_000,    // 85 SOL * 10^9
            virtual_initial_quote: 30_000_000_000, // 30 SOL * 10^9
            virtual_initial_base: 1_073_000_000_000_000, // 1073M * 10^6
        })
    } else if quote_mint == &USDC_MINT {
        Some(PumpQuoteConfig {
            quote_mint: USDC_MINT,
            max_quote_reserves: 12_161_433_377, // 12,161 USDC * 10^6
            virtual_initial_quote: 4_292_000_000, // 根据真实事件 4292 USDC * 10^6
            virtual_initial_base: 1_073_000_000_000_000, // 1073M * 10^6
        })
    } else {
        None
    }
}
