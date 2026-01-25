extends Control

const CONFIG_PATH := "user://settings.cfg"
const CONFIG_SECTION := "settings"
const CONFIG_KEY_INTERVAL := "interval"
const CONFIG_KEY_PROFIT_PERCENT := "profit_percent"
const CONFIG_KEY_MIN_PROFIT := "min_profit"
const CONFIG_KEY_FILTER_IDS := "filter_ids"
const DEFAULT_INTERVAL := 5.0
const DEFAULT_PROFIT_PERCENT := 1.0
const DEFAULT_MIN_PROFIT := 10000
const DEFAULT_FILTER_IDS := "385,183,97,902,901,904,129,184,260,903,263,617,272,264,271,267,277,282,276,186,187,215,261,618,273,258,266,268,269,281,274,384,533,555,532,554,530,553,987,986,985,206,586,587,151,556,529,528,36,527,310,35,210,39,37,209,38,541,552,542,638,551,531,550,818,283,370,364,1080,1079,1082,1083,1078,1081,367,366,1485,1486,1494,358"
const MAIN_SCENE_PATH := "res://scenes/main.tscn"

@onready var interval_edit: TextEdit = %IntervalEdit
@onready var profit_percent_edit: TextEdit = %ProfitPercentEdit
@onready var min_profit_edit: TextEdit = %MinProfitEdit
@onready var filter_id_edit: TextEdit = %FilterIdEdit
@onready var save_button: Button = %SaveButton


func _ready() -> void:
	var config := ConfigFile.new()
	var _err := config.load(CONFIG_PATH)
	interval_edit.text = str(config.get_value(CONFIG_SECTION, CONFIG_KEY_INTERVAL, DEFAULT_INTERVAL))
	profit_percent_edit.text = str(config.get_value(CONFIG_SECTION, CONFIG_KEY_PROFIT_PERCENT, DEFAULT_PROFIT_PERCENT))
	min_profit_edit.text = str(config.get_value(CONFIG_SECTION, CONFIG_KEY_MIN_PROFIT, DEFAULT_MIN_PROFIT))
	filter_id_edit.text = str(config.get_value(CONFIG_SECTION, CONFIG_KEY_FILTER_IDS, DEFAULT_FILTER_IDS))

	save_button.pressed.connect(_on_save_pressed)


func _on_save_pressed() -> void:
	var config := ConfigFile.new()
	var _err := config.load(CONFIG_PATH)
	config.set_value(CONFIG_SECTION, CONFIG_KEY_INTERVAL, interval_edit.text.to_float())
	config.set_value(CONFIG_SECTION, CONFIG_KEY_PROFIT_PERCENT, profit_percent_edit.text.to_float())
	config.set_value(CONFIG_SECTION, CONFIG_KEY_MIN_PROFIT, int(min_profit_edit.text))
	config.set_value(CONFIG_SECTION, CONFIG_KEY_FILTER_IDS, filter_id_edit.text.strip_edges())
	config.save(CONFIG_PATH)
	get_tree().change_scene_to_file(MAIN_SCENE_PATH)
