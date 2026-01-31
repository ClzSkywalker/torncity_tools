use godot::{classes::ConfigFile, prelude::*};

pub struct CfgTool {
    pub config_path: String,
    file: Gd<ConfigFile>,
}

impl CfgTool {
    pub fn new(config_path: &'static str) -> Result<Self, godot::global::Error> {
        let mut config = ConfigFile::new_gd();
        let err = config.load(config_path);
        // 不存在则创建
        if err == godot::global::Error::ERR_FILE_NOT_FOUND {
            config.save(config_path);
        } else if err != godot::global::Error::OK {
            return Err(err);
        }
        Ok(Self {
            config_path: config_path.to_string(),
            file: config,
        })
    }

    pub fn read_config_f64(&self, section: &str, key: &str, default: f64) -> f64 {
        self.read_config(section, key, default)
    }
    pub fn write_config_f64(&mut self, section: &str, key: &str, value: f64) {
        self.write_config(section, key, value)
    }


    pub fn read_config_i32(&self, section: &str, key: &str, default: i32) -> i32 {
        self.read_config(section, key, default)
    }
    pub fn write_config_i32(&mut self, section: &str, key: &str, value: i32) {
        self.write_config(section, key, value)
    }

    pub fn read_config_i64(&self, section: &str, key: &str, default: i64) -> i64 {
        self.read_config(section, key, default)
    }
    pub fn write_config_i64(&mut self, section: &str, key: &str, value: i64) {
        self.write_config(section, key, value)
    }

    pub fn read_config_u64(&self, section: &str, key: &str, default: u64) -> u64 {
        self.read_config(section, key, default)
    }
    pub fn write_config_u64(&mut self, section: &str, key: &str, value: u64) {
        self.write_config(section, key, value)
    }

    pub fn read_config_string(&self, section: &str, key: &str, default: &str) -> String {
        let value: GString = self.read_config(section, key, GString::from(default));
        value.to_string()
    }
    pub fn write_config_string(&mut self, section: &str, key: &str, value: &str) {
        self.write_config(section, key, GString::from(value))
    }

    pub fn read_config_bool(&self, section: &str, key: &str, default: bool) -> bool {
        self.read_config(section, key, default)
    }
    pub fn write_config_bool(&mut self, section: &str, key: &str, value: bool) {
        self.write_config(section, key, value)
    }

    pub fn save(&mut self) -> Result<(), godot::global::Error> {
        let err = self.file.save(&self.config_path);
        if err == godot::global::Error::OK {
            Ok(())
        } else {
            Err(err)
        }
    }

    fn read_config<T>(&self, section: &str, key: &str, default: T) -> T
    where
        T: ToGodot + FromGodot,
    {
        let default_value = default.to_variant();
        self.file
            .get_value_ex(section, key)
            .default(&default_value)
            .done()
            .to::<T>()
    }

    fn write_config<T>(&mut self, section: &str, key: &str, value: T)
    where
        T: ToGodot,
    {
        let value = value.to_variant();
        self.file.set_value(section, key, &value);
    }
}
