use std::path::PathBuf;

mod translator;

fn main() {
    let command_line_args: Vec<String> = std::env::args().collect();
    if command_line_args.len() != 2 {
        println!("Usage: vm_translator <filepath>");
        return;
    }
    let source_file_path = std::path::PathBuf::from(&command_line_args[1]);
    // `path/to/target.vm` という形式から `path/to/target.asm` に変換する
    let output_file_path = {
        if source_file_path.extension().unwrap() != "vm" {
            println!("Specify a file `*.vm`");
            return;
        }
        source_file_path.with_extension("asm")
    };

    let file_name = source_file_path.file_stem().unwrap().to_string_lossy().to_string();
    let mut vm_program = parse(source_file_path.clone(), file_name);
    let machine_language = vm_program.to_hack_assembly();
    let _ = std::fs::write(output_file_path, &machine_language);
}

// 任意のpathを渡せるようにしておくとUTが書きやすいので切り出しておく
fn parse(path: PathBuf, file_name: String) -> translator::VMProgram {
    let content = std::fs::read_to_string(path.clone()).expect("Failed to read file");
    translator::VMProgram::new(file_name, content)
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use pretty_assertions::assert_eq;
//
//     #[test]
//     fn test_assemble() {
//         assert_eq!(
//             format!("{}\n", assemble(PathBuf::from("test_data/add/Add.asm"))),
//             read_to_string("test_data/add/Add.hack").unwrap()
//         );
//         assert_eq!(
//             format!("{}\n", assemble(PathBuf::from("test_data/max/Max.asm"))),
//             read_to_string("test_data/max/Max.hack").unwrap()
//         );
//         assert_eq!(
//             format!("{}\n", assemble(PathBuf::from("test_data/max/MaxL.asm"))),
//             read_to_string("test_data/max/MaxL.hack").unwrap()
//         );
//         // NOTE: ↑の3ファイルだけ末尾改行が入ってない or ↓だけ末尾改行が入っちゃってる
//         assert_eq!(
//             assemble(PathBuf::from("test_data/pong/Pong.asm")),
//             read_to_string("test_data/pong/Pong.hack").unwrap()
//         );
//         assert_eq!(
//             assemble(PathBuf::from("test_data/pong/PongL.asm")),
//             read_to_string("test_data/pong/PongL.hack").unwrap()
//         );
//         assert_eq!(
//             assemble(PathBuf::from("test_data/rect/RectL.asm")),
//             read_to_string("test_data/rect/RectL.hack").unwrap()
//         );
//         assert_eq!(
//             assemble(PathBuf::from("test_data/rect/Rect.asm")),
//             read_to_string("test_data/rect/Rect.hack").unwrap()
//         );
//     }
// }
