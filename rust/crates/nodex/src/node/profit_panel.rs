use godot::{classes::*, prelude::*};
use tools::node::{INodeFunc, INodeTool};
use weav3r::profit::{ProfitInfo, ProfitMetrics};

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
    office_icon: Option<Gd<TextureRect>>,
    vbox_profit_list: Option<Gd<VBoxContainer>>,
}

#[godot_api]
impl IPanelContainer for ProfitPanel {
    fn ready(&mut self) {
        self.name_label = self.get_node_as::<Label>("%Name");
        self.quantity_label = self.get_node_as::<Label>("%Quantity");
        self.image_request = self.get_node_as::<ImageHttpRequest>("%ImageHttpRequest");
        self.office_icon = self.get_node_as::<TextureRect>("%OfficeIcon");
        self.vbox_profit_list = self.get_node_as::<VBoxContainer>("%VBoxProfitList");

        // 设置 Name 和 Quantity 标签
        if let Some(name_label) = self.name_label.as_mut() {
            name_label.set_text(format!("Name:{}", self.item.name).as_str());
        }
        if let Some(quantity_label) = self.quantity_label.as_mut() {
            quantity_label.set_text(format!("Quantity:{}", self.item.quantity).as_str());
        }
        if !self.item.final_profit.is_office
            && let Some(office_icon) = self.office_icon.as_mut()
        {
            office_icon.set_visible(false);
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
            self.item.final_profit.clone(),
            self.item.total_recyle_price(),
            self.item.final_profit.total_sell_price,
        );
        // todo 这个 list 可以考虑干掉
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
    #[base]
    base: Base<Control>,
    profit_single_label: Option<Gd<Label>>,
    profit_total_label: Option<Gd<Label>>,
    profit_percent_label: Option<Gd<Label>>,
    total_price_buy_label: Option<Gd<Label>>,
    total_price_sell_label: Option<Gd<Label>>,
    /// 利润
    pub profit: ProfitMetrics,
    /// 购买总价值
    pub total_recyle_price: u64,
    /// 出售的总价值
    pub total_sell_price: u64,
}

#[godot_api]
impl IControl for ProfitItem {
    fn ready(&mut self) {
        self.profit_single_label = self.get_node_as::<Label>("%ProfitSingleValue");
        self.profit_total_label = self.get_node_as::<Label>("%ProfitTotalValue");
        self.profit_percent_label = self.get_node_as::<Label>("%ProfitPercent");
        self.total_price_buy_label = self.get_node_as::<Label>("%TotalPriceBuy");
        self.total_price_sell_label = self.get_node_as::<Label>("%TotalPriceSell");

        if let Some(profit_single_label) = self.profit_single_label.as_mut() {
            profit_single_label
                .set_text(format!("Single Profit:{}", self.profit.single_profit_value).as_str());
        }
        if let Some(profit_total_label) = self.profit_total_label.as_mut() {
            profit_total_label
                .set_text(format!("Total Profit:{}", self.profit.total_profit_value).as_str());
        }
        if let Some(profit_percent_label) = self.profit_percent_label.as_mut() {
            profit_percent_label
                .set_text(format!("Percent:{:.2}%", self.profit.percentage).as_str());
        }
        if let Some(total_price_buy_label) = self.total_price_buy_label.as_mut() {
            total_price_buy_label.set_text(format!("Buy:{}", self.total_recyle_price).as_str());
        }
        if let Some(total_price_sell_label) = self.total_price_sell_label.as_mut() {
            total_price_sell_label.set_text(format!("Sell:{}", self.total_sell_price).as_str());
        }
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
        profit: ProfitMetrics,
        total_recyle_price: u64,
        total_sell_price: u64,
    ) {
        self.profit = profit;
        self.total_recyle_price = total_recyle_price;
        self.total_sell_price = total_sell_price;
    }
}
