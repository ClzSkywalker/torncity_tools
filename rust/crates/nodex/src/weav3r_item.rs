use godot::{classes::*, prelude::*};
use weav3r::profit::{ProfitUserInfo, get_bazaar_url};

use crate::profit_panel::ProfitPanel;

#[derive(GodotClass)]
#[class(init,base=PanelContainer)]
pub struct Weav3rItem {
    pub item: ProfitUserInfo,
    #[base]
    base: Base<PanelContainer>,
}

#[godot_api]
impl Weav3rItem {
    #[allow(dead_code)]
    fn init(base: Base<PanelContainer>) -> Self {
        Self {
            item: ProfitUserInfo::default(),
            base,
        }
    }

    pub fn set_item(&mut self, item: ProfitUserInfo) {
        self.item = item;
        // self.update_ui();
    }

    fn update_ui(&mut self) {
        self.base()
            .get_node_as::<Label>("%UserName")
            .set_text(format!("Name: {}", self.item.player_name).as_str());
        self.base()
            .get_node_as::<Label>("%TotalProfit")
            .set_text(format!("Profit: {}", self.item.profit_total_value).as_str());
        let mut link_button = self.base().get_node_as::<LinkButton>("%LinkButton");
        link_button.set_uri(get_bazaar_url(self.item.player_id).as_str());
        link_button.set_text("链接");

        let profit_panel_scene = ResourceLoader::singleton()
            .load("res://scenes/profit_panel.tscn")
            .unwrap()
            .try_cast::<PackedScene>()
            .unwrap();
        let mut vbox = self.base().get_node_as::<VBoxContainer>("%ProfitVBox");
        let children = vbox.get_children();
        for child in children.iter_shared() {
            let mut child = child.clone();
            child.queue_free();
        }
        for item in self.item.items.iter() {
            let mut profit_panel = profit_panel_scene
                .instantiate()
                .unwrap()
                .try_cast::<ProfitPanel>()
                .unwrap();
            profit_panel.bind_mut().set_item(item.clone());
            vbox.add_child(Some(&profit_panel.upcast::<Node>()));
        }
    }
}

#[godot_api]
impl IPanelContainer for Weav3rItem {
    fn ready(&mut self) {
        self.update_ui();
    }
}
