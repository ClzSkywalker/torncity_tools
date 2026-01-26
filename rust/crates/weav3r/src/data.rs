pub struct Weav3rSettingData;

impl Weav3rSettingData{
   pub const SETTINGS_PATH: &str = "user://settings.cfg";
    pub const SECTION: &str = "settings";
    pub const KEY_INTERVAL: &str = "interval";
    pub const KEY_PROFIT_PERCENT: &str = "profit_percent";
    pub const KEY_MIN_PROFIT: &str = "min_profit";
    pub const KEY_FILTER_IDS: &str = "filter_ids";
    pub const DEFAULT_INTERVAL: f64 = 5.0;
    pub const DEFAULT_PROFIT_PERCENT: f64 = 1.0;
    pub const DEFAULT_MIN_PROFIT: i32 = 10000;
    pub const DEFAULT_FILTER_IDS: &str = "385,183,97,902,901,904,129,184,260,903,263,617,272,264,271,267,277,282,276,186,187,215,261,618,273,258,266,268,269,281,274,384,533,555,532,554,530,553,987,986,985,206,586,587,151,556,529,528,36,527,310,35,210,39,37,209,38,541,552,542,638,551,531,550,818,283,370,364,1080,1079,1082,1083,1078,1081,367,366,1485,1486,1494,358";
}