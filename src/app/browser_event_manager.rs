// src/app/browser_event_manager.rs
//! Handles attaching and detaching browser event listeners (mousemove, mouseup) during drag operations.

use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, Event, MouseEvent, HtmlCanvasElement};
use crate::ecs::world::World;
use crate::network::NetworkManager;
use crate::app::drag_handler; // update_dragged_position, handle_drag_end を呼び出すため
use crate::log; // log マクロのみをインポート
use log::error; // ★追加: error! マクロを正しくインポート

/// Attaches mousemove and mouseup listeners to the window for drag updates and end detection.
pub(crate) fn attach_drag_listeners(
    world_arc: Arc<Mutex<World>>,
    network_manager_arc: Arc<Mutex<NetworkManager>>, // handle_drag_end で必要
    window_mousemove_closure_arc: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    window_mouseup_closure_arc: Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    entity_id: usize, // The entity being dragged
    canvas: &HtmlCanvasElement, // ★ 追加: Canvas 要素への参照 ★
) -> Result<(), JsValue> {
    log(&format!("Attaching drag listeners for entity {}", entity_id));

    // --- MouseMove Listener ---
    {
        // Clone Arcs for the closure
        let world_arc_clone = Arc::clone(&world_arc);
        let canvas_clone = canvas.clone();
        let _window_mousemove_closure_arc_clone = Arc::clone(&window_mousemove_closure_arc); // ★ 変数名に _ を追加 ★
        let _window_mouseup_closure_arc_clone = Arc::clone(&window_mouseup_closure_arc); // mouseup closure might need access inside mousemove? Unlikely but pass for now. ★ 変数名に _ を追加 ★

        let mousemove_closure = Closure::wrap(Box::new(move |event: Event| {
            // Cast the generic Event to a MouseEvent
            if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
                // ★★★ 座標変換ロジックを追加 ★★★
                let rect = canvas_clone.get_bounding_client_rect();
                let mouse_x = mouse_event.client_x() as f32 - rect.left() as f32;
                let mouse_y = mouse_event.client_y() as f32 - rect.top() as f32;
                // ★★★ ここまで ★★★

                // Directly call the update function (which locks the world)
                drag_handler::update_dragged_position(
                    &world_arc_clone, // Pass the cloned Arc
                    entity_id,
                    mouse_x, // ★ 変換後の座標を使う！ ★
                    mouse_y, // ★ 変換後の座標を使う！ ★
                );
            } else {
                error!("Failed to cast event to MouseEvent in mousemove listener");
            }
        }) as Box<dyn FnMut(Event)>);

        let window = window().ok_or("Failed to get window")?;
        window.add_event_listener_with_callback(
            "mousemove",
            mousemove_closure.as_ref().unchecked_ref(),
        )?;
        *window_mousemove_closure_arc.lock().expect("Failed to lock mousemove closure arc") = Some(mousemove_closure); // Store the closure
        log("  Attached mousemove listener.");
    }

    // --- MouseUp Listener ---
    {
        // Clone Arcs for the closure
        let world_arc_clone = Arc::clone(&world_arc);
        let network_manager_arc_clone = Arc::clone(&network_manager_arc);
        let _window_mousemove_closure_arc_clone = Arc::clone(&window_mousemove_closure_arc); // Need to detach this listener
        let _window_mouseup_closure_arc_clone = Arc::clone(&window_mouseup_closure_arc);   // Need to detach this listener

        let mouseup_closure = Closure::wrap(Box::new(move |event: Event| {
            log(&format!("MouseUp triggered for entity {}", entity_id));
            // Cast the generic Event to a MouseEvent
            if let Ok(mouse_event) = event.dyn_into::<MouseEvent>() {
                let end_x = mouse_event.client_x() as f32;
                let end_y = mouse_event.client_y() as f32;

                // --- Call handle_drag_end logic ---
                // This logic now resides directly in drag_handler
                drag_handler::handle_drag_end(
                    &world_arc_clone,
                    &network_manager_arc_clone,
                    entity_id,
                    end_x,
                    end_y,
                );

                // ★ Detachment is now called from within handle_drag_end or here? ★
                // Let's call detach from here AFTER handle_drag_end logic finishes.
                // This ensures listeners are removed even if handle_drag_end itself doesn't call detach.
                log("  Detaching listeners from within mouseup closure...");
                 if let Err(e) = detach_drag_listeners(
                     &_window_mousemove_closure_arc_clone, // Pass Arcs again
                     &_window_mouseup_closure_arc_clone,
                 ) {
                    error!("Error detaching listeners in mouseup: {:?}", e);
                 }

            } else {
                error!("Failed to cast event to MouseEvent in mouseup listener");
            }
        }) as Box<dyn FnMut(Event)>);

        let window = window().ok_or("Failed to get window")?;
        window.add_event_listener_with_callback(
            "mouseup",
            mouseup_closure.as_ref().unchecked_ref(),
        )?;
        *window_mouseup_closure_arc.lock().expect("Failed to lock mouseup closure arc") = Some(mouseup_closure); // Store the closure
        log("  Attached mouseup listener.");
    }

    Ok(())
}

