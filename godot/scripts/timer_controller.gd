extends Button


func _ready() -> void:
	button_down.connect(_on_toggle_timer)


func _on_toggle_timer() -> void:
	var timer := get_node_or_null("../../Timer") as Timer
	if timer == null:
		push_error("TimerController: Timer node not found.")
		return
	timer.paused = !timer.paused
	text = "Resume Timer" if timer.paused else "Pause Timer"
