use godot::{classes::*, prelude::*};
use tools::node::{INodeFunc, INodeTool};
use weav3r::profit::ProfitInfo;

use crate::node::http::image::ImageHttpRequest;

#[derive(GodotClass)]
#[class(init,base=PanelContainer)]
pub struct ProfitPanel {
    pub item: ProfitInfo,
    #[base]
    base: Base<PanelContainer>,
    name_label: Option<Gd<Label>>,
    quantity_label: Option<Gd<Label>>,
    image_request: Option<Gd<ImageHttpRequest>>,
    vbox_profit_list: Option<Gd<VBoxContainer>>,
}

#[godot_api]
impl IPanelContainer for ProfitPanel {
    fn ready(&mut self) {
        self.name_label = self.get_node_as::<Label>("%Name");
        self.quantity_label = self.get_node_as::<Label>("%Quantity");
        self.image_request = self.get_node_as::<ImageHttpRequest>("%ImageHttpRequest");
        self.vbox_profit_list = self.get_node_as::<VBoxContainer>("%VBoxProfitList");

        // 设置 Name 和 Quantity 标签
        if let Some(name_label) = self.name_label.as_mut() {
            name_label.set_text(format!("Name:{}", self.item.name).as_str());
        }
        if let Some(quantity_label) = self.quantity_label.as_mut() {
            quantity_label.set_text(format!("Quantity:{}", self.item.quantity).as_str());
        }
        if let Some(image_request) = self.image_request.as_mut() {
            image_request
                .bind_mut()
                .set_url_request(GString::from(self.item.image.as_str()));
        }

        let Some(mut profit_item) = ProfitItem::get_scene_instance() else {
            godot_error!("ProfitPanel: Failed to instantiate profit_item_scene");
            return;
        };
        profit_item.bind_mut().set_value(
            "Final Profit".to_string(),
            self.item.final_profit.single_value,
            self.item.final_profit.total_value,
            self.item.final_profit.percentage,
        );
        if let Some(vbox_profit_list) = self.vbox_profit_list.as_mut() {
            vbox_profit_list.add_child(Some(&profit_item.upcast::<Node>()));
        }

        let Some(mut profit_item) = ProfitItem::get_scene_instance() else {
            godot_error!("ProfitPanel: Failed to get profit_item_scene");
            return;
        };
        profit_item.bind_mut().set_value(
            "Market Profit".to_string(),
            self.item.market_profit.single_value,
            self.item.market_profit.total_value,
            self.item.market_profit.percentage,
        );
        if let Some(vbox_profit_list) = self.vbox_profit_list.as_mut() {
            vbox_profit_list.add_child(Some(&profit_item.upcast::<Node>()));
        }

        let Some(mut profit_item) = ProfitItem::get_scene_instance() else {
            godot_error!("ProfitPanel: Failed to get profit_item_scene");
            return;
        };
        profit_item.bind_mut().set_value(
            "Avg Bazaar Profit".to_string(),
            self.item.avg_bazaar_profit.single_value,
            self.item.avg_bazaar_profit.total_value,
            self.item.avg_bazaar_profit.percentage,
        );
        if let Some(vbox_profit_list) = self.vbox_profit_list.as_mut() {
            vbox_profit_list.add_child(Some(&profit_item.upcast::<Node>()));
        }
    }
}

impl INodeFunc for ProfitPanel {
    fn node_path() -> &'static str {
        "res://scenes/profit_panel.tscn"
    }
}

impl ProfitPanel {
    pub fn set_item(&mut self, item: ProfitInfo) {
        self.item = item;
    }
}

#[derive(GodotClass)]
#[class(init,base=Control)]
pub struct ProfitItem {
    pub label: String,
    pub single_value: i64,
    pub total_value: i64,
    pub percentage: f32,
    #[base]
    base: Base<Control>,
}

#[godot_api]
impl IControl for ProfitItem {
    fn ready(&mut self) {
        self.base()
            .get_node_as::<Label>("%Label")
            .set_text(format!("Name:{}", self.label).as_str());
        self.base()
            .get_node_as::<Label>("%ProfitSingleValue")
            .set_text(format!("Single Profit:{}", self.single_value).as_str());
        self.base()
            .get_node_as::<Label>("%ProfitTotalValue")
            .set_text(format!("Total Profit:{}", self.total_value).as_str());
        self.base()
            .get_node_as::<Label>("%ProfitPercent")
            .set_text(format!("Percent:{:.2}%", self.percentage).as_str());
    }
}

impl INodeFunc for ProfitItem {
    fn node_path() -> &'static str {
        "res://scenes/profit_item.tscn"
    }
}

impl ProfitItem {
    pub fn set_value(
        &mut self,
        label: String,
        single_value: i64,
        total_value: i64,
        percentage: f32,
    ) {
        self.label = label;
        self.single_value = single_value;
        self.total_value = total_value;
        self.percentage = percentage;
    }
}
