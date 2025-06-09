add-symbol-file target/x86_64-unknown-none/debug/kernel
br test_usermode
target remote localhost:1234
