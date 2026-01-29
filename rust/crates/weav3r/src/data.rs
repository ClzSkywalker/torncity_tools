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
    const DEFAULT_INTERVAL: f64 = 5.0;
    const DEFAULT_PROFIT_PERCENT: f64 = 1.0;
    const DEFAULT_MIN_PROFIT: i64 = 10000;
    const DEFAULT_FILTER_IDS: &str = "385,183,97,902,901,904,129,184,260,903,263,617,272,264,271,267,277,282,276,186,187,215,261,618,273,258,266,268,269,281,274,384,533,555,532,554,530,553,987,986,985,206,586,587,151,556,529,528,36,527,310,35,210,39,37,209,38,541,552,542,638,551,531,550,818,283,370,364,1080,1079,1082,1083,1078,1081,367,366,1485,1486,1494,358";
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
    pub fn get_profit_percent(&self) -> f64 {
        self.cfg.read_config_f64(
            Weav3rSettingData::SECTION,
            Weav3rSettingData::KEY_PROFIT_PERCENT,
            Weav3rSettingData::DEFAULT_PROFIT_PERCENT,
        )
    }
    pub fn set_profit_percent(&mut self, profit_percent: f64) {
        self.cfg.write_config_f64(
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

    pub fn save(&mut self) -> Result<(), godot::global::Error> {
        self.cfg.save()?;
        Ok(())
    }
}
