add-symbol-file target/x86_64-unknown-none/debug/kernel
br vmm.rs:radian_core::vmm::Manager::evil_function_do_not_call_except_on_init
target remote localhost:1234