/// Detaches the mousemove and mouseup listeners from the window.
pub(crate) fn detach_drag_listeners(
    window_mousemove_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    window_mouseup_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
) -> Result<(), JsValue> {
    log("Detaching drag listeners...");
    let window = window().ok_or("Failed to get window")?;

    // --- Remove MouseMove Listener ---
    // Lock the mutex, take the closure Option, and if it's Some, remove the listener
    if let Some(closure) = window_mousemove_closure_arc.lock().expect("Failed to lock mousemove closure arc").take() {
        window.remove_event_listener_with_callback(
            "mousemove",
            closure.as_ref().unchecked_ref(),
        )?;
        // Closure is dropped here when it goes out of scope (because we used take())
        log("  Detached mousemove listener.");
    } else {
        log("  Mousemove listener was already detached or never attached.");
    }

    // --- Remove MouseUp Listener ---
    if let Some(closure) = window_mouseup_closure_arc.lock().expect("Failed to lock mouseup closure arc").take() {
        window.remove_event_listener_with_callback(
            "mouseup",
            closure.as_ref().unchecked_ref(),
        )?;
        log("  Detached mouseup listener.");
    } else {
        log("  Mouseup listener was already detached or never attached.");
    }

    Ok(())
}

/// Detaches the canvas-specific event listeners (click, dblclick, mousedown).
pub(crate) fn detach_canvas_listeners(
    canvas: &HtmlCanvasElement,
    click_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    dblclick_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
    mousedown_closure_arc: &Arc<Mutex<Option<Closure<dyn FnMut(Event)>>>>,
) -> Result<(), JsValue> {
    log("Detaching canvas listeners...");

    // --- Remove Click Listener ---
    if let Some(closure) = click_closure_arc.lock().expect("Failed to lock click closure arc").take() {
        canvas.remove_event_listener_with_callback(
            "click",
            closure.as_ref().unchecked_ref(),
        )?;
        log("  Detached canvas click listener.");
    } else {
        log("  Canvas click listener was already detached or never attached.");
    }

    // --- Remove DblClick Listener ---
    if let Some(closure) = dblclick_closure_arc.lock().expect("Failed to lock dblclick closure arc").take() {
        canvas.remove_event_listener_with_callback(
            "dblclick",
            closure.as_ref().unchecked_ref(),
        )?;
        log("  Detached canvas dblclick listener.");
    } else {
        log("  Canvas dblclick listener was already detached or never attached.");
    }

    // --- Remove MouseDown Listener ---
    if let Some(closure) = mousedown_closure_arc.lock().expect("Failed to lock mousedown closure arc").take() {
        canvas.remove_event_listener_with_callback(
            "mousedown",
            closure.as_ref().unchecked_ref(),
        )?;
        log("  Detached canvas mousedown listener.");
    } else {
        log("  Canvas mousedown listener was already detached or never attached.");
    }

    Ok(())
} 