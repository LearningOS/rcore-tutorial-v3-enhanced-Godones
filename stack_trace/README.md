# 使用说明

与主分支上的相比，这个版本不需要文件系统支持，只需要实现全局分配器能使用`Vec`、`String`等数据结构即可,占用的内存大大减少，不用读取文件系统速度更快。其回溯的想法与主分支相同，只是获取内核所有符号信息的实现不同。

## 思路

为了使得内核符号信息在内核中可用，需要对内核进行两次编译，第一次编译的结果是不包含符号信息的，第二次编译包含符号信息，这样就可以在内核中读取符号信息了。具体的做法是将符号信息组织在`.section .rodata`段，这样在第二次编译链接时就不会破坏代码段的地址信息，然后再内核中导出信息即可。

## 如何获取函数信息

为了获取函数信息，这里使用linux下nm 命令，其可以解析出可执行文件中的符号信息，包括起始地址，符号类型，符号名称等。

使用`nm -n ...`可以按地址递增的顺序打印符号信息。

## trace_exe

这个工具可以将`nm -n`的输出转换为汇编文件，将符号信息写入文件中，具体的格式如下：

```assembly
.section .rodata
.align 3
.global symbol_num
.global symbol_address
.global symbol_index
.global symbol_name
symbol_num:
.quad 0
symbol_address:
symbol_index:
symbol_name:

```

`symbol_num`表示符号数目

`symbol_address`表示符号起始地址

`symbol_index`表示符号的名称起始位置

`symbol_name`部分是符号的名称

## trace_lib

这个库是主分支上的修改版，其只提供回溯的功能。这里为了与内核解耦合，提供了一个`trait`

```rust
pub trait Symbol{
    fn addr(&self)->usize;
    fn name(&self)->&str;
}
```

内核在从汇编文件中读取到符号信息后，需要传入一个包含所有符号信息的数组，数组的每个元素需要实现上述的`trait`.

## 内核代码修改

新增`trace`模块，负责从汇编文件读取符号信息，并实现上述的`trait`，其实现可查看源代码，这里不赘述。

在`lang_item`中，也相应地修改部分代码

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(location) = info.location() {
        println!(
            "[kernel] Panicked at {}:{} {}",
            location.file(),
            location.line(),
            info.message().unwrap()
        );
    } else {
        println!("[kernel] Panicked: {}", info.message().unwrap());
    }
    stack_trace();
    shutdown(255)
}

#[no_mangle]
fn stack_trace() {
    let info = crate::trace::init_kernel_trace();
    let func_info = unsafe { trace_lib::my_trace(info) };
    func_info.iter().for_each(|x| {
        println!("{}", x);
    });
}
```



为了在第二次编译中将内核符号信息链接到内核中，需要在第一次编译中生成一个伪汇编文件，就如上述所展示的那样，这是为了内核能找到对应的符号信息，因为在内核需要声明外部符号

```rust
extern "C" {
    fn symbol_num();
    fn symbol_address();
    fn symbol_index();
    fn symbol_name();
}
```

这里不能使用弱引用链接，`rust`访问没定义的弱引用会出错。因此只能伪造一份没有数据的汇编文件供内核第一次编译使用。在第一次编译完成后就可以使用`nm`命令获取信息并使用`trace_exe`生成新的包含信息的文件给第二次编译使用了。

在`Makefile`中需要添加相应的命令完成上述工作

```makefile
kernel:
	@echo Platform: $(BOARD)
	@#cp src/linker-$(BOARD).ld src/linker.ld
	@touch src/trace/kernel_symbol.S && rm src/trace/kernel_symbol.S
	@cargo build --release --features "board_$(BOARD)" #第一次编译没有链接符号信息
	@#rm src/linker.ld
	@cd ../stack_trace/trace_exe && make && (nm -n ../../os/${KERNEL_ELF} | ./target/release/kernel_trace > ../../os/src/trace/kernel_symbol.S)
	@cd ../../os
	@cargo build --release --features "board_$(BOARD)"  #第二次编译有链接符号信息
```

第一次编译生成伪汇编代码的工作在`build.rs`中完成

```rust
fn main() {
    println!("cargo:rerun-if-changed=../user/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    println!("cargo:rerun-if-changed={}", "src");
    let path = Path::new("src/trace/kernel_symbol.S");
    if !path.exists() {
        let mut file = File::create(path).unwrap();
        write!(file, ".section .rodata\n").unwrap();
        write!(file, ".align 3\n").unwrap();
        write!(file, ".global symbol_num\n").unwrap();
        write!(file, ".global symbol_address\n").unwrap();
        write!(file, ".global symbol_index\n").unwrap();
        write!(file, ".global symbol_name\n").unwrap();
        write!(file, "symbol_num:\n").unwrap();
        write!(file, ".quad {}\n", 0).unwrap();
        write!(file, "symbol_address:\n").unwrap();
        write!(file, "symbol_index:\n").unwrap();
        write!(file, "symbol_name:\n").unwrap();
    }
}
```



## 参考

源代码链接：https://github.com/LearningOS/rcore-tutorial-v3-enhanced-Godones

请查看`lf-dev-1`分支

