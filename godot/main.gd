extends Node2D
## Spike scene: confirms the Rust GDExtension loaded and train-core is callable.
## Replaced by the real board/HUD scenes as we port the prototype UI.

func _ready() -> void:
	var engine := RailEngine.new()
	$UI/Status.text = engine.status()
