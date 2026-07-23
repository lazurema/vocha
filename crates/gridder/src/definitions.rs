pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[macro_export]
macro_rules! app_name {
    () => {
        "Voĉa Gridder"
    };
}

#[macro_export]
macro_rules! scope_name {
    () => {
        "Lazurema"
    };
}

#[macro_export]
macro_rules! app_title {
    () => {
        concat!(crate::app_name!(), " @ ", crate::scope_name!())
    };
}
