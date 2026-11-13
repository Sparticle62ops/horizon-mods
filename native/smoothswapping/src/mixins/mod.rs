//! Upstream `mixin/` package, 1:1. Java mixins inject into game classes;
//! the port carries each mixin's handler as a plain module against the
//! game seams, keeping every injection site's shape and comment.

pub mod abstract_container_screen;
pub mod abstract_recipe_book_screen;
pub mod gui_graphics_extractor;
pub mod item_stack;
pub mod serverbound_container_click_packet;
pub mod simple_container_accessor;
