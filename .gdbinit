add-symbol-file target/x86_64-unknown-none/debug/kernel
br gdt.rs:radian_core::gdt::Manager::reload_segments
target remote localhost:1234
