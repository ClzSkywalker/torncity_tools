use tools::cfg::CfgTool;

pub struct Weav3rSettingData {
    cfg: CfgTool,
}

impl Weav3rSettingData {
    pub const SETTINGS_PATH: &str = "user://settings.cfg";
    const SECTION: &str = "settings";
    const KEY_INTERVAL: &str = "interval";
    const KEY_PROFIT_PERCENT: &str = "profit_percent";
    const KEY_MIN_PROFIT: &str = "min_profit";
    const KEY_FILTER_IDS: &str = "filter_ids";
    const KEY_AUDIO_SWITCH: &str = "audio_switch";
    const DEFAULT_AUDIO_SWITCH: bool = true;
    const DEFAULT_INTERVAL: f64 = 5.0;
    const DEFAULT_PROFIT_PERCENT: f32 = 1.0;
    const DEFAULT_MIN_PROFIT: i64 = 10000;
    const DEFAULT_FILTER_IDS: &str = "385,183,97,902,901,904,129,184,260,903,263,617,272,264,271,267,277,282,276,186,187,215,261,618,273,258,266,268,269,281,274,384,533,555,532,554,530,553,987,986,985,206,586,587,151,556,529,528,36,527,310,35,210,39,37,209,38,541,552,542,638,551,531,550,818,283,370,364,1080,1079,1082,1083,1078,1081,367,366,1485,1486,1494,358";
    const KEY_NEXT_ACTION: &str = "next_action";
    const DEFAULT_NEXT_ACTION: &str = "404cb5d1e07e9049af7adcc4201bc257fc4af6aa67";
    const KEY_OFFICE_SELL_PRICE: &str = "office_sell_price";
    const DEFAULT_OFFICE_SELL_PRICE: u64 = 5000;
}

impl Weav3rSettingData {
    pub fn new(cfg: CfgTool) -> Self {
        Self { cfg }
    }
    pub fn get_interval(&self) -> f64 {
        self.cfg.read_config_f64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_INTERVAL,
            Weav3rSettingData::DEFAULT_INTERVAL,
        )
    }
    pub fn set_interval(&mut self, interval: f64) {
        self.cfg.write_config_f64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_INTERVAL,
            interval,
        );
    }
    pub fn get_profit_percent(&self) -> f32 {
        self.cfg.read_config_f32(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_PROFIT_PERCENT,
            Weav3rSettingData::DEFAULT_PROFIT_PERCENT,
        )
    }
    pub fn set_profit_percent(&mut self, profit_percent: f32) {
        self.cfg.write_config_f32(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_PROFIT_PERCENT,
            profit_percent,
        );
    }
    pub fn get_min_profit(&self) -> i64 {
        self.cfg.read_config_i64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_MIN_PROFIT,
            Weav3rSettingData::DEFAULT_MIN_PROFIT,
        )
    }
    pub fn set_min_profit(&mut self, min_profit: i64) {
        self.cfg.write_config_i64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_MIN_PROFIT,
            min_profit,
        );
    }
    pub fn get_filter_ids(&self) -> String {
        self.cfg.read_config_string(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_FILTER_IDS,
            Weav3rSettingData::DEFAULT_FILTER_IDS,
        )
    }
    pub fn set_filter_ids(&mut self, filter_ids: &str) {
        self.cfg.write_config_string(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_FILTER_IDS,
            filter_ids,
        );
    }

    pub fn get_audio_switch(&self) -> bool {
        self.cfg.read_config_bool(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_AUDIO_SWITCH,
            Weav3rSettingData::DEFAULT_AUDIO_SWITCH,
        )
    }
    pub fn set_audio_switch(&mut self, audio_switch: bool) {
        self.cfg.write_config_bool(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_AUDIO_SWITCH,
            audio_switch,
        );
    }

    pub fn get_next_action(&self) -> String {
        self.cfg.read_config_string(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_NEXT_ACTION,
            Weav3rSettingData::DEFAULT_NEXT_ACTION,
        )
    }

    pub fn set_next_action(&mut self, next_action: &str) {
        self.cfg.write_config_string(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_NEXT_ACTION,
            next_action,
        );
    }

    pub fn get_office_sell_price(&self) -> u64 {
        self.cfg.read_config_u64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_OFFICE_SELL_PRICE,
            Weav3rSettingData::DEFAULT_OFFICE_SELL_PRICE,
        )
    }
    pub fn set_office_sell_price(&mut self, office_sell_price: u64) {
        self.cfg.write_config_u64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_OFFICE_SELL_PRICE,
            office_sell_price,
        );
    }

    pub fn save(&mut self) -> Result<(), godot::global::Error> {
        self.cfg.save()?;
        Ok(())
    }
}
