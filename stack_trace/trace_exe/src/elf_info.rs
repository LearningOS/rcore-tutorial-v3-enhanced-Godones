use rustc_demangle::demangle;

#[derive(Debug,Clone)]
pub struct SymbolEntry {
    addr:usize,
    s_type:char,
    symbol_str:String,
    symbol_length:usize
}

impl SymbolEntry{
    pub fn new(addr:usize,s_type:char,symbol_str:String,symbol_length:usize)->Self{
        Self{
            addr,
            s_type,
            symbol_str,
            symbol_length
        }
    }
}


pub fn entry_from_stdin(line:String)->Option<SymbolEntry>{
    let line = line.trim();
    // nm -n 格式
    // address type symbol
    // 0000000000000000 T _DYNAMIC
    let info:Vec<&str> = line.split(' ').collect();
    if info.len()==3 {
        let addr = usize::from_str_radix(info[0],16).unwrap();
        let s_type = info[1].chars().next().unwrap();
        let mut symbol_str = info[2].to_string();
        if symbol_str.starts_with("_ZN") {
            symbol_str = format!("{:#}", demangle(&symbol_str));
        } else {
            symbol_str = format!("{}", symbol_str);
        } //命名重整
        // //去掉一些无用的信息，比如函数代码段内的.LLB段
        if symbol_str.starts_with(".") {
            return None;
        }
        let symbol_length = symbol_str.len();
        return Some(SymbolEntry::new(addr,s_type,symbol_str,symbol_length))
    }
    None
}


pub fn re_organize(entrys:&mut Vec<SymbolEntry>){
    // 返回代码段起始地址,结束地址
    // 对于代码段中可能出现的函数地址重复情况，选择最后一个，剔除其它符号
    // nm -n 会将地址从小到大排序，因此可以从后往前筛选
    for i in (0..entrys.len()).rev(){
        let entry = &entrys[i];
        if entry.symbol_str == "skernel"{
            entrys.drain(0..i);
            break;
        }
    }
}

pub fn format_link_file(entrys:&Vec<SymbolEntry>){
    // 生成链接文件供内核链接
    // 直接使用标准输出并使用管道输出到文件
    println!(".align 3");
    println!(".section .rodata");

    println!(".globl symbol_num");
    println!("symbol_num:");
    println!("\t.quad\t{}",entrys.len());

    println!(".globl symbol_address");
    println!("symbol_address:");
    entrys.iter().for_each(|entry|{
        println!("\t.quad\t{}",entry.addr);
    });

    println!(".globl symbol_index");
    println!("symbol_index:");
    let mut index = 0;
    entrys.iter().for_each(|entry|{
        println!("\t.quad\t{}",index);
        index += entry.symbol_length+1;
    });

    println!(".globl symbol_name");
    println!("symbol_name:");
    entrys.iter().for_each(|entry|{
        println!("\t.asciz\t\"{}\"",entry.symbol_str);
    });
}