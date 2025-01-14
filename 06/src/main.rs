use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

mod assembler;

fn main() {
    let command_line_args: Vec<String> = env::args().collect();
    if command_line_args.len() != 2 {
        println!("Usage: assembler <filename>");
        return;
    }
    let source_file_path = PathBuf::from(&command_line_args[1]);
    let output_file_path = {
        // `path/to/target.asm` という形式から `path/to/gen.target.hack` に変換する
        let hack = source_file_path.with_extension("hack");
        let file_name = format!("gen.{}", hack.file_name().unwrap().to_string_lossy());
        match source_file_path.parent() {
            Some(p) => p.join(file_name),
            None => PathBuf::from(file_name),
        }
    };

    let binary = assemble(source_file_path);
    let _ = std::fs::write(output_file_path, &binary);
}

// 任意のpathを渡せるようにしておくとUTが書きやすいので切り出しておく
fn assemble(path: PathBuf) -> String {
    let content = read_to_string(path.clone()).expect("Failed to read file");
    assembler::ParseHackResult::new(content).to_string()
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_assemble() {
        assert_eq!(
            format!("{}\n", assemble(PathBuf::from("test_data/add/Add.asm"))),
            read_to_string("test_data/add/Add.hack").unwrap()
        );
        assert_eq!(
            format!("{}\n", assemble(PathBuf::from("test_data/max/Max.asm"))),
            read_to_string("test_data/max/Max.hack").unwrap()
        );
        assert_eq!(
            format!("{}\n", assemble(PathBuf::from("test_data/max/MaxL.asm"))),
            read_to_string("test_data/max/MaxL.hack").unwrap()
        );
        // NOTE: ↑の3ファイルだけ末尾改行が入ってない or ↓だけ末尾改行が入っちゃってる
        assert_eq!(
            assemble(PathBuf::from("test_data/pong/Pong.asm")),
            read_to_string("test_data/pong/Pong.hack").unwrap()
        );
        assert_eq!(
            assemble(PathBuf::from("test_data/pong/PongL.asm")),
            read_to_string("test_data/pong/PongL.hack").unwrap()
        );
        assert_eq!(
            assemble(PathBuf::from("test_data/rect/RectL.asm")),
            read_to_string("test_data/rect/RectL.hack").unwrap()
        );
        assert_eq!(
            assemble(PathBuf::from("test_data/rect/Rect.asm")),
            read_to_string("test_data/rect/Rect.hack").unwrap()
        );
    }
}
