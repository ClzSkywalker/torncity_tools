use crate::{http_weav3r::Weav3rHttpRequest, prelude::Weav3rItem};
use godot::{
    classes::{
        Button, ConfigFile, Control, GridContainer, IControl, Node, PackedScene, ResourceLoader,
        Timer,
    },
    global::Error,
    prelude::*,
};
use weav3r::profit::{FavoritesRes, ProfitUserInfo};

pub mod http_weav3r;
pub mod prelude;
pub mod profit_panel;
pub mod weav3r_item;

const BUTTON_PATH: &str = "Button";
const HTTP_REQUEST_PATH: &str = "HTTPRequest";
const GRID_CONTAINER_PATH: &str = "ScrollContainer/GridContainer";
const TIMER_PATH: &str = "Timer";
const SETTINGS_BUTTON_PATH: &str = "%SettingsButton";
const SETTINGS_SCENE_PATH: &str = "res://scenes/settings.tscn";
const CONFIG_PATH: &str = "user://settings.cfg";
const CONFIG_SECTION: &str = "settings";
const CONFIG_KEY_INTERVAL: &str = "interval";
const CONFIG_KEY_PROFIT_PERCENT: &str = "profit_percent";
const CONFIG_KEY_MIN_PROFIT: &str = "min_profit";
const CONFIG_KEY_FILTER_IDS: &str = "filter_ids";
const DEFAULT_INTERVAL: f64 = 5.0;
const DEFAULT_PROFIT_PERCENT: f64 = 1.0;
const DEFAULT_MIN_PROFIT: i64 = 10000;
const DEFAULT_FILTER_IDS: &str = "385,183,97,902,901,904,129,184,260,903,263,617,272,264,271,267,277,282,276,186,187,215,261,618,273,258,266,268,269,281,274,384,533,555,532,554,530,553,987,986,985,206,586,587,151,556,529,528,36,527,310,35,210,39,37,209,38,541,552,542,638,551,531,550,818,283,370,364,1080,1079,1082,1083,1078,1081,367,366,1485,1486,1494,358";

#[derive(GodotClass)]
#[class(base=Control)]
pub struct UiController {
    #[base]
    base: Base<Control>,
    favorites_res: FavoritesRes,
}

#[godot_api]
impl IControl for UiController {
    fn init(base: Base<Control>) -> Self {
        UiController {
            base,
            favorites_res: FavoritesRes::default(),
        }
    }

    fn ready(&mut self) {
        if let Some(button) = self.get_node_as::<Button>(BUTTON_PATH) {
            button
                .signals()
                .pressed()
                .connect_other(self, Self::on_button_pressed);
        } else {
            godot_error!("UiController: Button node not found.");
        }

        if let Some(http) = self.get_node_as::<Weav3rHttpRequest>(HTTP_REQUEST_PATH) {
            http.signals()
                .request_completed()
                .connect_other(self, Self::on_request_completed);
        } else {
            godot_error!("UiController: HTTPRequest node not found.");
        }

        let interval = self.read_config_f64(CONFIG_KEY_INTERVAL, DEFAULT_INTERVAL);

        if let Some(mut timer) = self.get_node_as::<Timer>(TIMER_PATH) {
            timer.set_wait_time(interval);
            timer
                .signals()
                .timeout()
                .connect_other(self, Self::on_timer_timeout);
        } else {
            godot_error!("UiController: Timer node not found.");
        }

        if let Some(settings_button) = self.get_node_as::<Button>(SETTINGS_BUTTON_PATH) {
            settings_button
                .signals()
                .pressed()
                .connect_other(self, Self::on_settings_pressed);
        } else {
            godot_error!("UiController: SettingsButton node not found.");
        }
    }
}

#[godot_api]
impl UiController {
    #[func]
    fn on_button_pressed(&mut self) {
        let filter_id_text = self.read_config_string(CONFIG_KEY_FILTER_IDS, DEFAULT_FILTER_IDS);
        if filter_id_text.trim().is_empty() {
            godot_error!("UiController: FilterIdEdit is empty.");
            return;
        }
        let target_ids = filter_id_text.split(',').collect::<Vec<&str>>().join(",");

        let Some(mut http) = self.get_node_as::<Weav3rHttpRequest>(HTTP_REQUEST_PATH) else {
            godot_error!("UiController: HTTPRequest node not found.");
            return;
        };

        http.bind_mut().send_request(GString::from(&target_ids));
    }

    #[func]
    fn on_timer_timeout(&mut self) {
        self.on_button_pressed();
    }

