//! Auto-generated Rust from .at sources (a2r transpilation).
//!
//! This module contains the a2r-transpiled output of all .at files.
//! It coexists with the hand-written Rust in the parent modules.
//! extern_impl provides glue-layer stubs; real implementations delegate
//! to the hand-written store/registry/agent APIs.

#![allow(dead_code, unused_imports, unused_variables, unused_mut, non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub mod extern_impl;

pub mod app_config;
pub mod auth;
pub mod chats;
pub mod conversation;
pub mod handoff_store;
pub mod hello;
pub mod auto_lib;
pub mod auto_main;
pub mod mode;
pub mod orch_tools;
pub mod relay_api;
pub mod relay_driver;
pub mod relay_flows;
pub mod relay_mod;
pub mod relay_profession;
pub mod relay_store;
pub mod server;
pub mod server_serve;
pub mod server_stream;
pub mod spec_tools;
pub mod task_plan;
pub mod specs;
pub mod tool_context;
pub mod tool_safety;
pub mod tools;
pub mod wiki;
