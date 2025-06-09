add-symbol-file target/x86_64-unknown-none/debug/kernel
br test_usermode
br cpu.rs:radian_core::cpu::Manager::dummy_int_handler
br task.rs:radian_core::task::Manager::switch_to_usermode
target remote localhost:1234
