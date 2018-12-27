#![feature(proc_macro_hygiene, decl_macro)]

mod cache;
mod content;
mod paste_app;

fn main() {
    paste_app::start();
}
