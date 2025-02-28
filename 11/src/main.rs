use compiler::ast;

mod analyzer;
mod compiler;

fn main() {
    let command_line_args: Vec<String> = std::env::args().collect();
    if command_line_args.len() != 2 {
        println!("Usage: jack_analyzer <filepath>");
        return;
    }
    let source_file_path = std::path::PathBuf::from(&command_line_args[1]);
    let target_files = get_target_files(&source_file_path).unwrap();

    for target in target_files {
        let content = std::fs::read_to_string(target.clone()).unwrap();
        let tokens = analyzer::token::Tokens::new(content);
        let output_file_path = target.with_extension("vm");
        let ast = ast::Ast::new(tokens.tokens, target.file_stem().unwrap().to_string_lossy().to_string());
        let vm = ast.to_vm();
        let _ = std::fs::write(output_file_path, vm);
    }
}

fn get_target_files(input_path: &std::path::Path) -> Option<Vec<std::path::PathBuf>> {
    // inputがファイルだったら.jackかどうか判定して(target_files, output_file_path)を返す
    if input_path.is_file() {
        if input_path.extension().unwrap() != "jack" {
            return None;
        }
        return Some(vec![input_path.to_path_buf()]);
    }
    if !input_path.is_dir() {
        return None;
    }

    // inputがフォルダだったら.jackファイルを探してきて(target_files, output_file_path)を返す
    let mut result = vec![];
    for e in std::fs::read_dir(input_path).unwrap() {
        let e_path = e.unwrap().path();
        if e_path.extension().unwrap() == "jack" {
            result.push(e_path);
        }
    }

    Some(result)
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    #[test]
    fn test_get_target_files() {
        // cleanup before test
        let test_root_dir = std::env::temp_dir().join("nand2tetris08");
        // error will be returned if the directory does not exist.
        let _ = std::fs::remove_dir_all(&test_root_dir);
        std::fs::create_dir(&test_root_dir).unwrap();

        // fileのパスを渡す(拡張子がjack以外)
        {
            let test_target_dir = test_root_dir.join(Uuid::new_v4().to_string());
            std::fs::create_dir(&test_target_dir).unwrap();

            let file_path = test_target_dir.join("foo.md");
            std::fs::File::create(&file_path).unwrap();
            assert_eq!(get_target_files(&file_path), None);
        }

        // fileのパスを渡す(拡張子がjack)
        {
            let test_target_dir = test_root_dir.join(Uuid::new_v4().to_string());
            std::fs::create_dir(&test_target_dir).unwrap();

            let file_path = test_target_dir.join("foo.jack");
            std::fs::File::create(&file_path).unwrap();
            assert_eq!(get_target_files(&file_path), Some(vec![file_path]));
        }

        // dirのパスを渡す
        {
            let test_target_dir = test_root_dir.join(Uuid::new_v4().to_string());
            std::fs::create_dir(&test_target_dir).unwrap();

            let parent_dir = test_target_dir.join("Foo");
            std::fs::create_dir(&parent_dir).unwrap();
            let main = parent_dir.join("Main.jack");
            std::fs::File::create(&main).unwrap();
            let child1 = parent_dir.join("Child1.jack");
            std::fs::File::create(&child1).unwrap();
            let child2 = parent_dir.join("Child2.jack");
            std::fs::File::create(&child2).unwrap();
            let other = parent_dir.join("README.md");
            std::fs::File::create(&other).unwrap();

            let result = get_target_files(&parent_dir.clone()).unwrap();
            assert_eq!(result.len(), 3);
            assert_eq!(result.contains(&main), true);
            assert_eq!(result.contains(&child1), true);
            assert_eq!(result.contains(&child2), true);
            assert_eq!(result.contains(&other), false);
        }

        let _ = std::fs::remove_dir_all(&test_root_dir);
    }
}
