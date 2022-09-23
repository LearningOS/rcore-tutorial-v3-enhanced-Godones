mod elf_info;

fn main() {
    let mut entrys = Vec::new();
    loop {
        let mut line = String::new();
        let size= std::io::stdin().read_line(&mut line).unwrap();
        if size==0 {
            break
        }
        let entry = elf_info::entry_from_stdin(line);
        if entry.is_some(){
            entrys.push(entry.unwrap());
        }
    }
    elf_info::re_organize(&mut entrys);
    elf_info::format_link_file(&entrys);
}
