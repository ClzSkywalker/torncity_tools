use godot::{classes::*, prelude::*};
use tools::node::{INodeFunc, INodeTool};
use weav3r::profit::{ProfitUserInfo, get_bazaar_url};

use crate::profit_panel::ProfitPanel;

#[derive(GodotClass)]
#[class(init,base=PanelContainer)]
pub struct Weav3rItem {
    pub item: ProfitUserInfo,
    #[base]
    base: Base<PanelContainer>,
    user_name: Option<Gd<Label>>,
    total_profit: Option<Gd<Label>>,
    link_button: Option<Gd<LinkButton>>,
    profit_vbox: Option<Gd<VBoxContainer>>,
    top_bar: Option<Gd<PanelContainer>>,
}

#[godot_api]
impl IPanelContainer for Weav3rItem {
    fn ready(&mut self) {
        self.user_name = self.get_node_as::<Label>("%UserName");
        self.total_profit = self.get_node_as::<Label>("%TotalProfit");
        self.link_button = self.get_node_as::<LinkButton>("%LinkButton");
        self.profit_vbox = self.get_node_as::<VBoxContainer>("%ProfitVBox");
        self.top_bar = self.get_node_as::<PanelContainer>("VBoxContainer/UserInfo");
        self.update_ui();
    }
}

impl INodeFunc for Weav3rItem {
    fn node_path() -> &'static str {
        "res://scenes/weav3r_item.tscn"
    }
}

#[godot_api]
impl Weav3rItem {
    pub fn set_item(&mut self, item: ProfitUserInfo) {
        self.item = item;
    }

    fn update_ui(&mut self) {
        if let Some(user_name) = self.user_name.as_mut() {
            user_name.set_text(format!("Name:{}", self.item.player_name).as_str());
        }
        if let Some(total_profit) = self.total_profit.as_mut() {
            total_profit.set_text(format!("Profit:{}", self.item.profit_total_value).as_str());
        }
        if let Some(link_button) = self.link_button.as_mut() {
            link_button.set_uri(get_bazaar_url(self.item.player_id).as_str());
            link_button.set_text("Link");
        }

        if let Some(top_bar) = self.top_bar.as_mut() {
            let sec = tools::time::get_current_time() - self.item.created_on;
            if sec <= 30 {
                let mut stylebox = StyleBoxFlat::new_gd();
                let max_sec = 40.0;
                let t = (sec as f32 / max_sec).clamp(0.0, 1.0);
                // sec 越大颜色越浅：从中等绿过渡到浅绿
                let start_r = 80.0;
                let start_g = 220.0;
                let start_b = 80.0;
                let end_r = 200.0;
                let end_g = 255.0;
                let end_b = 200.0;
                let r = (start_r + (end_r - start_r) * t).round() as u8;
                let g = (start_g + (end_g - start_g) * t).round() as u8;
                let b = (start_b + (end_b - start_b) * t).round() as u8;
                stylebox.set_bg_color(Color::from_rgba8(r, g, b, 255));
                top_bar.add_theme_stylebox_override("panel", Some(&stylebox));
            }
        }

        let Some(vbox) = self.profit_vbox.as_mut() else {
            godot_error!("Weav3rItem: Failed to get profit_vbox");
            return;
        };

        let children = vbox.get_children();
        for child in children.iter_shared() {
            let mut child = child.clone();
            child.queue_free();
        }

        for (i, item) in self.item.items.iter().enumerate() {
            // 在非第一个元素前添加分隔线
            if i > 0 {
                let separator = HSeparator::new_alloc();
                vbox.add_child(Some(&separator.upcast::<Node>()));
            }

            let Some(mut profit_panel) = ProfitPanel::get_scene_instance() else {
                godot_error!("Weav3rItem: Failed to get profit_panel");
                return;
            };
            profit_panel.bind_mut().set_item(item.clone());
            vbox.add_child(Some(&profit_panel.upcast::<Node>()));
        }
    }
}
