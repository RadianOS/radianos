add-symbol-file target/x86_64-unknown-none/debug/kernel
br test_usermode
br cpu.rs:radian_core::cpu::Manager::dummy_int_handler
br task.rs:radian_core::task::Manager::switch_to_usermode
# set $i = 0
# while ($i < 4096)
#     br *(&GLOBAL_IDT_ASM + $i)
#     set $i = $i + 16
# end
#br kernel.rs:323
br test_self
target remote localhost:1234