    #[func]
    fn on_settings_pressed(&mut self) {
        let Some(mut tree) = self.base().get_tree() else {
            godot_error!("UiController: SceneTree not found.");
            return;
        };
        let err = tree.change_scene_to_file(SETTINGS_SCENE_PATH);
        if err != Error::OK {
            godot_error!("UiController: Failed to open settings scene: {:?}", err);
        }
    }

    #[func]
    fn on_request_completed(
        &mut self,
        _result: i64,
        response_code: i64,
        _headers: PackedStringArray,
        body: PackedByteArray,
    ) {
        if response_code != 200 {
            godot_error!(
                "UiController: Failed to get response.code: {}",
                response_code
            );
            return;
        }

        let profit_percentage =
            self.read_config_f64(CONFIG_KEY_PROFIT_PERCENT, DEFAULT_PROFIT_PERCENT);
        let profit_minimum_value =
            self.read_config_i64(CONFIG_KEY_MIN_PROFIT, DEFAULT_MIN_PROFIT) as i32;

        let response_text = String::from_utf8_lossy(body.as_slice());
        let Ok(favorites_response) = weav3r::favorite::parse_favorites_response(&response_text)
        else {
            godot_error!("UiController: Failed to parse favorites response.");
            return;
        };
        let profit_items = weav3r::profit::filter(
            favorites_response,
            weav3r::profit::Filter {
                min_profit: profit_minimum_value,
                min_profit_percentage: profit_percentage as f32,
                ignore_names: vec![],
                filter_items: vec![
                    weav3r::profit::FilterItem {
                        name: "Xanax".to_string(),
                        price: 800000,
                        // profit_percentage: 10.0,
                        ..Default::default()
                    },
                    weav3r::profit::FilterItem {
                        name: "Donator Pack".to_string(),
                        price: 23000000,
                        // profit_percentage: 10.0,
                        ..Default::default()
                    },
                    weav3r::profit::FilterItem {
                        name: "Feathery Hotel Coupon".to_string(),
                        price: 12000000,
                        // profit_percentage: 10.0,
                        ..Default::default()
                    },
                ],
            },
        );

        let resp = weav3r::profit::calc_profit(self.favorites_res.clone(), profit_items);
        self.favorites_res = resp.clone();
        self.render_list(resp.user_profit_result);
    }

    fn render_list(&mut self, items: Vec<ProfitUserInfo>) {
        let Some(mut grid_container) = self.get_node_as::<GridContainer>(GRID_CONTAINER_PATH)
        else {
            godot_error!("UiController: GridContainer node not found.");
            return;
        };

        let Some(scene) = ResourceLoader::singleton()
            .load("res://scenes/weav3r_item.tscn")
            .and_then(|res| res.try_cast::<PackedScene>().ok())
        else {
            godot_error!("UiController: Failed to load weav3r_item.tscn");
            return;
        };

        let children = grid_container.get_children();
        for child in children.iter_shared() {
            let mut child = child.clone();
            child.queue_free();
        }

        for item in items {
            let Some(node) = scene.instantiate() else {
                godot_error!("UiController: Failed to instance Weav3rItem");
                continue;
            };
            let Ok(mut item_node) = node.try_cast::<Weav3rItem>() else {
                godot_error!("UiController: Instance is not Weav3rItem");
                continue;
            };
            item_node.bind_mut().set_item(item);
            let child = item_node.upcast::<Node>();
            // vbox.add_child(Some(&child));
            grid_container.add_child(Some(&child));
        }
    }

    fn load_config(&self) -> Gd<ConfigFile> {
        let mut config = ConfigFile::new_gd();
        let _err = config.load(CONFIG_PATH);
        config
    }

    fn read_config_f64(&self, key: &str, default: f64) -> f64 {
        let config = self.load_config();
        let default_value = Variant::from(default);
        config
            .get_value_ex(CONFIG_SECTION, key)
            .default(&default_value)
            .done()
            .to::<f64>()
    }

    fn read_config_i64(&self, key: &str, default: i64) -> i64 {
        let config = self.load_config();
        let default_value = Variant::from(default);
        config
            .get_value_ex(CONFIG_SECTION, key)
            .default(&default_value)
            .done()
            .to::<i64>()
    }

    fn read_config_string(&self, key: &str, default: &str) -> String {
        let config = self.load_config();
        let default_value = Variant::from(GString::from(default));
        let value = config
            .get_value_ex(CONFIG_SECTION, key)
            .default(&default_value)
            .done()
            .to::<GString>();
        value.to_string()
    }

    fn get_node_as<T>(&self, path: &str) -> Option<Gd<T>>
    where
        T: GodotClass + Inherits<Node>,
    {
        self.base()
            .get_node_or_null(path)
            .and_then(|node| node.try_cast::<T>().ok())
    }
}
